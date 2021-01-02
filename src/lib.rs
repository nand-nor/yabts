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

/// Valid Requests for Debugging
/// TODO need to add support for an admin mode
pub const PING: u16 = 1;
pub const GET: u16 = 2;
pub const RESET: u16 = 3;

/// Valid Client requests (for non debugging)
pub const COMPRESS: u16 = 4;
pub const DECOMPRESS: u16 = 5;
pub const ENCODE: u16 = 6;
pub const DECODE: u16 = 7;

//pub const STORE: u16 = 8;       ///Store a payload -- to be implemented
//pub const RETRIEVE: u16 = 9;    ///Retrieve a payload -- to be implemented

pub enum RUNMODE {
    CLIENT(String, String, String, Option<String>),
    SERVER(String, String),
}
