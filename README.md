# YABTS: Yet Another Byte Transformation Service #

This crate provides a simple data transformation service and includes a library 
and trait definitions for user-extensibility. It provides both server and client 
modes for a socket-based (currently TCP) mechanism to transform data in both 
directions (e.g. compression on server end, decompression on client end). 

The data transformation provided by this service is intended to be easily extended 
by a user, but includes some basic algorithms. This transformation is in addition 
to the byte serialization/ deserialization needed for sending data via network 
socket. A data payload, provided via `stdin` or file, is transformed according to 
the user-specified trait implementation via a series of generic functions for 
encoding/decoding, compressing/decompressing, etc., and then further transformed 
to handle byte marshalling across sockets. 

The `examples` dir contains  examples of already-implemented data transforms, 
specifically compression/ decompression and encoding/decoding implementations of:

    – Base 64
    – COBS (Consistent Overhead Byte Stuffing)
    – "Simple" compression scheme for a trivial compression/decompression (see example)
    – Using 3rd party crate for Snappy algorithm implementation (WIP)
    – Possibly others, TBD

### Why do? ###

Great question. I initially wrote this for an interview code assignment asking for a 
TCP-based compression service with a lot of "hidden gotchas" requirements. So a lot 
of the initial structure is based on the specification document. 
I have since begun to build up more functionality and turn it into something new
 for the following reasons:
 - I enjoyed the overall concept as something that exercises multiple skillsets (algorithms, networks, etc.) 
 - I wanted to showcase some of the more interesting features of Rust 
(such as dynamic trait objects). 
 - Most of all I wanted to implement more interesting and complex 
 encoding and compression algorithms and this is a very easy vessel to do so.
 
As such this is a work in progress and mainly exists for my own edification. Also, all company-specific details (e.g. solutions) have
been obscured per request. 

## Target Platform // Development Environment ##

Development environment is Ubuntu 18.04.3 LTS. It is written in Rust using the 
as-of-this-writing latest stable toolchain e.g. rustc 1.48.0 (7eac88abb 2020-11-16).

## Description ##

The service uses a simple client-server design pattern: it is a single
threaded TCP server that responds to the various requests as listed 
in the initial spec. The crate is composed of a library and a series of example binaries 
which can run in either client or server mode. 
```


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

```

## 3rd Party Libraries ##

For this service I have used the `serde` and `bincode` libraries, which together
perform serialization and deserialization of bytes sent over the TCP socket. From 
`bincode`'s documentation: "A compact encoder / decoder pair that uses a binary 
zero-fluff encoding scheme" This handles encoding and decoding network byte order.
Also used is the argparse crate for commandline option parsing