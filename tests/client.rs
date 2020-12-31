use bincode;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process;
use std::str;

use yabts::{message, MAX_MSG_LEN, *};

/*
fn serialize_to_stats(payload: Vec<u8>) -> Result<(u64, u64, u8), ()> {
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
        PING => {
            println!("\nCLIENT:  requesting ping...");
            message::Message::new(0, request, None)
        }
        GET => {
            println!("\nCLIENT:  requesting server stats...");
            message::Message::new(0, request, None)
        }
        RESET => {
            println!("\nCLIENT: requesting server stat reset...");
            message::Message::new(0, request, None)
        }
        COMPRESS => {
            println!("\nCLIENT: requesting compression...");
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
}*/

pub fn run_client_tests() {
    let simple = "aaabbbcccddd";
    let no_compress = "abcdefghijklmnop";
    let all_compress = "xxxxxxxxxxxxxxxxxxxxxxxxxx";
    //assert these pass
    let test_cases = [COMPRESS, COMPRESS, COMPRESS, PING, GET];
    //assert these fail
    let more_tests = [COMPRESS, COMPRESS, COMPRESS, 5, 99, RESET];
    let mut results: [(u32, u32, message::MessageHeader); 8] =
        [(0, 0, message::MessageHeader::default()); 8];
    let strings = [simple, no_compress, all_compress];
    let bad_strings: [&str; 3] = ["ABCDEFG", "0xdeadbeef", "jjjjaaazzz!"];
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
            if test_cases[i] == COMPRESS {
                let payload = strings[i].as_bytes();
                length = payload.len();
                request_msg = message::Message::new(length as u16, 4, Some(payload.to_vec()));
            } else {
                request_msg = message::Message::new(0, test_cases[i], None);
            }
            let send_bytes = bincode::serialize(&request_msg).unwrap();

            stream.write(send_bytes.as_slice());
            println!("\nClient: wrote {:?} (serialized)", send_bytes);

            let mut read_it = [0u8; MAX_MSG_LEN];
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
                    msg.get_payload().unwrap()
                );

                if test_cases[i] == COMPRESS {
                    // let payload = msg.get_payload();

                    compressed.push(msg.get_payload().unwrap()); //returned.clone();
                }

                if test_cases[i] == GET {
                    stats.push(msg.get_payload().unwrap());
                } else {
                    println!(
                        "Incrementing count sent and rcv!! Sent was {:?}, rcv was {:?}",
                        count_sent, count_rcv
                    );
                    count_sent +=
                        length as u32 + std::mem::size_of::<message::MessageHeader>() as u32;
                    count_rcv += msg.get_payload().unwrap().len() as u32
                        + std::mem::size_of::<message::MessageHeader>() as u32;
                    println!(" Sent is now {:?}, rcv is now {:?}", count_sent, count_rcv);
                }

                results[i] = (
                    length as u32 + std::mem::size_of::<message::MessageHeader>() as u32,
                    msg.get_payload().unwrap().len() as u32
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
                }
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
                _ => continue,
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

    let res = client::serialize_to_stats(stats[0].clone());

    if res.is_err() {
        println!("\nClient: unable to serialize received stats data");
    } else {
        let (returned_sent, returned_rcv, returned_ratio) = res.unwrap();
        if returned_rcv != count_sent as u64 {
            //- 8 /*size of header*/ {
            println!(
                "\nClient: Stats received not as expected! \
                 Local count for sent bytes {:?}, response {:?}",
                count_sent, returned_rcv
            );
        }
        if returned_sent != count_rcv as u64 {
            //- 8 /*size of header*/{
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
            if more_tests[i] == COMPRESS {
                let payload = bad_strings[i].as_bytes();
                length = payload.len();
                request_msg = message::Message::new(length as u16, 4, Some(payload.to_vec()));
            } else {
                request_msg = message::Message::new(0, more_tests[i], None);
            }
            let send_bytes = bincode::serialize(&request_msg).unwrap();

            stream.write(send_bytes.as_slice());
            let mut read_it = [0u8; MAX_MSG_LEN];
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

                if more_tests[i] == COMPRESS && msg.get_header().get() != (0, EINVAL) {
                    println!("Server did not return appropriate error for bad string!");
                    error_flag = true;
                //println!("Set compressed array to {:?} versus compressed[i] {:?}", returned.clone(), compressed[i]);
                } else if more_tests[i] > 4 && msg.get_header().get() != (0, ENOSUP) {
                    println!("Server did not return appropriate error for bad string!");
                    error_flag = true;
                }

                count_sent += length as u32 + std::mem::size_of::<message::MessageHeader>() as u32;
                count_rcv += msg.get_payload().unwrap().len() as u32
                    + std::mem::size_of::<message::MessageHeader>() as u32;

                results[i] = (
                    length as u32 + std::mem::size_of::<message::MessageHeader>() as u32,
                    msg.get_payload().unwrap().len() as u32
                        + std::mem::size_of::<message::MessageHeader>() as u32,
                    msg.get_header(),
                );
            }
        } else {
            println!("\nClient: unable to connect to TCP server"); //, exiting");
            process::exit(1);
        }
    }

    if !error_flag {
        println!("\nClient: all failure conditions handled appropriately by server!");
    } else {
        println!("\nClient: failure conditions not handled appropriately by server, needs fixing");
    }

    println!("\nClient: checking stats reset...");
    let stats_msg = message::Message::new(0, GET, None);
    let stream = TcpStream::connect("127.0.0.1:4000");

    if stream.is_ok() {
        let mut stream = stream.unwrap();
        let send_bytes = bincode::serialize(&stats_msg).unwrap();
        stream.write(send_bytes.as_slice());
        let mut read_it = [0u8; MAX_MSG_LEN];
        stream.read(&mut read_it);

        let message = bincode::deserialize(&read_it);
        let mut msg: message::Message = message::Message::default();
        if message.is_err() {
            println!("\nClient: Unable to deserialize read-in bytes");
        } else {
            msg = message.unwrap();

            let res = client::serialize_to_stats(msg.get_payload().unwrap().clone());

            if res.is_err() {
                println!("\nClient: unable to serialize received stats data");
            } else {
                let (returned_sent, returned_rcv, returned_ratio) = res.unwrap();
                println!("\nClient: received this data, post reset request! sent {:?} rcv {:?} ratio {:?}", returned_sent, returned_rcv, returned_ratio);
            }
        }
    } else {
        println!("\nClient: unable to connect to TCP server");
        process::exit(1);
    }

    //create a message of length
    let mut long_msg: Vec<char> = Vec::new();
    for i in 0..MAX_PAYLOAD_LEN / 4 {
        long_msg.push('z'); //z in ascii
        long_msg.push('z');
        long_msg.push('z');
        long_msg.push('z');
    }

    let long_string: String = long_msg.into_iter().collect();

    println!(
        "\nClient: FINAL CHECK-- checking stats reset with new ratio! Compression should be 50"
    );
    let compress_msg = message::Message::new(
        long_string.len() as u16,
        COMPRESS,
        Some(long_string.as_bytes().to_vec()),
    );
    let stream = TcpStream::connect("127.0.0.1:4000");

    if stream.is_ok() {
        let mut stream = stream.unwrap();

        let send_bytes = bincode::serialize(&compress_msg).unwrap();
        stream.write(send_bytes.as_slice());
        let mut read_it = [0u8; MAX_MSG_LEN];
        stream.read(&mut read_it);

        let message = bincode::deserialize(&read_it);
        let mut msg: message::Message = message::Message::default();
        if message.is_err() {
            println!("\nClient: Unable to deserialize read-in bytes");
        } else {
            msg = message.unwrap();

            println!(
                "\nClient: got this back header {:?}, string {:?}",
                msg.get_header(),
                str::from_utf8(msg.get_payload().unwrap().as_slice()).unwrap()
            );
        }
    } else {
        println!("\nClient: unable to connect to TCP server");
        process::exit(1);
    }

    let stats_msg = message::Message::new(0, GET, None);
    let stream = TcpStream::connect("127.0.0.1:4000");

    if stream.is_ok() {
        let mut stream = stream.unwrap();
        let send_bytes = bincode::serialize(&stats_msg).unwrap();
        stream.write(send_bytes.as_slice());
        let mut read_it = [0u8; MAX_MSG_LEN];
        stream.read(&mut read_it);

        let message = bincode::deserialize(&read_it);
        let mut msg: message::Message = message::Message::default();
        if message.is_err() {
            println!("\nClient: Unable to deserialize read-in bytes");
        } else {
            msg = message.unwrap();

            let res = client::serialize_to_stats(msg.get_payload().unwrap().clone());

            if res.is_err() {
                println!("\nClient: unable to serialize received stats data");
            } else {
                let (returned_sent, returned_rcv, returned_ratio) = res.unwrap();
                println!("\nClient: received this data, post reset request! sent {:?} rcv {:?} ratio {:?}", returned_sent, returned_rcv, returned_ratio);
            }
        }
    } else {
        println!("\nClient: unable to connect to TCP server");
        process::exit(1);
    }

    process::exit(0);
}
