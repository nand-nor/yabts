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

    for i in 0..src_len{
        if src[i] == 0 {
            dst.push(search_len);
            search_len = 1;
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


    let mut remaining_bytes: usize = 0;
    let mut src_byte: u8 = 0;
    let mut len_code: u8 = 0;

    let src_len = src.len();


    for i in 0..src_len{

        len_code = src[i];
        if len_code == 0 {
            break;
        }
        len_code -= 1;
        remaining_bytes = src_len - i;
        if len_code as usize > remaining_bytes {
            // TODO fix this dangerous game
            len_code = remaining_bytes as u8;
        }

       // for i in std::iter::range_step(len_code as usize, 0, -1){
        //for j in (0isize..len_code as isize).step_by(-1){
        for j in (0isize..len_code as isize).rev(){

        src_byte = src[j as usize];//*src_read_ptr++;
            if (src_byte == 0)
            {
                break;
            }
            dst.push(src_byte);
        }

        if (len_code != 0xFE)
        {

            dst.push(0);
        }
    }
    Ok(())
}