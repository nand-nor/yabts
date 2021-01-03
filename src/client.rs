use bincode;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::str;
use std::str::FromStr;

use crate::{message, MAX_MSG_LEN, MAX_PAYLOAD_LEN};
use std::process;

pub fn parse_payload(file: Option<String>) -> Option<Vec<u8>> {
    if let Some(file) = file {
        let mut fp = match File::open(file) {
            Err(why) => {
                println!("\nCLIENT: Could not open file, reason {:?}, exiting", why);
                return None;
            }
            Ok(fp) => fp,
        };
        let mut file_payload: Vec<u8> = vec![];
        if fp.read_to_end(&mut file_payload).is_err() {
            println!("\nCLIENT:  process unable to read file, exiting");
            return None;
        }
        println!("Got the payload: {:?}", file_payload);
        Some(file_payload)
    } else {
        None
    }
}

pub fn run_client(addr: String, _port: String, request: super::REQUEST, file: Option<String>) -> u16 {
    let send_payload: Option<Vec<u8>> = match request {
        super::REQUEST::COMPRESS => parse_payload(file),
        super::REQUEST::DECOMPRESS => parse_payload(file),
        super::REQUEST::ENCODE => parse_payload(file),
        super::REQUEST::DECODE => parse_payload(file),
        super::REQUEST::PING => None,
        super::REQUEST::GET => None,
        super::REQUEST::RESET => None,
        _ => {
            return super::EINVAL;
        }
    };

    let ip = Ipv4Addr::from_str(&addr);
    let port = u16::from_str_radix(&_port, 10);

    let socket = match ip {
        Ok(a) => match port {
            Ok(p) => SocketAddr::new(IpAddr::V4(a), p),
            Err(e) => {
                println!("\nSERVER: Invalid server port provided");
                return super::EINVAL;
            }
        },
        Err(e) => {
            println!("\nSERVER: Invalid server address provided");
            return super::EINVAL;
        }
    };

    let stream = TcpStream::connect(socket);
    if stream.is_ok() {
        let mut stream = stream.unwrap();
        let request_msg = base_request(request as u16, send_payload);
        let send_bytes = bincode::serialize(&request_msg).unwrap();
        if stream.write(send_bytes.as_slice()).is_err() {
            println!("\nCLIENT: Failure to write to TCP stream");
            return super::UNKNOWN as _;
        }
        let mut read_it = [0u8; MAX_MSG_LEN];
        if stream.read(&mut read_it).is_err() {
            println!("\nCLIENT: Failure to read TCP stream");
            return super::UNKNOWN as _;
        }
        process_response(request, &read_it)
    } else {
        println!("\nCLIENT:  unable to connect to TCP server at 127.0.0.1:4000, exiting");
        return super::UNKNOWN as _;
    }
}

pub fn process_response(request: super::REQUEST, read_it: &[u8]) -> u16 {
    let msg: message::Message;

    match bincode::deserialize(&read_it) {
        Ok(message) => {
            msg = message;
            println!(
                "\nCLIENT: received response with header {:?} and payload {:?}",
                msg.get_header(),
                msg.get_payload()
            );
            let (_, status) = msg.get_header().get();

            if status != super::OK {
                println!("\nCLIENT: received error response {:?}", status);
                return status as _;
            }

            match request {
                super::REQUEST::PING => {
                    println!("\nCLIENT: Ping: return status {:?}", status);
                }
                super::REQUEST::GET => {
                    let res = serialize_to_stats(msg.get_payload().unwrap());

                    if res.is_err() {
                        println!("\nCLIENT: unable to serialize received stats data");
                        return super::UNKNOWN;
                    } else {
                        let (returned_sent, returned_rcv, returned_ratio) = res.unwrap();
                        println!(
                        "\nCLIENT: Stats: return status {:?}, server returned sent bytes: {:?}, \
                         rcv'd bytes {:?}, ratio: {:?}",
                        status, returned_sent, returned_rcv, returned_ratio
                    );
                    }
                }
                super::REQUEST::RESET => {
                    println!("\nCLIENT: Reset: return status {:?}", status);
                }
                _ => {
                    match str::from_utf8(msg.get_payload().unwrap().as_slice()) {
                        Ok(p) => {
                            println!("\nCLIENT: transformed data:\n {:?}", p);
                        }
                        Err(e) => {
                            println!(
                                "\nCLIENT: unable to decode returned payload as utf: {:?}",
                                e
                            );
                            return super::UNKNOWN;
                        }
                    };
                }
            };
        }
        Err(e) => {
            println!("\nCLIENT: Unable to deserialize read-in bytes");
            return super::OTHER_ERROR;
        }
    };

    return super::OK;
}

pub fn serialize_to_stats(payload: Vec<u8>) -> Result<(u64, u64, u8), ()> {
    if payload.len() != 17 {
        println!(
            "\nCLIENT: Payload is longer than it should be! Length is {:?}",
            payload.len()
        );
        return Err(());
    }

    //copy_from_slice() and clone_from_slice both fail to copy over vec's bytes?
    let mut sent_bytes: [u8; 8] = [0; 8];
    let mut rcv_bytes: [u8; 8] = [0; 8];
    for i in 0..8 {
        sent_bytes[i] = payload[i];
        rcv_bytes[i] = payload[i + 8];
    }

    let sent = u64::from_ne_bytes(sent_bytes);
    let rcv = u64::from_ne_bytes(rcv_bytes);
    let ratio = payload[16];

    Ok((sent, rcv, ratio))
}

fn base_request(request: u16, payload: Option<Vec<u8>>) -> message::Message {
    match request {
        x if x == super::REQUEST::PING as u16 => {
            println!("\nCLIENT:  requesting ping...");
            message::Message::new(0, request, None)
        }
        x if x == super::REQUEST::GET as u16 => {
            println!("\nCLIENT:  requesting server stats...");
            message::Message::new(0, request, None)
        }
        x if x == super::REQUEST::RESET as u16 => {
            println!("\nCLIENT: requesting server stat reset...");
            message::Message::new(0, request, None)
        }
        x if x == super::REQUEST::COMPRESS as u16 => {
            println!("\nCLIENT: requesting compression...");
            if let Some(payload) = payload {
                let length = payload.len();
                message::Message::new(length as u16, request, Some(payload))
            } else {
                println!("\nCLIENT: invalid request");
                return message::Message::default();
            }
        }
        x if x == super::REQUEST::DECOMPRESS as u16 => {
            println!("\nCLIENT: requesting compression...");
            if let Some(payload) = payload {
                let length = payload.len();
                message::Message::new(length as u16, request, Some(payload))
            } else {
                println!("\nCLIENT: invalid request");
                return message::Message::default();
            }
        }
        x if x == super::REQUEST::ENCODE as u16 => {
            println!("\nCLIENT: requesting encoding...");
            if let Some(payload) = payload {
                let length = payload.len();
                message::Message::new(length as u16, request, Some(payload))
            } else {
                println!("\nCLIENT: invalid request");
                return message::Message::default();
            }
        }
        x if x == super::REQUEST::DECODE as u16 => {
            println!("\nCLIENT: requesting decoding...");
            if let Some(payload) = payload {
                let length = payload.len();
                message::Message::new(length as u16, request, Some(payload))
            } else {
                println!("\nCLIENT: invalid request");
                return message::Message::default();
            }
        }
        _ => {
            println!("\nCLIENT: invalid request");
            return message::Message::default();
        }
    }
}
