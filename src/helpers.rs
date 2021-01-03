use crate::*;
use std::env;
use std::process;

use argparse::{ArgumentParser, StoreOption};

pub fn run_client_mode(
    addr: String,
    port: String,
    request: Option<super::REQUEST>,
    file: Option<String>,
) -> u16 {

    if let Some(req) = request {
        client::run_client(addr, port, req, file)
    } else {
        return super::EINVAL;
    }
}

///Parse args such that a user can enter args in any order and all are optional
pub fn parse_args() -> RUNMODE {
    let mut mode: Option<String> = None;
    let mut file: Option<String> = None;
    let mut addr_str: Option<String> = Some("127.0.0.1".to_string());
    let mut port_str: Option<String> = Some("4000".to_string());
    //let mut request: Option<String> = None;

    let mut req: Option<super::REQUEST> = None;

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("YABTS: Transform some bytes via TCP service");

        parser.refer(&mut mode).add_option(
            &["-m", "--mode"],
            StoreOption,
            r#"RUNMODE: default is server, use client for client mode"#,
        );

        parser.refer(&mut req).add_option(
            &["-r", "--request"],
            StoreOption,
            r#"REQUEST: If in client mode, specify the request to send to server"#,
        );

        parser.refer(&mut file).add_option(
            &["-f", "--file"],
            StoreOption,
            r#"FILE: If in client mode and requestin transformation, specify file"#,
        );

        parser.refer(&mut addr_str).add_option(
            &["-a", "--address"],
            StoreOption,
            r#"SERVER ADDRESS (default is 127.0.0.1)"#,
        );

        parser.refer(&mut port_str).add_option(
            &["-p", "--port"],
            StoreOption,
            r#"SERVER PORT (default is 4000)"#,
        );

        parser.parse_args_or_exit();
    }

    if let Some(mode) = mode {
        match mode.as_str() {
            "client" => RUNMODE::CLIENT(addr_str.unwrap(), port_str.unwrap(), req, file),
            "server" => RUNMODE::SERVER(addr_str.unwrap(), port_str.unwrap()),
            _ => RUNMODE::SERVER(addr_str.unwrap(), port_str.unwrap()),
        }
    } else {
        RUNMODE::SERVER(addr_str.unwrap(), port_str.unwrap())
    }
}

pub fn display_useage() {
    println!(
        "Useage:{} [-p <port>] [-a <address>] [-m <runmode>] [-r <request>] [-f <file name>]\n\
         -p : \tspecify port for server (default is 4000)\n\
         -a : \tspecify address for server(default is 127.0.0.1)\n\
         -m : \tuse 'client' for client mode, 'server' for server mode. \n\
         -r : \tfor client mode only, specify request using one of: encode/decode/compress/decompress\n\
         -f : \tfor client mode only, optional file name for byte transform request\n\
         \nIf no args provided, will default to run in server mode on 127.0.0.1:4000",
        env::args().nth(0).unwrap()
    );
}

pub fn exit(status: u16) {
    process::exit(status as _);
}
