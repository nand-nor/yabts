pub struct ServerStats {
    bytes_sent: u64,     //all bytes sent, including headers
    bytes_received: u64, //all bytes received, including headers

    compress_sent: u64, //all -valid- bytes sent (excludes invalid compression requests)
    compress_rcv: u64,  //all -valid- bytes received (excludes invalid compression requests)

    decompress_sent: u64, //all -valid- bytes sent (excludes invalid decompression requests)
    decompress_rcv: u64,  //all -valid- bytes received (excludes invalid decompression requests)

    compression_ratio: u8, //ratio of all uncompressed bytes (excluding headers) received versus compressed size
    //space_saving: u8,       // 1 - uncompressed/compressed
    encode_sent: u64,
    encode_rcv: u64,

    decode_sent: u64,
    decode_rcv: u64,
}

impl ServerStats {
    pub fn new() -> ServerStats {
        ServerStats {
            bytes_sent: 0,
            bytes_received: 0,
            compress_sent: 0,
            compress_rcv: 0,
            compression_ratio: 0,
            decompress_sent: 0,
            decompress_rcv: 0,
            encode_sent: 0,
            encode_rcv: 0,
            decode_sent: 0,
            decode_rcv: 0,
        }
    }

    pub fn get_stats(&self) -> (u64, u64, u8) {
        (self.bytes_sent, self.bytes_received, self.compression_ratio)
    }

    //ensure safe addition-- if overflow will occur, do wrapping add
    pub fn add_compression_data(&mut self, rcv: u64, sent: u64) {
        if self.compress_sent.checked_add(sent).is_some() {
            self.compress_sent += sent;
        } else {
            self.compress_sent = self.compress_sent.wrapping_add(sent);
        }
        //ensure no overflow -- check add and if not safe then wrapping add
        if self.compress_rcv.checked_add(rcv).is_some() {
            self.compress_rcv += rcv;
        } else {
            self.compress_rcv = self.compress_rcv.wrapping_add(rcv);
        }
        //only update this when we compress data
        //ensure safe division & safe multiplication
        if self.compress_rcv.checked_div(self.compress_sent).is_some() {
            self.compression_ratio =
                ((self.compress_rcv as f64 / self.compress_sent as f64) * 100 as f64) as u8;
        } else {
            //retain the old ratio
        }
    }

    //reset stats
    pub fn reset_stats(&mut self) {
        self.bytes_sent = 0;
        self.bytes_received = 0;
        self.compress_rcv = 0;
        self.compress_sent = 0;
        self.compression_ratio = 0;
    }

    //safely update the server's inner stats
    pub fn update_stats(&mut self, update_sent: u64, update_recv: u64) {
        //ensure no overflow -- check add and if not safe then wrapping add
        if self.bytes_sent.checked_add(update_sent).is_some() {
            self.bytes_sent += update_sent;
        } else {
            self.bytes_sent = self.bytes_sent.wrapping_add(update_sent);
        }
        //ensure no overflow -- check add and if not safe then wrapping add
        if self.bytes_received.checked_add(update_recv).is_some() {
            self.bytes_received += update_recv;
        } else {
            self.bytes_received = self.bytes_received.wrapping_add(update_recv);
        }
    }
}
