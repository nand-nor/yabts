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
    – Using 3rd party crate for Snappy algorithm implementation
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

The service uses a simple client-server design pattern: it is a single
threaded TCP server that responds to the various requests as listed 
in the initial spec. The crate is composed of a library and a series of example binaries 
which can run in either client or server mode. 

Specifying the `-c` option will run the binary in client mode, however a server instance 
must be running prior to running client mode. To run in client mode, all requests
are made via command line. If requesting compression, then you must specify a file
with the to-be-compressed bytes.  

To run in server mode, do not specify -c. Server mode takes either no arguments, 3, 
or 5 arguments: use -p to specify a port and -a to specify an address. 
The default address and port, if either or none specified, is 127.0.0.1:400.


## 3rd Party Libraries ##

For this service I have used the `serde` and `bincode` libraries, which together
perform serialization and deserialization of bytes sent over the TCP socket. From 
`bincode`'s documentation: "A compact encoder / decoder pair that uses a binary 
zero-fluff encoding scheme" This handles encoding and decoding network byte order.


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
