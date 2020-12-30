use bincode;
use std::io::prelude::*;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;

use crate::message::*;
use crate::transform::*;

use super::MAX_MSG_LEN;
use super::MAX_PAYLOAD_LEN;


pub struct ServerStats {
    bytes_sent: u64,      //all bytes sent, including headers
    bytes_received: u64,  //all bytes received, including headers
    compress_sent: u64,   //all -valid- bytes sent (excludes invalid compression requests)
    compress_rcv: u64,    //all -valid- bytes received (excludes invalid compression requests)
    decompress_sent: u64, //all -valid- bytes sent (excludes invalid compression requests)
    decompress_rcv: u64,  //all -valid- bytes received (excludes invalid compression requests)

    compression_ratio: u8, //ratio of compress_sent / compress_rcv

    encode_sent: u64,
    encode_rcv: u64,
    decode_sent: u64,
    decode_rcv: u64,
}

impl ServerStats {
    fn new() -> ServerStats {
        ServerStats {
            bytes_sent: 0,
            bytes_received: 0,
            compress_sent: 0,
            compress_rcv: 0,
            compression_ratio: 0,
            decompress_sent: 0,
            decompress_rcv: 0,
            encode_sent: 0,
            encode_rcv: 0,
            decode_sent: 0,
            decode_rcv: 0,
        }
    }

    fn get_stats(&self) -> (u64, u64, u8) {
        (self.bytes_sent, self.bytes_received, self.compression_ratio)
    }

    //ensure safe addition-- if overflow will occur, do wrapping add
    fn add_compression_data(&mut self, rcv: u64, sent: u64) {
        if self.compress_sent.checked_add(sent).is_some() {
            self.compress_sent += sent;
        } else {
            self.compress_sent = self.compress_sent.wrapping_add(sent);
        }
        //ensure no overflow -- check add and if not safe then wrapping add
        if self.compress_rcv.checked_add(rcv).is_some() {
            self.compress_rcv += rcv;
        } else {
            self.compress_rcv = self.compress_rcv.wrapping_add(rcv);
        }
        //only update this when we compress data
        //ensure safe division & safe multiplication
        if self.compress_rcv.checked_div(self.compress_sent).is_some() {
            self.compression_ratio =
                ((self.compress_rcv as f64 / self.compress_sent as f64) * 100 as f64) as u8;
        } else {
            //retain the old ratio
        }
    }

    //reset stats
    fn reset_stats(&mut self) {
        self.bytes_sent = 0;
        self.bytes_received = 0;
        self.compress_rcv = 0;
        self.compress_sent = 0;
        self.compression_ratio = 0;
    }

    //safely update the server's inner stats
    fn update_stats(&mut self, update_sent: u64, update_recv: u64) {
        //ensure no overflow -- check add and if not safe then wrapping add
        if self.bytes_sent.checked_add(update_sent).is_some() {
            self.bytes_sent += update_sent;
        } else {
            self.bytes_sent = self.bytes_sent.wrapping_add(update_sent);
        }
        //ensure no overflow -- check add and if not safe then wrapping add
        if self.bytes_received.checked_add(update_recv).is_some() {
            self.bytes_received += update_recv;
        } else {
            self.bytes_received = self.bytes_received.wrapping_add(update_recv);
        }
    }
}

pub struct Server {
    stats: ServerStats,
    error_state: Option<u16>,
    method: Box<dyn Transform>,
}

