use bincode;
use std::io::prelude::*;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;

use crate::message::*;
use crate::stats::*;
use crate::transform::*;

use super::MAX_MSG_LEN;
use super::MAX_PAYLOAD_LEN;

pub struct Server {
    addr: Ipv4Addr,
    port: u16,
    stats: ServerStats,
    error_state: Option<u16>,
    method: Box<dyn Transform>,
}

impl Server {
    pub fn new<B: 'static + Transform>(
        method: B,
        addr: String,
        _port: String,
    ) -> Result<Server, u16> {
        let ip = Ipv4Addr::from_str(&addr);
        let port = u16::from_str_radix(&_port, 10);

        if ip.is_err() {
            println!("\nSERVER: Invalid server address provided");
            return Err(super::EINVAL);
        }

        if port.is_err() {
            println!("\nSERVER: Invalid server port provided");
            return Err(super::EINVAL);
        }

        Ok(Server {
            addr: ip.unwrap(),
            port: port.unwrap(),
            stats: ServerStats::new(),
            error_state: None,
            method: Box::new(method),
        })
    }

    pub fn listen(&mut self) -> u16 {
        let socket = SocketAddr::new(
            IpAddr::V4(self.addr), //ip.unwrap()),
            self.port,             //u16::from_str_radix(&port, 10).unwrap(),
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

    //get stats does not include the bytes that are about to be generated & sent in
    //response to the message
    fn get_stats(&self) -> (u64, u64, u8, u8, u8, u8) {
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

    ///TODO need to fix this-- three copies of the same damn vec is bad
    fn transform_payload(&mut self, direction: bool, mut payload: Vec<u8>) -> Result<Vec<u8>, ()> {
        let cp: Vec<u8>; // = Vec::new();
        match self.method.transform(direction, &mut payload) {
            Ok(()) => {
                cp = payload.clone();
                Ok(cp.clone())
            }
            Err(t) => {
                //TODO set internal error state?
                println!("SERVER: Encountered error transforming bytes: {:?}", t);
                Err(())
            }
        }
    }

    fn generate_encode(&mut self, payload: Vec<u8>) -> Message {
        let old_len = payload.len();
        let encode = self.transform_payload(true, payload);
        if encode.is_err() {
            //internal error occurred
            return self.generate_msg(super::OTHER_ERROR, None);
        }
        let msg = encode.unwrap();
        self.stats.add_encode_data(msg.len() as u64, old_len as u64);
        Message::new(msg.len() as u16, 0, Some(msg.clone()))
    }

    fn generate_decode(&mut self, payload: Vec<u8>) -> Message {
        let old_len = payload.len();
        let decode = self.transform_payload(false, payload);
        if decode.is_err() {
            //internal error occurred
            return self.generate_msg(super::OTHER_ERROR, None);
        }
        let msg = decode.unwrap();
        self.stats.add_decode_data(msg.len() as u64, old_len as u64);
        Message::new(msg.len() as u16, 0, Some(msg.clone()))
    }

    fn generate_compression(&mut self, payload: Vec<u8>) -> Message {
        let old_len = payload.len();
        let compressed = self.transform_payload(true, payload);
        if compressed.is_err() {
            //internal error occurred
            return self.generate_msg(super::OTHER_ERROR, None);
        }
        let msg = compressed.unwrap();
        self.stats
            .add_compression_data(msg.len() as u64, old_len as u64);
        Message::new(msg.len() as u16, 0, Some(msg.clone()))
    }

    fn generate_decompression(&mut self, payload: Vec<u8>) -> Message {
        let old_len = payload.len();
        let decompressed = self.transform_payload(false, payload);
        if decompressed.is_err() {
            //internal error occurred
            return self.generate_msg(super::OTHER_ERROR, None);
        }
        let msg = decompressed.unwrap();
        self.stats
            .add_decompression_data(msg.len() as u64, old_len as u64);
        Message::new(msg.len() as u16, 0, Some(msg.clone()))
    }

    //Generate a statistics message by deserializing the two u64's into
    //big endiann bytes, pushing all bytes including the u8 ratio byte
    //into a vector of 20 u8's. This then gets populated as a message
    //payload
    fn generate_stats(&mut self) -> Message {
        //}, usize) {
        let (sent, rcv, cratio, dratio, eratio, deratio) = self.get_stats();
        let mut sent_bytes = sent.to_ne_bytes().to_vec();
        let mut rcv_bytes = rcv.to_ne_bytes().to_vec();
        let mut payload: Vec<u8> = Vec::new();
        payload.append(&mut sent_bytes);
        payload.append(&mut rcv_bytes);
        payload.push(cratio);
        payload.push(dratio);
        payload.push(eratio);
        payload.push(deratio);

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
                x if x == super::REQUEST::COMPRESS as u16 => self.generate_compression(payload),
                x if x == super::REQUEST::DECOMPRESS as u16 => self.generate_decompression(payload),
                x if x == super::REQUEST::ENCODE as u16 => self.generate_encode(payload),
                x if x == super::REQUEST::DECODE as u16 => self.generate_decode(payload),
                _ => Message::new(0, super::ENOSUP, None),
            }
        } else {
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

        let payload; // = None;
        let mut bytes_rcv = std::mem::size_of::<MessageHeader>();

        let res = match stream.peek(&mut bytes) {
            Ok(_t) => bincode::deserialize(&bytes),
            Err(e) => {
                println!("\nSERVER: Failure to read stream-- error: {:?}", e);
                return Err(self.generate_msg(super::EINVAL, None));
            }
        };

        let header: MessageHeader;

        let (len, request) = match res {
            Ok(_t) => {
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
                println!("\nSERVER: Failure to deserialize sent bytes: {:?}", e);
                return Err(self.generate_msg(super::EINVAL, None));
            }
        };

        if len as usize > MAX_PAYLOAD_LEN {
            println!("\nSERVER: Invalid length, returning error");
            return Err(self.generate_msg(super::EINVAL, None));
        }

        let full: Result<Message, bincode::Error> = match stream.read(&mut full_bytes) {
            Ok(_t) => bincode::deserialize(&full_bytes),
            Err(e) => {
                println!("\nSERVER: Failure to read stream-- drop connection {:?}", e);
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
                        self.stats.update_stats(0, bytes_rcv as u64);
                        //TODO determine if this is needed? Can I just use the
                        // deserialized message, or will that mess with ownership?
                        Ok(Message::new(len, request, payload))
                    } else {
                        println!("\nSERVER: length is not correctly reported, returning error");
                        //TODO determine is errorneous bytes e.g.
                        // non-transformed bytes should be stored
                        bytes_rcv += msg_payload.len();
                        self.stats.update_stats(0, bytes_rcv as u64);
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

    fn process(&mut self, msg: Message) -> Message {
        match msg.get_header().code() {
            x if x == super::REQUEST::PING as u16 => Message::new(0, super::OK, None),
            x if x == super::REQUEST::GET as u16 => self.generate_stats(),
            x if x == super::REQUEST::RESET as u16 => self.reset_stats(),
            request => self.generate_msg(request, msg.get_payload()),
        }
    }

    //This function handles reading bytes in from a TCP stream, serializing,
    //parsing a request, then processing an appropriate response and writing
    //back to the stream. The mutability of a stream object prevents this
    //from being more easily modularized, need to look into this further
    fn handle_client(&mut self, mut stream: TcpStream) {
        let response = match self.get_request(&stream) {
            Ok(r) => self.process(r),
            Err(r) => r,
        };

        match bincode::serialize(&response) {
            Ok(mut t) => {
                match stream.write(&mut t) {
                    Ok(t) => {
                        let send_len = t - std::mem::size_of::<MessageHeader>();
                        //Note: if reset request was made, this will add 8 bytes to each
                        self.update_stats(send_len as u64, 0);
                    }
                    Err(e) => {
                        println!("\nSERVER: Failure to write stream: {:?}", e);
                        //TODO Set internal error state
                        // Err(e)
                    }
                }
            }
            Err(e) => {
                println!("\nSERVER: Failure to serialize response bytes: {:?}", e);

                //TODO convert from Box<bincode::ErrorKInd> to std::io::ErrorKind
                //set internal error state!
                // return Err(std::io::Error::new(std::io::ErrorKind::Other, "Bincode error"))
            }
        }
    }
}
