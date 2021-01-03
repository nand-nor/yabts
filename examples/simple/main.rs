extern crate yabts;


use yabts::{helpers::*, *};

pub mod simple;

use simple::*;

fn main() {
    let status = match parse_args() {
        RUNMODE::CLIENT(addr, port, mode, file) => run_client_mode(addr, port, mode, file),
        RUNMODE::SERVER(addr, port) => {
            let method = Simple {};

            let server = server::Server::new(
                transform::TransformPayload {
                    to_method_e: None,
                    from_method_ed: None,
                    to_method_c: Some(Box::new(method)),
                    from_method_cd: Some(Box::new(method.clone())),
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
