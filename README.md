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

    – Lehmer code
    – Consistent overhead byte stuffing (COBS)
    – Base 64
    – "Simple" compression scheme for a trivial compression/decompression (see example)
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
as-of-this-writing latest stable toolchain e.g. rustc 1.41.0 (5e1a79984 2020-01-27).

## Building ##
 
For users who do not already have the Rust toolchain, run `build.sh`. This script 
will by default install the latest version of `rustc` and `cargo`. For those who
already have Rust toolchains. simply use the Makefile to compile via `make release`. 

By default this will build the server binary. To build the client binary / run in
client mode, run `make client_test` option. 

## Description ##

The compression service uses a simple client-server design pattern: it is a single
threaded TCP server that responds to the various requests as listed 
in the initial spec. The crate is composed of a library and a binary 
(`src/bin/compression.rs` defines `main()`) which can run in either client 
or server mode. 

Specifying the `-c` option will run the binary in client mode, however a server instance 
must be running prior to running client mode. To run in client mode, all requests
are made via command line. If requesting compression, then you must specify a file
with the to-be-compressed bytes.  

To run in server mode, do not specify -c. Server mode takes either no arguments, 3, 
or 5 arguments: use -p to specify a port and -a to specify an address. 
The default address and port, if either or none specified, is 127.0.0.1:400.

The Server object, as defined in server.rs, runs a simple TcpListener object, and holds
an inner struct to track service statistics. It runs an event loop to listen for and 
then handle TCP stream connections as they are made by a client. Each new stream is 
handled by parsing the incoming request, processing the request, and generating the 
appropriate response, which is written back to the stream. 

For testing I have added in a `run_client_tests()` function, defined in the client 
module to conditionally compile a program with the `client` feature enabled, which
automates sending various messages to the service after it has been launched. Other 
simple unit tests for correct compression handling are defined in lib.rs


## 3rd Party Libraries ##

For this service I have used the `serde` and `bincode` libraries, which together
perform serialization and deserialization of bytes sent over the TCP socket. From 
`bincode`'s documentation: "A compact encoder / decoder pair that uses a binary 
zero-fluff encoding scheme" This handles encoding and decoding network byte order.

## Assumptions ##

The main assumptions made are around the appropriate error handling, and what conditions 
consitute an error beyond that which is specified in the spec. For example, the following
decisions around error conditions are implemented:
   
1. The following error conditions will generate an appropriate error response from server
- Sending invalid message header (pulled via peek) e.g. undecodeable bytes or invalid magic number
- Sending invalid message (pulled via full read) e.g. undecodeable bytes in the message or
in the payload of a compression request message
- Sending message with incorrect payload length e.g. payload.len() != msg_header.get_length() as usize
2. The following error conditions will not generate an error response due TCP failure
- TCP stream error (failure to read or write)

Any TCP stream error will set the server's internal error state to INTERNAL_ERROR,
which, after this has occurred, will be sent back to any requester regardless
of the request.

Additionally, other error conditions are handled internally. For example, 
if adding new bytes to the stored statistics will cause an overflow of the u32 that
holds them, the values are modified for wrapping addition. 

## Improvements ##

Given more time I would like to make the following improvements. Mainly I would
take the time to make the server object multi-threaded, using some mechanism 
like select or epoll, since as it stands a single-threaded server makes using 
TCP inefficient. The way it is structured now, due to scoping and the lifetime
of a stream, after the response is sent from the server, the stream is dropped
out of scope and the connection closes, so a client cannot maintain a 
persistent TCP connection beyond 1 request/response interaction at a time. 
It is possible that another protocol woul dbe more suitable for single threaded,
like UDP, so that is another option I'd work on given more time. However given 
the specification of the service requirements and the time limit of ~5 hours
single threaded TCP is likely sufficient (although probably not ideal from 
the client's perspective). Adding support for IpV6 addresses would also be
potentially valuable. 

Also given more time I would like to have done more testing and built out more
robust error handling. The internal error state tracking could definitely be
given a lot more attention.  

There are also some obvious code refactoring opportunities throughout the 
code in server.rs, mainly in the function definition of `handle_client`;
the mutability of TcpStream objects prevents a more 
elegant-and-quick-to-develop approach, so given more time I'd like 
to explore possibilities for making that nicer. 


## TODO ##
- Implement UDP option
- Implement comprehensive test suite
- Implement other encoding/decoding schemes
    
    - b64
    - lehmer (may be too complex?)
    - COBS (may be stupid?)
    - whatelse?
    - maybe encryption?
   
- Restructure crate into something a user can pull in and then implement their own trait objects
- Make server multi threaded? Not sure we really need this optimization, 
the service is not intended to scale to serve many concurrent users...
- 