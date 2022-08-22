#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[derive(Debug,PartialEq,Eq,Copy,Clone)]
pub enum Header {
    HELLO,
    MULTIPLE,
    PING,
    PONG,
    UNKNOWN,
}

impl Header {
    pub fn to_byte(&self) -> u8 {
        match self {
            Header::UNKNOWN => 0,
            Header::HELLO => 1,
            Header::PING => 2,
            Header::PONG => 4,
            Header::MULTIPLE => 63,
        }
    }

    pub fn from_byte(header_byte: u8) -> Header {
        return match header_byte {
            63 => Header::MULTIPLE,
            4 => Header::PONG,
            2 => Header::PING,
            1 => Header::HELLO,
            _ => Header::UNKNOWN,
        }
    }
}
