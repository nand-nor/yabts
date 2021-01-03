extern crate argparse;
extern crate bincode;
extern crate serde;

pub mod client;
pub mod helpers;
pub mod message;
pub mod server;
pub mod stats;
pub mod transform;

///end users may want to modify for different MTUs
pub const MAX_MSG_LEN: usize = 4000;

///header size is 8 bytes
pub const MAX_PAYLOAD_LEN: usize = MAX_MSG_LEN - 8;

///deadbeef for valid header magic
pub const MAGIC: u32 = 0xDEADBEEF;

/// Valid Statuses w.r.t incoming requests / outgoing responses
pub const OK: u16 = 0; //OK request/response
pub const UNKNOWN: u16 = 1; //unknown internal error
pub const EINVAL: u16 = 2; //invalid request e.g. message too large;
pub const ENOSUP: u16 = 3; //Unsupported request type
                           // RESERVE 4-32
pub const INTERNAL_ERROR: u16 = 33; //some internal error state
pub const OTHER_ERROR: u16 = 34;

/// Valid Client Requests
#[derive(Copy, Clone)]
pub enum REQUEST {
    NONE = 0,
    PING = 1,
    GET,
    RESET,
    COMPRESS,
    DECOMPRESS,
    ENCODE,
    DECODE,
}


impl std::str::FromStr for REQUEST {
    type Err = std::io::Error;
    fn from_str(request: &str)->Result<REQUEST,std::io::Error>{
        match request {
            "ping" => Ok(REQUEST::PING),
            "get" => Ok(REQUEST::GET),
            "reset" => Ok(REQUEST::RESET),
            "compress" => Ok(REQUEST::COMPRESS),
            "decompress" => Ok(REQUEST::DECOMPRESS),
            "encode" => Ok(REQUEST::ENCODE),
            "decode" => Ok(REQUEST::DECODE),
            _ => return Err(std::io::Error::new(std::io::ErrorKind::Other,
                                                "Invalid request provided"))
        }
    }
}

use std::convert::TryFrom;

impl TryFrom<u16> for REQUEST {
    type Error = ();
    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == REQUEST::NONE as u16 => Ok(REQUEST::NONE),
            x if x == REQUEST::PING as u16 => Ok(REQUEST::PING),
            x if x == REQUEST::GET as u16 => Ok(REQUEST::GET),
            x if x == REQUEST::RESET as u16 => Ok(REQUEST::RESET),
            x if x == REQUEST::COMPRESS as u16 => Ok(REQUEST::COMPRESS),
            x if x == REQUEST::DECOMPRESS as u16 => Ok(REQUEST::DECOMPRESS),
            x if x == REQUEST::ENCODE as u16 => Ok(REQUEST::ENCODE),
            x if x == REQUEST::DECODE as u16 => Ok(REQUEST::DECODE),
            _ => Err(()),
        }
    }
}


pub enum RUNMODE {
    CLIENT(String, String, Option<REQUEST>, Option<String>),
    SERVER(String, String),
}
