use std::io; //error::Error;
//use std::iter::Iterator;
use yabts::transform::*;

//use std::str;

use std::slice;
use std::str;

#[derive(Copy, Clone)]
pub struct Simple {}

/// The simple compression algorithm cannot handle uppercase
/// letters, numbers, or punctuation
///
/// The algorithm compresses a given string by first converting to ascii utf8 encoding, ensuring
/// that this is valid then back to raw bytes and iterating recursively over each repeated substring.
/// To perform compression it iterates through a string by recursing over each repeating char
/// substring and returning the count via a helper function.
///
/// For example, a string `aaabbbccc` would be compressed to 3a3b3c
///
/// If any error condition met, return Err(), else return with OK(())
///
/// To do: make this more memory efficient
impl _Compress for Simple {
    fn _compress(&self, payload: &mut Vec<u8>) -> Result<(), io::Error> {
        //Make sure that the payload has a valid UTF8 encoding
        let ascii_slice = str::from_utf8(payload.as_slice());
        if ascii_slice.is_ok() {
            let ascii_slice = ascii_slice.unwrap();
            let length = ascii_slice.len();
            let ptr = ascii_slice.as_ptr();
            let slice = unsafe {
                // build a &[u8]
                slice::from_raw_parts(ptr, length)
            };

            let mut counts: Vec<(usize, u8)> = Vec::new(); //vec![0usize, length, length];

            //recursively count each repeating substring
            //reject string if anything other than lowercase chars
            // (numeric values, punctuation, uppercase) detected
            if count(slice, &mut counts, length).is_err() {
                return Err(io::Error::new(io::ErrorKind::Other, "Invalid string"));
            }

            //build a new vector of bytes from the returned counts and chars of each
            //repeating substring
            let mut new_string_vec: Vec<u8> = Vec::new();
            for i in 0..counts.len() {
                let (count, char_byte) = counts[i];
                match count {
                    1 => {
                        //dont compress
                        new_string_vec.push(char_byte);
                    }
                    2 => {
                        //dont compress
                        new_string_vec.push(char_byte);
                        new_string_vec.push(char_byte);
                    }
                    _ => {
                        //if count is single digit, convert to a char
                        let count_char = std::char::from_digit(count as u32, 10);
                        if count_char.is_some() {
                            let count_char = count_char.unwrap();
                            new_string_vec.push(count_char as u8);
                            new_string_vec.push(char_byte);
                        } else {
                            //count is  double to quadruple digit, so must get
                            // the individual ascii values for each byte individually
                            // by converting it to a string, then indexing it as a slice
                            let chars = count.to_string();
                            let length = chars.len();
                            let ptr = chars.as_ptr();
                            let char_slice = unsafe { slice::from_raw_parts(ptr, length) };
                            if count >= 10 && count < 100 {
                                //push the individual bytes
                                new_string_vec.push(char_slice[0] as u8);
                                new_string_vec.push(char_slice[1] as u8);
                                new_string_vec.push(char_byte as u8);
                            } else if count >= 100 && count < 1000 {
                                new_string_vec.push(char_slice[0] as u8);
                                new_string_vec.push(char_slice[1] as u8);
                                new_string_vec.push(char_slice[2] as u8);
                                new_string_vec.push(char_byte as u8);
                            } else if count >= 1000 && count < super::MAX_MSG_LEN {
                                new_string_vec.push(char_slice[0] as u8);
                                new_string_vec.push(char_slice[1] as u8);
                                new_string_vec.push(char_slice[2] as u8);
                                new_string_vec.push(char_slice[3] as u8);
                                new_string_vec.push(char_byte as u8);
                            } else {
                                //invalid string
                                return Err(io::Error::new(io::ErrorKind::Other, "Invalid string"));
                            }
                        }
                    }
                }
            }

            payload.clear();
            *payload = new_string_vec.clone();
            return Ok(());
        }
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Cannot decode as utf8",
        ));
    }
}

impl _Decompress for Simple {
    fn _decompress(&self, _payload: &mut Vec<u8>) -> Result<(), io::Error> {
        Ok(())
    }
}

/// Iterate over a substring and return the count
pub fn count(slice: &[u8], counts: &mut Vec<(usize, u8)>, length: usize) -> Result<(), ()> {
    let mut index = 0;

    while index < length {
        if (slice[index] =< 122 && slice[index] >= 97) ||
            (slice[index] =< 90 && slice[index] >= 41) {
            //EOF and \n are not considered invalid if at end of input
            if index == length - 1 && slice[index] == 0xa {
                break;
            } else {
                return Err(());
            }
        }
        //recurse over the string until a new char is encountered, returning the count of the
        //repeating substring
        let char_count = helper(slice, slice[index], index, length);
        //push a tuple to the counts vector: the count and the char value
        counts.push((char_count, slice[index]));
        //increment the index
        if char_count == 0 {
            index += 1;
        } else {
            index += char_count;
        }
    }
    Ok(())
}

///Recursive helper function. Obtain counts of all chars in the string
pub fn helper(slice: &[u8], char_byte: u8, index: usize, length: usize) -> usize {
    //base cases: string has been iterated through or char vals not equivalent
    if index == length || slice[index] != char_byte {
        return 0;
    } else if slice[index] == char_byte {
        return helper(slice, char_byte, index + 1, length) + 1;
    } else {
        return 0;
    }
}
