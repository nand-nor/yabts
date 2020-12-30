use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct MessageHeader {
    magic: u32,
    length: u16,
    code: u16,
}

impl MessageHeader {
    pub fn new(length: u16, code: u16) -> MessageHeader {
        MessageHeader {
            magic: super::MAGIC,
            length: length,
            code: code,
        }
    }

    pub fn default() -> MessageHeader {
        MessageHeader {
            magic: super::MAGIC,
            length: 0,
            code: 0,
        }
    }

    pub fn set(&mut self, length: u16, code: u16) {
        self.length = length;
        self.code = code;
    }

    pub fn get(&self) -> (u16, u16) {
        (self.length, self.code)
    }

    pub fn is_valid(&self) -> bool {
        self.magic == super::MAGIC
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    header: MessageHeader,
    payload: Vec<u8>,
}

impl Message {
    pub fn default() -> Message {
        Message {
            header: MessageHeader::default(),
            payload: Vec::new(),
        }
    }

    pub fn new(length: u16, code: u16, payload: Option<Vec<u8>>) -> Message {
        let mut message = Message::default();
        message.set_header(length, code);
        if let Some(payload) = payload {
            message.set_payload(payload);
        }
        message
    }

    pub fn payload_len(&self) -> usize {
        return self.payload.len();
    }

    pub fn get_payload(&self) -> Option<Vec<u8>> {
        if self.payload.len() != 0 {
            Some(self.payload.clone())
        } else {
            None
        }
    }

    pub fn get_header(&self) -> MessageHeader {
        self.header
    }

    fn set_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload.clone()
    }

    fn set_header(&mut self, length: u16, code: u16) {
        self.header.set(length, code);
    }
}
