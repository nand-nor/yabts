extern crate yabts;

use std::env;
use std::process;
use yabts::{helpers::*, *};

pub mod cobs;
use cobs::*;

fn main() {
    let status = match parse_args() {
        RUNMODE::CLIENT(addr, port, mode, file) => run_client_mode(addr, port, mode, file),
        RUNMODE::SERVER(addr, port) => {
            let method = COBS {};

            let server = server::Server::new(
                transform::TransformPayload {
                    to_method_e: Some(Box::new(method)),
                    from_method_ed: Some(Box::new(method.clone())),
                    to_method_c: None,
                    from_method_cd: None,
                },
                addr.clone(),
                port.clone(),
            );
            if server.is_err() {
                println!("\nERROR: internal server error\n");
                display_useage();
                exit(EINVAL);
            }
            let mut server = server.unwrap();
            println!(
                "\nServer initilized, listening on address {:?}, port {:?}...",
                addr, port
            );
            server.listen()
        }
    };

    exit(status);
}
