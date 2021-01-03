use std::io;
use std::iter::Iterator;
use yabts::transform::*;

#[derive(Copy, Clone)]
pub struct B64 {}

impl _Encode for B64 {
    fn _encode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        let mut new_vec: Vec<u8> = Vec::new();
        let res = base64encode(payload, &mut new_vec);
        payload.clear();
        *payload = new_vec.clone();
        res
    }
}

impl _Decode for B64 {
    fn _decode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        let mut new_vec: Vec<u8> = Vec::new();
        let res = base64decode(payload, &mut new_vec);
        payload.clear();
        *payload = new_vec.clone();
        res
    }
}

//TODO: IS there a "safe" way to have the b64 char vec global?
fn base64encode(input: &mut Vec<u8>, output: &mut Vec<u8>) -> Result<(), io::Error> {
    let in_len: usize = input.len();
    let pad: usize = in_len % 3;
    let mut n: u32; // = 0;
    let mut n0: u8; // = 0;
    let mut n1: u8; // = 0;
    let mut n2: u8; // = 0;
    let mut n3: u8; // = 0;
    let mut i = 0;
    let b64charvec: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .chars()
        .collect();
    // for i in (0..in_len).step_by(3) {
    while i < in_len {
        n = (input[i] as u32) << 16;
        if (i + 1) < in_len {
            n += (input[i + 1] as u32) << 8;
        }
        if (i + 2) < in_len {
            n += input[i + 2] as u32;
        }

        /* this 24-bit number gets separated into four 6-bit numbers */
        n0 = ((n >> 18) & 63) as u8;
        n1 = ((n >> 12) & 63) as u8;
        n2 = ((n >> 6) & 63) as u8;
        n3 = (n & 63) as u8;

        output.push(b64charvec[n0 as usize] as u8);
        output.push(b64charvec[n1 as usize] as u8);

        if (i + 1) < in_len {
            output.push(b64charvec[n2 as usize] as u8);
        }
        if (i + 2) < in_len {
            output.push(b64charvec[n3 as usize] as u8);
        }
        i += 3;
    }

    if pad > 0 {
        for _i in pad..3 {
            output.push(0x3d);
        }
    }
    output.push(0);
    Ok(())
}

//TODO!! Implement this
fn base64decode(input: &mut Vec<u8>, output: &mut Vec<u8>) -> Result<(), io::Error> {
    let in_len: usize = input.len();
    let mut i = 0;

    let d: Vec<u8> = vec![
        66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 64, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
        66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 62, 66, 66,
        66, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 66, 66, 66, 65, 66, 66, 66, 0, 1, 2, 3, 4,
        5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 66, 66, 66,
        66, 66, 66, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
        46, 47, 48, 49, 50, 51, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
        66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
        66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
        66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
        66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
        66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
        66,
    ];
    let mut iter: u8 = 0;
    let mut buf: u32 = 0;
    while i < in_len {
        let idx = input[i];
        let c = d[idx as usize];

        match c {
            64 => {
                i += 1;
                continue;
            }
            65 => {
                break;
            }
            66 => {
                //TODO err
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid bytes for b64 decoding",
                ));
            }
            _ => {
                buf = buf << 6 | c as u32;
                iter += 1;
                if iter == 4 {
                    output.push(((buf >> 16) & 255) as u8);
                    output.push(((buf >> 8) & 255) as u8);
                    output.push((buf & 255) as u8);
                    buf = 0;
                    iter = 0;
                }
            }
        };
        i += 1;
    }

    if iter == 3 {
        output.push(((buf >> 10) & 255) as u8);
        output.push(((buf >> 2) & 255) as u8);
    } else if iter == 2 {
        output.push(((buf >> 4) & 255) as u8);
    }
    Ok(())
}
