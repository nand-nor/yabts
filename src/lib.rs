#![feature(toowned_clone_into)]
extern crate bincode;
extern crate serde;

pub mod client;
pub mod message;
pub mod server;
pub mod transform;

pub const MAX_MSG_LEN: usize = 4000;
///end users may want to modify for different MTUs
pub const MAX_PAYLOAD_LEN: usize = MAX_MSG_LEN - 8;
///header size is 8 bytes
pub const MAGIC: u32 = 0xDEADBEEF;
///deadbeef

/*Valid Statuses*/
pub const OK: u16 = 0; //OK request/response
pub const UNKNOWN: u16 = 1; //unknown internal error
pub const EINVAL: u16 = 2; //invalid request e.g. message too large;
pub const ENOSUP: u16 = 3; //Unsupported request type
                           // RESERVE is 4-32
pub const INTERNAL_ERROR: u16 = 33; //some internal error state
pub const OTHER_ERROR: u16 = 34;

/// Valid Requests
/// Requests are in the form of:
/// ./yabts [mode] [request] [type] [filename]
/// where mode is server or client mode
/// request is one of the below values
/// type is conditional upon request -- only compress/decompress or encode/decode
/// and only if multiple methods are compiled in user binary
pub const PING: u16 = 1;
pub const GET: u16 = 2;
pub const RESET: u16 = 3;
pub const COMPRESS: u16 = 4;
pub const DECOMPRESS: u16 = 5;
pub const ENCODE: u16 = 6;
pub const DECODE: u16 = 7;

//pub const STORE: u16 = 8;       ///Store a payload -- to be implemented
//pub const RETRIEVE: u16 = 9;    ///Retrieve a payload -- to be implemented

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod test {
    //edge cases: no compression
    //all compression
    //maximum possible compression
    //fail cases: invalid char input, invalid lengths, etc.

    use super::*;
    use std::str;

    #[test]
    fn check_compression() {
        let all_compress = compress::compress("aaaaabbbb");
        let no_compress = compress::compress("abcdefg");

        if no_compress.is_ok() || all_compress.is_ok() {
            let case1 = all_compress.unwrap();
            let case2 = no_compress.unwrap();

            let case1_string = unsafe {
                let res = str::from_utf8(case1.as_slice());
                if res.is_err() {
                    ""
                } else {
                    res.unwrap()
                }
            };

            let case2_string = unsafe {
                let res = str::from_utf8(case2.as_slice());
                if res.is_err() {
                    ""
                } else {
                    res.unwrap()
                }
            };

            println!("Compressed: {:?} and {:?}", case1_string, case2_string);
            assert_eq!(case1_string, "5a4b");
            assert_eq!(case2_string, "abcdefg");
        }
    }

}
