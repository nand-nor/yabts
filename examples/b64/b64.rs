use std::io; //error::Error;
use std::iter::Iterator;
use yabts::transform::*;

//use std::str;

//pub mod b64 {
#[derive(Copy, Clone)]
pub struct B64 {
    // pub bytes: vec<u8>
//helpers needed?
//pub Payload<>
}
/*
    impl Encode<Vec<u8>> for Payload<Vec<u8>> {
        fn encode<T>(payload: Vec<u8>) -> Result<Vec<u8>, dyn Error> {
            Self
        }

    }
*/

impl _Encode for B64 {
    fn _encode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        println!("Running the inner Encode method!!!\n");
        let mut new_vec: Vec<u8> = Vec::new();
        base64encode(payload, &mut new_vec);

        payload.clear();
        *payload = new_vec.clone();

        Ok(())
    }
}

impl _Decode for B64 {
    fn _decode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        println!("Running the inner Decode method!!!\n");
        for i in payload.iter() {
            println!("{}", i);
        }
        Ok(())
    }
}

pub fn b64() {
    let test = B64 {};
    let mut vec: Vec<u8> = Vec::new();
    let payload = EncodePayload {
        to_method: Box::new(test),
        from_method: Box::new(test.clone()),
    };
    payload.run_to(&mut vec);
    payload.run_from(&mut vec);

    println!("Example calling all methods complete");
}

fn base64encode(input: &mut Vec<u8>, output: &mut Vec<u8>) -> u64 {
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

        /*
         * if we have only two bytes available, then their encoding is
         * spread out over three chars
         */
        if (i + 1) < in_len {
            output.push(b64charvec[n2 as usize] as u8);
        }

        /*
         * if we have all three bytes available, then their encoding is spread
         * out over four characters
         */
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
    0
}
/*
impl <E: _Encode> ___Payload<E> for _Encode{
    type Input = Vec<u8>;a
    fn encode<E, T>(&self, payload: &mut T) -> Result<(), io::Error>{
        E::_encode(self, T)
    }

YWFhYWJiYmJjY2Nj

}*/
//}
