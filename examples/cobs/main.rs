
extern crate yabts;
use std::env;
use std::process;
use yabts::*;
//use std::io;

pub mod cobs;

use cobs::*;

fn main() {
    //only accept 1, 3, 4, or 5 args
    if env::args().len() > 5 || env::args().len() == 2 {
        display_useage();
        exit(EINVAL);
    }

    let mut addr_str: String = "".to_string();
    let mut port_str: String = "".to_string();
    let mut client_mode: String = "".to_string();
    let mut file: String = "".to_string();

    match parse_args(&mut port_str, &mut addr_str, &mut client_mode, &mut file) {
        1 => {
            run_client_mode(&mut client_mode, &mut file);
        }
        _ => {}
    }

    //ensure that we have been given a valid number for a port before we try to bind to a socket
    if u16::from_str_radix(&port_str, 10).is_err() {
        println!("\nERROR: Invalid address\n");
        display_useage();
        exit(EINVAL);
    }
    if addr_str == "localhost".to_string() {
        addr_str = "127.0.0.1".to_string();
    }

    let method = COBS {};

    let server = server::Server::new(transform::TransformPayload {
        to_method_e: Some(Box::new(method)),
        from_method_ed: Some(Box::new(method.clone())),
        to_method_c: None,
        from_method_cd: None,
    });
    if server.is_err() {
        println!("\nERROR: internal server error (did you provide an address and port you have access to?)\n");
        display_useage();
        exit(EINVAL);
    }
    let mut server = server.unwrap();
    println!(
        "\nServer initilized, listening on address {:?}, port {:?}...",
        addr_str, port_str
    );
    let status = server.listen(addr_str.clone(), port_str.clone());
    exit(status);
}

pub fn run_client_mode(mode: &mut String, file: &mut String) {
    //check that the request is a valid u16
    let request = u16::from_str_radix(&mode, 10);
    if request.is_err() {
        display_useage();
        exit(EINVAL);
    }

    let status;
    if file == "" {
        status = client::run_client(request.unwrap() as u16, None)
    } else {
        status = client::run_client(request.unwrap() as u16, Some(file))
    }
    exit(status);
}

///Parse args such that a user can enter args in any order and all are optional
fn parse_args(
    port_str: &mut String,
    addr_str: &mut String,
    mode: &mut String,
    file: &mut String,
) -> u32 {
    if env::args().len() == 1 {
        *port_str = "4000".to_string();
        *addr_str = "127.0.0.1".to_string();
        return 0;
    } else {
        let p = "-p";
        let a = "-a";
        let c = "-c";

        if env::args().len() == 3 {
            let arg = env::args().nth(1).unwrap();

            match arg.as_str() {
                _ if p == arg => {
                    *port_str = env::args().nth(2).unwrap();
                    *addr_str = "127.0.0.1".to_string();
                }
                _ if a == arg => {
                    *addr_str = env::args().nth(2).unwrap();
                    *port_str = "4000".to_string();
                }
                _ if c == arg => {
                    *mode = env::args().nth(2).unwrap();
                    return 1;
                }
                _ => {
                    display_useage();
                    exit(EINVAL);
                }
            };
        } else if env::args().len() == 4 {
            let compress = "4".to_string();
            let arg1 = env::args().nth(1).unwrap();
            let arg2 = env::args().nth(2).unwrap();
            match arg1.as_str() {
                _ if c == arg1 => match arg2.as_str() {
                    _ if compress == arg2 => {
                        *mode = env::args().nth(2).unwrap();
                        *file = env::args().nth(3).unwrap();
                        return 1;
                    }
                    _ => {
                        display_useage();
                        exit(EINVAL);
                    }
                },
                _ => {
                    display_useage();
                    exit(EINVAL);
                }
            };
        } else if env::args().len() == 5 {
            let arg1 = env::args().nth(1).unwrap();
            let arg2 = env::args().nth(3).unwrap();

            match arg1.as_str() {
                _ if p == arg1 => {
                    *port_str = env::args().nth(2).unwrap();
                }
                _ if a == arg1 => {
                    *addr_str = env::args().nth(2).unwrap();
                }
                _ => {
                    display_useage();
                    exit(EINVAL);
                }
            };

            match arg2.as_str() {
                _ if p == arg2 => {
                    *port_str = env::args().nth(4).unwrap();
                }
                _ if a == arg2 => {
                    *addr_str = env::args().nth(4).unwrap();
                }
                _ => {
                    display_useage();
                    exit(EINVAL);
                }
            };
        }
    }
    return 0;
}

fn display_useage() {
    println!(
        "Useage:{} -p <port> [-a <address>] [-c <request> [file name]]\n run -c for client mode. \
         Specify valid request type and include file name for compression requests\
         \nDefault mode a.k.a. server mode will run in server mode on 127.0.0.1:4000",
        env::args().nth(0).unwrap()
    );
}

fn exit(status: u16) {
    process::exit(status as _);
}
