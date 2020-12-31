use std::io; //error::Error;
use std::iter::Iterator;
use yabts::transform::*;


#[derive(Copy, Clone)]
pub struct COBS{}

impl _Encode for COBS {
    fn _encode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        println!("Running the inner Encode method!!!\n");
        let mut new_vec: Vec<u8> = Vec::new();
        stuffdata(payload, &mut new_vec);

        payload.clear();
        *payload = new_vec.clone();

        Ok(())
    }
}

impl _Decode for COBS {
    fn _decode(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        println!("Running the inner Decode method!!!\n");
        let mut new_vec: Vec<u8> = Vec::new();
        unstuffdata(payload, &mut new_vec);

        payload.clear();
        *payload = new_vec.clone();

        Ok(())
    }
}

fn stuffdata(src: &mut Vec<u8>, dst: &mut Vec<u8>) -> Result<(),()> {

    let src_len = src.len();
    let mut search_len: u8 = 1;
    let mut dst_ptr: usize = 0;

    for i in 0..src_len{
        if src[i] == 0 {
            dst.push(search_len);
            search_len = 1;
            dst_ptr+=1;
        } else {
            search_len+=1;
            dst.push(src[i]);
            if search_len == 0xFF {
                dst.push(search_len);
                search_len = 1;
            }
        }
    }
    dst.push(search_len);
    Ok(())
}


fn unstuffdata(src: &mut Vec<u8>, dst: &mut Vec<u8>) -> Result<(),()> {

  /*  let src_len = src.len();
    let mut search_len: u8 = 1;
    let mut dst_ptr: usize = 0;

    for i in 0..src_len{
        if src[i] == 0 {
            dst.push(search_len);
            search_len = 1;
            dst_ptr+=1;
        } else {
            search_len+=1;
            dst.push(src[i]);
            if search_len == 0xFF {
                dst.push(search_len);
                search_len = 1;
            }
        }
    }
    dst.push(search_len);*/
    Ok(())
}