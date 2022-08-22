#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::VecDeque;

use crate::message::tlv::TLV;
use crate::network::host::Host;
use crate::message::header::Header;

#[derive(Debug,Clone)]
pub struct Datagram {
    src: Option<Host>,
    data: TLV,
    dst: Option<Host>,
}

impl Datagram {
    pub fn new(src: Option<Host>, data: TLV, dst: Option<Host>) -> Datagram {
        Datagram { src, data, dst }
    }

    pub fn src(&self) -> Option<Host> { 
        self.src 
    }

    pub fn header(&self) -> Header {
        self.data.header()
    }

    pub fn set_header(&mut self, header: Header) {
        self.data.set_header(header);
    }

    pub fn data(&self) -> TLV { 
        self.data.clone() 
    }

    pub fn dst(&self) -> Option<Host> { 
        self.dst 
    }

    pub fn to_bytes(&self) -> Vec<u8> { 
        Vec::from( self.data.to_bytes() ) 
    }

    pub fn from_bytes(src: Option<Host>, dg_bytes: Vec<u8>, dst: Option<Host>) -> Option<Datagram> { 
        if let Some(data) = TLV::from_bytes( VecDeque::from(dg_bytes) ) {
            Some(Datagram { src, data, dst })
        }
        else { None }
    }

    pub fn swap(&mut self) {
        let tmp: Option<Host>;
        
        if let Some(_) = self.src {
            tmp = self.src;
            self.src = self.dst;
            self.dst = tmp;
        }// else if src = None
        else if let Some(_) = self.dst {
            self.src = self.dst;
            self.dst = None;
        }
    
    }
}

impl From<TLV> for Datagram {
    fn from(data: TLV) -> Self {
        Datagram { src: None, data, dst: None }
    }
}

impl From<Header> for Datagram {
    fn from(header: Header) -> Self {
        Datagram { src: None, data: TLV::new(header,None).unwrap() , dst: None }
    }
}