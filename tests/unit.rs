/*
#[cfg(test)]
mod test {
    //edge cases: no compression
    //all compression
    //maximum possible compression
    //fail cases: invalid char input, invalid lengths, etc.

    use super::*;
    use std::str;

    #[test]
    fn check_compression() {
        let all_compress = yabts::simple::compress("aaaaabbbb");
        let no_compress = yabts::simple::compress("abcdefg");

        if no_compress.is_ok() || all_compress.is_ok() {
            let case1 = all_compress.unwrap();
            let case2 = no_compress.unwrap();

            let case1_string = unsafe {
                let res = str::from_utf8(case1.as_slice());
                if res.is_err() {
                    ""
                } else {
                    res.unwrap()
                }
            };

            let case2_string = unsafe {
                let res = str::from_utf8(case2.as_slice());
                if res.is_err() {
                    ""
                } else {
                    res.unwrap()
                }
            };

            println!("Compressed: {:?} and {:?}", case1_string, case2_string);
            assert_eq!(case1_string, "5a4b");
            assert_eq!(case2_string, "abcdefg");
        }
    }

}
*/