impl Server {
    pub fn new<B: 'static + Transform>(method: B) -> Result<Server, ()> {
        Ok(Server {
            stats: ServerStats::new(),
            error_state: None,
            method: Box::new(method),
        })
    }

    pub fn listen(&mut self, addr: String, port: String) -> u16 {
        let ip = Ipv4Addr::from_str(&addr);
        if ip.is_err() {
            println!("\nSERVER: Invalid address");
            return super::EINVAL;
        }
        let socket = SocketAddr::new(
            IpAddr::V4(ip.unwrap()),
            u16::from_str_radix(&port, 10).unwrap(),
        );
        let bind = TcpListener::bind(&socket);
        if bind.is_err() {
            println!("\nSERVER: Cannot bind! Exiting");
            return super::UNKNOWN;
        }
        let listener = bind.unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.handle_client(stream);
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::BrokenPipe =>
                {
                    //just move on, someday implement something nicer here
                    continue;
                }
                Err(e) => {
                    //Return to main as this is an unrecoverable state
                    println!("\nSERVER: Encountered error {:?}, exiting", e);
                    return super::UNKNOWN;
                }
            }
        }
        0
    }

    fn process_request(&mut self, request: u16, payload: Option<Vec<u8>>) -> Message {
        match request {
            super::PING => Message::new(0, super::OK, None),
            super::GET => self.generate_stats(),
            super::RESET => self.reset_stats(),
            _ => self.generate_msg(request, payload),
        }
    }

    //get stats does not include the bytes that are about to be generated & sent in
    //response to the message
    fn get_stats(&self) -> (u64, u64, u8) {
        return self.stats.get_stats();
    }

    //reset all internal stats to 0
    fn reset_stats(&mut self) -> Message {
        self.stats.reset_stats();
        Message::new(0, super::OK, None)
    }

    fn update_stats(&mut self, update_sent: u64, update_recv: u64) {
        self.stats.update_stats(update_sent, update_recv)
    }

    fn add_compression_data(&mut self, update_sent: u64, update_recv: u64) {
        self.stats.add_compression_data(update_sent, update_recv)
    }

    ///TODO need to fix this-- three copies of the same damn vec is bad
    fn transform_payload(&mut self, direction: bool, payload: Vec<u8>) -> Result<Vec<u8>, ()> {
        let mut cp: Vec<u8> = Vec::new();
        payload.clone_into(&mut cp);
        self.method.transform(direction, &mut cp);
        //Err(())
        Ok(cp.clone())
    }

    //The compress function will return Err(()) for requests for compressing non-lowercase
    // alphabetic vals; in this case generate an error message
    fn generate_compression(&mut self, payload: Vec<u8>) -> Message {
        let old_len = payload.len();
        let compressed = self.transform_payload(true, payload);
        if compressed.is_err() {
            //internal error occurred
            return self.generate_msg(super::OTHER_ERROR, None);
        }
        let msg = compressed.unwrap();
        self.add_compression_data(msg.len() as u64, old_len as u64);
        Message::new(msg.len() as u16, 0, Some(msg.clone()))
    }

    //The compress function will return Err(()) for requests for compressing non-lowercase
    // alphabetic vals; in this case generate an error message
    fn generate_decompression(&mut self, payload: Vec<u8>) -> Message {
        let old_len = payload.len();
        let decompressed = self.transform_payload(false, payload);
        if decompressed.is_err() {
            //internal error occurred
            return self.generate_msg(super::OTHER_ERROR, None);
        }
        let msg = decompressed.unwrap();
        Message::new(msg.len() as u16, 0, Some(msg.clone()))
    }

    //Generate a statistics message by deserializing the two u64's into
    //big endiann bytes, pushing all bytes including the u8 ratio byte
    //into a vector of 9 u8's. This then gets populated as a message
    //payload
    fn generate_stats(&mut self) -> Message {
        //}, usize) {
        let (sent, rcv, ratio) = self.get_stats();
        let mut sent_bytes = sent.to_ne_bytes().to_vec();
        let mut rcv_bytes = rcv.to_ne_bytes().to_vec();
        let mut payload: Vec<u8> = Vec::new();
        payload.append(&mut sent_bytes);
        payload.append(&mut rcv_bytes);
        payload.push(ratio);
        let len = payload.len();
        Message::new(len as u16, super::OK, Some(payload))
    }

    /// Return the requested generated message
    /// This function will only ever be called to generate an error message
    /// Or a message with a payload, so we know that if payload == None then
    /// we are creating an error message
    fn generate_msg(&mut self, mtype: u16, payload: Option<Vec<u8>>) -> Message {
        if let Some(payload) = payload {
            match mtype {
                super::COMPRESS => self.generate_compression(payload),
                super::DECOMPRESS => self.generate_decompression(payload),
                super::ENCODE => match self.transform_payload(true, payload) {
                    Ok((t)) => Message::new(t.len() as u16, super::OK, Some(t)),
                    Err(e) => Message::new(0, super::EINVAL, None),
                },
                super::DECODE => match self.transform_payload(false, payload) {
                    Ok(t) => Message::new(t.len() as u16, super::OK, Some(t)),
                    Err(e) => Message::new(0, super::EINVAL, None),
                },
                _ => Message::new(0, super::ENOSUP, None),
            }
        } else {
            //Message::new(0, super::EINVAL, None)
            Message::new(0, mtype, None)
        }
    }

    fn get_request(&mut self, mut stream: &TcpStream) -> Result<Message, Message> {
        if let Some(state) = self.error_state {
            return Err(self.generate_msg(state, None));
        }
        let mut bytes: [u8; std::mem::size_of::<MessageHeader>()] =
            [0; std::mem::size_of::<MessageHeader>()];
        let mut full_bytes = [0u8; MAX_MSG_LEN];

        let mut payload = None;
        let mut bytes_rcv = std::mem::size_of::<MessageHeader>();

        let res = match stream.peek(&mut bytes) {
            Ok(t) => bincode::deserialize(&bytes),
            Err(e) => {
                println!("\nSERVER: Failure to read stream-- incomplete message header found?");
                return Err(self.generate_msg(super::EINVAL, None));
            }
        };

        let header: MessageHeader;

        let (len, request) = match res {
            Ok(t) => {
                header = res.unwrap();
                match header.is_valid() {
                    true => header.get(),
                    false => {
                        println!("\nSERVER: Failure to parse correct message header");
                        return Err(self.generate_msg(super::EINVAL, None));
                    }
                }
            }
            Err(e) => {
                println!("\nSERVER: Failure to deserialize sent bytes");
                return Err(self.generate_msg(super::EINVAL, None));
            }
        };

        if len as usize > MAX_PAYLOAD_LEN {
            println!("\nSERVER: Invalid length, returning error");
            return Err(self.generate_msg(super::EINVAL, None));
        }

        let full = match stream.read(&mut full_bytes) {
            Ok(t) => bincode::deserialize(&bytes),
            Err(e) => {
                println!("\nSERVER: Failure to read stream-- drop connection");
                return Err(self.generate_msg(super::INTERNAL_ERROR, None));
            }
        };

        let deserialized_msg: Message;
        match full {
            Ok(t) => {
                deserialized_msg = t; //full.unwrap();
                if let Some(msg_payload) = deserialized_msg.get_payload() {
                    if len as usize == msg_payload.len() {
                        payload = Some(msg_payload);
                        bytes_rcv += len as usize;
                        self.update_stats(0, bytes_rcv as u64);
                        //TODO determine if this is needed? Can I just use the
                        // deserialized message, or will that mess with ownership?
                        Ok(Message::new(len, request, payload))
                    } else {
                        println!("\nSERVER: length is not correctly reported, returning error");
                        //TODO determine is errorneous bytes e.g.
                        // non-transformed bytes should be stored
                        bytes_rcv += msg_payload.len();
                        self.update_stats(0, bytes_rcv as u64);
                        Err(self.generate_msg(super::EINVAL, None))
                    }
                } else {
                    Err(self.generate_msg(super::EINVAL, None))
                }
            }
            Err(e) => {
                println!(
                    "\nSERVER: Unable to deserialize message request, Error {:?}",
                    e
                );
                //invalid message structure parsed
                Err(self.generate_msg(super::EINVAL, None))
            }
        }
    }

    fn process(&mut self, mut msg: Message) -> Message {
        match msg.get_header().get() {
            //request {
            (_, super::PING) => Message::new(0, super::OK, None),
            (_, super::GET) => self.generate_stats(),
            (_, super::RESET) => self.reset_stats(),
            (_, request) => self.generate_msg(request, msg.get_payload()),
        }
    }

    //This function handles reading bytes in from a TCP stream, serializing,
    //parsing a request, then processing an appropriate response and writing
    //back to the stream. The mutability of a stream object prevents this
    //from being more easily modularized, need to look into this further
    fn handle_client(&mut self, mut stream: TcpStream) {
        let mut response = match self.get_request(&stream) {
            Ok(r) => self.process(r),
            Err(r) => r,
        };

        match bincode::serialize(&response) {
            Ok(mut t) => {
                match stream.write(&mut t) {
                    Ok(t) => {
                        let send_len = t - std::mem::size_of::<MessageHeader>();
                        //Note: if reset request was made, this will add 8 bytes to each
                        self.update_stats(send_len as u64, 0); //bytes_rcv as u64);
                                                               // Ok(())
                    }
                    Err(e) => {
                        println!("\nSERVER: Failure to write stream");
                        //TODO Set internal error state
                        // Err(e)
                    }
                }
            }
            Err(e) => {
                //TODO convert from Box<bincode::ErrorKInd> to std::io::ErrorKind
                //set internal error state!
                // return Err(std::io::Error::new(std::io::ErrorKind::Other, "Bincode error"))
            }
        }
    }
}
