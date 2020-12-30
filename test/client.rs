use std::net::TcpStream;
use std::str;
use bincode;
use std::fs::File;
use std::io::{Read, Write};

use crate::{message, MAX_MSG_LEN};


pub fn run_client<'a>(request: u16, file: Option<&'a str>)->u16{
    let send_payload: Option<Vec<u8>>;
        if request == super::COMPRESS  {
            if let Some(file) = file {
                let mut fp = match File::open(file) {
                    Err(why) => {
                        println!("\nCLIENT: Could not open input target input, reason {:?}, exiting", why);
                        return super::UNKNOWN
                    }
                    Ok(fp) => fp,
                };
                let mut file_payload: Vec<u8> = vec![];
                if fp.read_to_end(&mut file_payload).is_err() {
                    println!("\nCLIENT:  process unable to read file, exiting");
                    return super::UNKNOWN as _
                }
                send_payload = Some(file_payload)
            } else {
                println!("\nCLIENT:  failed to specify compression payload target file, exiting");
                return super::UNKNOWN as _
            }
        } else {
            send_payload = None;
        }

    let stream = TcpStream::connect("127.0.0.1:4000");
    if stream.is_ok() {
        let mut stream = stream.unwrap();
        let request_msg = base_request(request as u16, send_payload);
        let send_bytes = bincode::serialize(&request_msg).unwrap();
        if stream.write(send_bytes.as_slice()).is_err(){
            println!("\nCLIENT: Failure to write to TCP stream");
            return super::UNKNOWN as _
        }
        let mut read_it = [0u8; MAX_MSG_LEN];
        if stream.read(&mut read_it).is_err(){
            println!("\nCLIENT: Failure to read TCP stream");
            return super::UNKNOWN as _
        }
        let message = bincode::deserialize(&read_it);
        let msg: message::Message;// = message::Message::default();
        if message.is_err() {
            println!("\nCLIENT: Unable to deserialize read-in bytes");
        } else {
            msg = message.unwrap();
            println!("\nCLIENT: received response with header {:?} and payload {:?}", msg.get_header(), msg.get_payload());
            let (_, status) = msg.get_header().get();

            if status != 0{
                println!("\nCLIENT: received error response {:?}", status);
                return status as _
            }

            match request as u16 {
                super::PING => {
                    println!("\nCLIENT: Ping: return status {:?}", status);
                },
                super::GET => {
                    let res = serialize_to_stats(msg.get_payload().clone());

                    if res.is_err() {
                        println!("\nCLIENT: unable to serialize received stats data");
                        return super::UNKNOWN
                    } else {
                        let (returned_sent, returned_rcv, returned_ratio) = res.unwrap();
                        println!("\nCLIENT: Stats: return status {:?}, server returned sent bytes: {:?}, \
                        rcv'd bytes {:?}, ratio: {:?}", status, returned_sent, returned_rcv, returned_ratio);
                    }

                },
                super::RESET => {
                    println!("\nCLIENT: Reset: return status {:?}", status);
                },
                super::COMPRESS => {
                    let temp_load = msg.get_payload();
                    let compressed =  str::from_utf8(temp_load.as_slice());
                    if compressed.is_err(){

                        println!("\nCLIENT: unable to serialize received stats data");
                        return super::UNKNOWN
                    } else {
                        let compressed: &str = compressed.unwrap().clone();
                        println!("\nCLIENT: compressed data:\n {:?}", compressed);
                    }

                },
                _ => {return super::EINVAL}

            };


        }
    } else {
        println!("\nCLIENT:  unable to connect to TCP server at 127.0.0.1:4000, exiting");
        return super::UNKNOWN as _
    }
    return super::OK
}



fn serialize_to_stats(payload: Vec<u8>) -> Result<(u32, u32, u8), ()> {
    if payload.len() != 9 {
        println!(
            "\nCLIENT: Payload is longer than it should be! Length is {:?}",
            payload.len()
        );
        return Err(());
    }

    //copy_from_slice() and clone_from_slice both fail to copy over vec's bytes?
    let mut sent_bytes: [u8; 4] = [0; 4];
    let mut rcv_bytes: [u8; 4] = [0; 4];
    for i in 0..4 {
        sent_bytes[i] = payload[i];
        rcv_bytes[i] = payload[i + 4];
    }

    let sent = u32::from_ne_bytes(sent_bytes);
    let rcv = u32::from_ne_bytes(rcv_bytes);
    let ratio = payload[8];

    Ok((sent, rcv, ratio))
}



fn base_request(request: u16, payload: Option<Vec<u8>>)->message::Message {
    match request {
        super::PING  => {
            println!("\nCLIENT:  requesting ping...");
            message::Message::new(0, request)
        }
        super::GET => {
            println!("\nCLIENT:  requesting server stats...");
            message::Message::new(0, request)
        }
        super::RESET => {
            println!("\nCLIENT: requesting server stat reset...");
            message::Message::new(0, request)
        }
        super::COMPRESS => {
            println!("\nCLIENT: requesting compression...");
            if let Some(payload) = payload {
                let length = payload.len();
                message::Message::new_with_payload(length as u16, request, payload)
            } else {
                println!("\nCLIENT: invalid request");
                return message::Message::default()
            }
        }
        _ => {
            println!("\nCLIENT: invalid request");
            return message::Message::default()
        }
    }

}



#[cfg(feature = "client")]
pub fn run_client_tests() {
    let simple = "aaabbbcccddd";
    let no_compress = "abcdefghijklmnop";
    let all_compress = "xxxxxxxxxxxxxxxxxxxxxxxxxx";
    //assert these pass
    let test_cases = [super::COMPRESS, super::COMPRESS, super::COMPRESS, super::PING, super::GET];
    //assert these fail
    let more_tests = [super::COMPRESS, super::COMPRESS, super::COMPRESS,  5, 99, super::RESET];
    let mut results: [(u32, u32, message::MessageHeader); 8] =
        [(0, 0, message::MessageHeader::default()); 8];
    let strings = [simple, no_compress, all_compress];
    let bad_strings: [&str;3] = ["ABCDEFG", "0xdeadbeef", "jjjjaaazzz!"];
    let mut compressed: Vec<Vec<u8>> = Vec::new();
    let mut stats: Vec<Vec<u8>> = Vec::new();
    let mut count_sent: u32 = 0;
    let mut count_rcv: u32 = 0;

    /* ~~~ check that all valid requests are handled appropriately (first set of tests)~~~ */
    for i in 0..test_cases.len() {
        let stream = TcpStream::connect("127.0.0.1:4000");

        if stream.is_ok() {
            let mut stream = stream.unwrap();
            let mut length: usize = 0;

            let request_msg: message::Message;
            if test_cases[i] == super::COMPRESS {
                let payload = strings[i].as_bytes();
                length = payload.len();
                request_msg =
                    message::Message::new_with_payload(length as u16, 4, payload.to_vec());
            } else {
                request_msg = message::Message::new(0, test_cases[i]);
            }
            let send_bytes = bincode::serialize(&request_msg).unwrap();

            stream.write(send_bytes.as_slice());
            println!("\nClient: wrote {:?} (serialized)", send_bytes);

            let mut read_it = [0u8; super::MAX_MSG_LEN];
            stream.read(&mut read_it);

            let message = bincode::deserialize(&read_it);
            let mut msg: message::Message = message::Message::default();
            if message.is_err() {
                println!("\nClient: Unable to deserialize read-in bytes");
            } else {
                msg = message.unwrap();
                println!(
                    "\nClient: received response with header {:?} and payload {:?}",
                    msg.get_header(),
                    msg.get_payload()
                );

                if test_cases[i] == super::COMPRESS {
                    // let payload = msg.get_payload();

                    compressed.push(msg.get_payload()); //returned.clone();

                }

                if test_cases[i] == super::GET {
                    stats.push(msg.get_payload());
                } else {
                    println!("Incrementing count sent and rcv!! Sent was {:?}, rcv was {:?}", count_sent, count_rcv);
                    count_sent += length as u32 + std::mem::size_of::<message::MessageHeader>() as u32;
                    count_rcv += msg.get_payload().len() as u32
                        + std::mem::size_of::<message::MessageHeader>() as u32;
                    println!(" Sent is now {:?}, rcv is now {:?}", count_sent, count_rcv);
                }

                results[i] = (
                    length as u32 + std::mem::size_of::<message::MessageHeader>() as u32,
                    msg.get_payload().len() as u32
                        + std::mem::size_of::<message::MessageHeader>() as u32,
                    msg.get_header(),
                );
            }
        } else {
            println!("\nClient: unable to connect to TCP server");
            process::exit(1);
        }
    }

    let mut flag = true;

    /*~*~*~*~Check results~*~*~*~*/

    let returned_simple = str::from_utf8(compressed[0].as_slice()).unwrap();
    if returned_simple != "3a3b3c3d" {
        println!(
            "\nClient: Message received is not compressed as expected! \
             Input {:?} output{:?}",
            strings[0], returned_simple
        );
        flag = false;
    }
    let returned_none = str::from_utf8(compressed[1].as_slice()).unwrap();

    if returned_none != strings[1] {
        println!(
            "\nClient: Message received is not compressed as expected! \
             Input {:?} output {:?}",
            strings[1], returned_none
        );
        flag = false;
    }
    let returned_all = str::from_utf8(compressed[2].as_slice()).unwrap();

    if returned_all != "26x" {
        println!(
            "\nClient: Message received is not compressed as expected! \
             Input {:?} output{:?}",
            strings[2], returned_all
        );
        flag = false;
    }

    if flag {
        println!("\nClient: Received all correctly compressed strings!");
        for i in 0..3 {
            match i {
                0 => {
                    println!(
                        "\nClient: Sent {:?}, Received {:?}",
                        strings[i], returned_simple
                    );
                },
                1 => {
                    println!(
                        "\nClient: Sent {:?}, Received {:?}",
                        strings[i], returned_none
                    );
                }
                2 => {
                    println!(
                        "\nClient: Sent {:?}, Received {:?}",
                        strings[i], returned_all
                    );
                }
                _ => {continue}
            };
        }
    }

    let (_, _, ping_msg) = results[3];
    let (_, _, stats_msg) = results[4];

    if ping_msg.get() != (0, 0) {
        println!(
            "\nClient: Message received in response to PING not as expected! \
             Returned header {:?}",
            ping_msg.get()
        );
    }

    let res = serialize_to_stats(stats[0].clone());

    if res.is_err() {
        println!("\nClient: unable to serialize received stats data");
    } else {
        let (returned_sent, returned_rcv, returned_ratio) = res.unwrap();
        if returned_rcv != count_sent {//- 8 /*size of header*/ {
            println!(
                "\nClient: Stats received not as expected! \
                 Local count for sent bytes {:?}, response {:?}",
                 count_sent, returned_rcv
            );
        }
        if returned_sent != count_rcv  {//- 8 /*size of header*/{
            println!(
                "\nClient: Stats received not as expected! \
                 Local count for received bytes {:?}, response {:?}",
                 count_rcv, returned_sent
            );
        }

        println!(
            "\nClient: Stats received server has sent {:?} \
             server has received {:?}, \
             server compression ratio {:?}",
            returned_sent, returned_rcv, returned_ratio
        );
    }
    /* ~~~ assert that the appropriate failure conditions are handled as expected ~~~ */
    let mut error_flag = false;
    for i in 0..more_tests.len() {
        let stream = TcpStream::connect("127.0.0.1:4000");

        if stream.is_ok() {
            let mut stream = stream.unwrap();
            let mut length: usize = 0;

            let request_msg: message::Message;
            if more_tests[i] == super::COMPRESS {
                let payload = bad_strings[i].as_bytes();
                length = payload.len();
                request_msg =
                    message::Message::new_with_payload(length as u16, 4, payload.to_vec());
            } else {
                request_msg = message::Message::new(0, more_tests[i]);
            }
            let send_bytes = bincode::serialize(&request_msg).unwrap();

            stream.write(send_bytes.as_slice());
            let mut read_it = [0u8; super::MAX_MSG_LEN];
            stream.read(&mut read_it);

            let message = bincode::deserialize(&read_it);
            let mut msg: message::Message = message::Message::default();
            if message.is_err() {
                println!("\nClient: Unable to deserialize read-in bytes");
            } else {
                msg = message.unwrap();
                println!(
                    "\nClient: received response with header {:?} and payload {:?}",
                    msg.get_header(),
                    msg.get_payload()
                );

                if more_tests[i] == super::COMPRESS && msg.get_header().get() != (0, super::EINVAL){
                    println!("Server did not return appropriate error for bad string!");
                    error_flag = true;
                    //println!("Set compressed array to {:?} versus compressed[i] {:?}", returned.clone(), compressed[i]);
                } else if more_tests[i] > 4 && msg.get_header().get() != (0, super::ENOSUP){
                    println!("Server did not return appropriate error for bad string!");
                    error_flag = true;

                }

                count_sent += length as u32 + std::mem::size_of::<message::MessageHeader>() as u32;
                count_rcv += msg.get_payload().len() as u32
                    + std::mem::size_of::<message::MessageHeader>() as u32;

                results[i] = (
                    length as u32 + std::mem::size_of::<message::MessageHeader>() as u32,
                    msg.get_payload().len() as u32
                        + std::mem::size_of::<message::MessageHeader>() as u32,
                    msg.get_header(),
                );
            }
        } else {
            println!("\nClient: unable to connect to TCP server"); //, exiting");
            process::exit(1);
        }
    }

    if !error_flag{
        println!("\nClient: all failure conditions handled appropriately by server!");

    } else {
        println!("\nClient: failure conditions not handled appropriately by server, needs fixing");
    }

    println!("\nClient: checking stats reset...");
    let stats_msg = message::Message::new(0, super::GET);
    let stream = TcpStream::connect("127.0.0.1:4000");

    if stream.is_ok() {
        let mut stream = stream.unwrap();
        let send_bytes = bincode::serialize(&stats_msg).unwrap();
        stream.write(send_bytes.as_slice());
        let mut read_it = [0u8; super::MAX_MSG_LEN];
        stream.read(&mut read_it);

        let message = bincode::deserialize(&read_it);
        let mut msg: message::Message = message::Message::default();
        if message.is_err() {
            println!("\nClient: Unable to deserialize read-in bytes");
        } else {
            msg = message.unwrap();

            let res = serialize_to_stats(msg.get_payload().clone());

            if res.is_err() {
                println!("\nClient: unable to serialize received stats data");
            } else {
                let (returned_sent, returned_rcv, returned_ratio) = res.unwrap();
                println!("\nClient: received this data, post reset request! sent {:?} rcv {:?} ratio {:?}", returned_sent, returned_rcv, returned_ratio);
            }

        }
    }else {
        println!("\nClient: unable to connect to TCP server");
        process::exit(1);
    }

    //create a message of length
    let mut long_msg: Vec<char> = Vec::new();
    for i in 0..MAX_PAYLOAD_LEN / 4{
        long_msg.push('z');  //z in ascii
        long_msg.push('z');
        long_msg.push('z');
        long_msg.push('z');
    }

    let long_string: String = long_msg.into_iter().collect();

    println!("\nClient: FINAL CHECK-- checking stats reset with new ratio! Compression should be 50");
    let compress_msg = message::Message::new_with_payload(long_string.len() as u16, super::COMPRESS, long_string.as_bytes().to_vec());
    let stream = TcpStream::connect("127.0.0.1:4000");

    if stream.is_ok() {
        let mut stream = stream.unwrap();

        let send_bytes = bincode::serialize(&compress_msg).unwrap();
        stream.write(send_bytes.as_slice());
        let mut read_it = [0u8; super::MAX_MSG_LEN];
        stream.read(&mut read_it);

        let message = bincode::deserialize(&read_it);
        let mut msg: message::Message = message::Message::default();
        if message.is_err() {
            println!("\nClient: Unable to deserialize read-in bytes");
        } else {
            msg = message.unwrap();

           println!("\nClient: got this back header {:?}, string {:?}", msg.get_header(), str::from_utf8(msg.get_payload().as_slice()).unwrap());

        }
    }else {
        println!("\nClient: unable to connect to TCP server");
        process::exit(1);
    }

    let stats_msg = message::Message::new(0, super::GET);
    let stream = TcpStream::connect("127.0.0.1:4000");

    if stream.is_ok() {
        let mut stream = stream.unwrap();
        let send_bytes = bincode::serialize(&stats_msg).unwrap();
        stream.write(send_bytes.as_slice());
        let mut read_it = [0u8; super::MAX_MSG_LEN];
        stream.read(&mut read_it);

        let message = bincode::deserialize(&read_it);
        let mut msg: message::Message = message::Message::default();
        if message.is_err() {
            println!("\nClient: Unable to deserialize read-in bytes");
        } else {
            msg = message.unwrap();

            let res = serialize_to_stats(msg.get_payload().clone());

            if res.is_err() {
                println!("\nClient: unable to serialize received stats data");
            } else {
                let (returned_sent, returned_rcv, returned_ratio) = res.unwrap();
                println!("\nClient: received this data, post reset request! sent {:?} rcv {:?} ratio {:?}", returned_sent, returned_rcv, returned_ratio);
            }

        }
    }else {
        println!("\nClient: unable to connect to TCP server");
        process::exit(1);
    }

    process::exit(0);
}
