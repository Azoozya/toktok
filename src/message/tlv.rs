use std::collections::VecDeque;

use crate::message::header::Header;

#[derive(Debug,PartialEq,Eq,Clone)]
pub struct TLV {
    header: Header,
    length: u16,
    payload: Vec<u8>,
    mergeable: bool,
}

impl TLV {

    pub fn set_header(&mut self, header: Header) {
        self.header = header;
    }

    pub fn header(&self) -> Header {
        self.header
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn payload(&self) -> Vec<u8> {
        self.payload.clone()
    }

    pub fn mergeable(&self) -> bool {
        self.mergeable
    }

    // Clone data ?
    pub fn new(header: Header, data: Option<Vec<u8>>) -> Option<TLV> {
        let length: u16;
        let payload: Vec<u8>;

        match data {
            None => {
                length = 0;
                payload = Vec::new(); 
            }
            Some(data) => {
                length = data.len() as u16;
                if length > 1024 {
                    return None;
                }
                payload = data.clone();
            }
        }

        Some( TLV { header, length, payload, mergeable: (length <= 1020) } )
    }

    pub fn to_bytes(&self) -> VecDeque<u8> {
        let mut tl: u16 = (self.header.to_byte()) as u16 * 1024 ;
        let mut v: VecDeque<u8> = VecDeque::from( self.payload.clone() );

        if self.length < 1024 {
            tl += self.length;
            v.truncate(self.length as usize);
        }

        let mut tlv: VecDeque<u8> = VecDeque::new();
        tlv.push_back((tl >> 8) as u8);
        tlv.push_back((tl % 256) as u8);
        tlv.append(&mut v);
        
        tlv
    }

    pub fn from_bytes( mut tlv_bytes: VecDeque<u8> ) -> Option<TLV> {
        let len = tlv_bytes.len();

        if len < 2 {
            return None;
        }

        let tl: u16 = 
            (tlv_bytes.pop_front()).unwrap() as u16 * 256 + 
            (tlv_bytes.pop_front()).unwrap() as u16;
        

        let header: Header =  Header::from_byte((tl >> 10) as u8); 
        let mut length: u16 = tl % 1024;

        // 1024 = 2^10 i.e overflow our counter
        if length == 0 && len == 1026 {
            length = 1024;
        }
        // UDP doesn't work with frames, it's all in 1 TLV
        else if length as usize != (len-2) {   
            return None;
        }

        let payload = Vec::from( tlv_bytes );

        Some( TLV { header, length, payload, mergeable: (length <= 1020) } )
    }

    pub fn split(&self) -> Option<Vec<TLV>> {
        let mut result: Vec<TLV> = Vec::new();
        
        match self.header {
            Header::MULTIPLE => {
                let mut cursor: usize = 0;
                
                let mut tmp_dg: Option<TLV>;
                let mut tl: u16;
                let mut head: Header;
                let mut len: usize;
                while cursor < self.length as usize  {
                    tl = 
                        self.payload[cursor] as u16 * 256 + 
                        self.payload[cursor+1] as u16;
                    
                    // pass the overhead
                    cursor += 2;
                    
                    // to be part of a MULTIPLE data size must be less than 1020 i.e if len = 0 % 1024 => len = 0     
                    len = tl as usize % 1024;
                    head = Header::from_byte( (tl >> 10) as u8);
                    
                    match len {
                        0 => {
                            tmp_dg = TLV::new(
                                head, 
                                None
                            );
                        }
                        _ => {
                            tmp_dg = TLV::new(
                                head, 
                                Some(
                                    self.payload[(cursor)..(cursor+len)].to_vec()
                                ) 
                            );
                        }
                    }

                    match tmp_dg {
                        None => {
                            // error in one part of MULTIPLE 
                            return None;
                        },
                        Some(dg) => {
                            result.push(dg);
                        },
                    }
                    // pass the data
                    cursor += len;
                }
            },
            _ => {
                result.push(self.clone());
            },
        }

        Some(result)
    }

    pub fn merge(left: TLV,right: TLV)-> Option<TLV> {
        // check if left is meargeable
        if !left.mergeable() {
            return None;
        }
        
        /* ############################################## */
        
        // if left/right is a multiple, we don't copy the header
        let left_len: u16;
        let left_header: Header = left.header();

        match left_header {
            Header::MULTIPLE => {
                left_len = left.length();
            }
            _ => {
                left_len = left.length() + 2;
            }
        }

        let right_len: u16;
        let right_header: Header = right.header();

        match right_header {
            Header::MULTIPLE => {
                right_len = right.length();
            }
            _ => {
                right_len = right.length() + 2;
            }
        }

        // check if sum of length is included in [0;1024]
        if !(left_len + right_len <= 1024) {
            return None;
        }

        /* ############################################## */

        // if left/right is a multiple, we don't copy the header
        let mut left_data: VecDeque<u8> = left.to_bytes();
        if let Header::MULTIPLE = left_header {
            //we don't copy the header
            left_data.pop_front();
            left_data.pop_front();
        }

        let mut right_data: VecDeque<u8> = right.to_bytes();
        if let Header::MULTIPLE = right_header {
            //we don't copy the header
            right_data.pop_front();
            right_data.pop_front();
        }

        let mut data: Vec<u8> = Vec::from(left_data);
        data.append(&mut Vec::from(right_data));

        TLV::new(Header::MULTIPLE,Some(data))
    }

}



#[test]
fn test_new_overflow() {
    let dg = TLV::new(Header::UNKNOWN,Some(std::vec![1; 1025]));
    assert_eq!(None,dg);
}

#[test]
fn test_from_to_empty() {
    /* with empty payload */
    let dg = TLV::new(Header::UNKNOWN,None);
    assert_ne!(None,dg);
    let dg = dg.unwrap();

    let dgg = dg.to_bytes();
    assert_eq!(0,dgg[0]);
    assert_eq!(0,dgg[1]);


    let dggg = TLV::from_bytes(dgg);
    assert_ne!(None,dggg);
    let dggg = dggg.unwrap();
    assert_eq!(dggg,dg);
}

#[test]
fn test_from_to_not_empty() {
    /* with not empty payload */
    let dg = TLV::new(Header::UNKNOWN,Some(std::vec![1; 512]));
    assert_ne!(None,dg);
    let dg = dg.unwrap();

    let dgg = dg.to_bytes();
    assert_eq!(2,dgg[0]);
    assert_eq!(0,dgg[1]);


    let dggg = TLV::from_bytes(dgg);
    assert_ne!(None,dggg);
    let dggg = dggg.unwrap();
    assert_eq!(dggg,dg);
}

#[test]
fn test_from_to_full() {
    /* with not empty payload */
    let dg = TLV::new(Header::UNKNOWN,Some(std::vec![1; 1024]));
    assert_ne!(None,dg);
    let dg = dg.unwrap();

    let dgg = dg.to_bytes();
    assert_eq!(0,dgg[0]);
    assert_eq!(0,dgg[1]);


    let dggg = TLV::from_bytes(dgg);
    assert_ne!(None,dggg);
    let dggg = dggg.unwrap();
    assert_eq!(dggg,dg);
}

#[test]
fn test_from_incorrect_length() {
    let mut dg = TLV::new(Header::UNKNOWN,Some(std::vec![1; 512])).unwrap().to_bytes();
    
    // test if you add data
    dg.push_back(2_u8);
    let dgg = TLV::from_bytes(dg.clone());
    assert_eq!(None,dgg);

    // test if you alter length
    dg.pop_back();
    dg.pop_front();
    dg.push_front(1_u8);
    let dggg = TLV::from_bytes(dg);
    assert_eq!(None,dggg);
}

#[test]
fn test_merge_one_to_one() {
    let left: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right: TLV = TLV::new(Header::UNKNOWN,Some(vec![1])).unwrap();

    let merged: TLV = TLV::merge(left,right).unwrap();
    let merged_bytes = merged.to_bytes();

    assert_eq!(merged.header(),Header::MULTIPLE);
    assert_eq!(merged.length(),5);

    assert_eq!(merged_bytes[0],252);
    assert_eq!(merged_bytes[1],5);
    assert_eq!(merged_bytes[2],0);
    assert_eq!(merged_bytes[3],0);
    assert_eq!(merged_bytes[4],0);
    assert_eq!(merged_bytes[5],1);
    assert_eq!(merged_bytes[6],1);
}

#[test]
fn test_merge_one_to_n() {
    let left: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right_0: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right_1: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right: TLV = TLV::merge(right_0,right_1).unwrap();

    let merged: TLV = TLV::merge(left,right).unwrap();
    let merged_bytes = merged.to_bytes();

    assert_eq!(merged.header(),Header::MULTIPLE);
    assert_eq!(merged.length(),6);

    assert_eq!(merged_bytes[0],252);
    assert_eq!(merged_bytes[1],6);
    assert_eq!(merged_bytes[2],0);
    assert_eq!(merged_bytes[3],0);
    assert_eq!(merged_bytes[4],0);
    assert_eq!(merged_bytes[5],0);
    assert_eq!(merged_bytes[6],0);
    assert_eq!(merged_bytes[7],0);
}

#[test]
fn test_merge_n_to_one() {
    let left_0: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let left_1: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let left: TLV = TLV::merge(left_0,left_1).unwrap();
    let right: TLV = TLV::new(Header::UNKNOWN,None).unwrap();

    let merged: TLV = TLV::merge(left,right).unwrap();
    let merged_bytes = merged.to_bytes();

    assert_eq!(merged.header(),Header::MULTIPLE);
    assert_eq!(merged.length(),6);

    assert_eq!(merged_bytes[0],252);
    assert_eq!(merged_bytes[1],6);
    assert_eq!(merged_bytes[2],0);
    assert_eq!(merged_bytes[3],0);
    assert_eq!(merged_bytes[4],0);
    assert_eq!(merged_bytes[5],0);
    assert_eq!(merged_bytes[6],0);
    assert_eq!(merged_bytes[7],0);
}

#[test]
fn test_merge_n_to_n() {
    let left_0: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let left_1: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let left: TLV = TLV::merge(left_0,left_1).unwrap();

    let right_0: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right_1: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right: TLV = TLV::merge(right_0,right_1).unwrap();

    let merged: TLV = TLV::merge(left,right).unwrap();
    let merged_bytes = merged.to_bytes();

    assert_eq!(merged.header(),Header::MULTIPLE);
    assert_eq!(merged.length(),8);

    assert_eq!(merged_bytes[0],252);
    assert_eq!(merged_bytes[1],8);
    assert_eq!(merged_bytes[2],0);
    assert_eq!(merged_bytes[3],0);
    assert_eq!(merged_bytes[4],0);
    assert_eq!(merged_bytes[5],0);
    assert_eq!(merged_bytes[6],0);
    assert_eq!(merged_bytes[7],0);
    assert_eq!(merged_bytes[8],0);
    assert_eq!(merged_bytes[9],0);
}

#[test]
fn test_merge_overflow_left() {
    let left: TLV = TLV::new(Header::UNKNOWN,Some(vec![0; 1021])).unwrap();
    let right: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let merged: Option<TLV> = TLV::merge(left,right);
    assert_eq!(merged,None);
}

#[test]
fn test_merge_overflow_right() {
    let left: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right: TLV = TLV::new(Header::UNKNOWN,Some(vec![0; 1021])).unwrap();
    let merged: Option<TLV> = TLV::merge(left,right);
    assert_eq!(merged,None);
}

#[test]
fn test_split_empty() {
    let left_0: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let left_1: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let left: TLV = TLV::merge(left_0,left_1).unwrap();

    let right_0: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right_1: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right: TLV = TLV::merge(right_0,right_1).unwrap();

    let merged: TLV = TLV::merge(left,right).unwrap();

    let splited: Vec<TLV> = merged.split().unwrap();
    for dg in splited.iter() {
        assert_eq!(dg.header(),Header::UNKNOWN);
        assert_eq!(dg.length(),0);
    }
}

#[test]
fn test_split_not_empty() {
    let left_0: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let left_1: TLV = TLV::new(Header::UNKNOWN,Some(vec![1,2,3])).unwrap();
    let left: TLV = TLV::merge(left_0,left_1).unwrap();

    let right_0: TLV = TLV::new(Header::UNKNOWN,Some(vec![4,5])).unwrap();
    let right_1: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right: TLV = TLV::merge(right_0,right_1).unwrap();

    let merged: TLV = TLV::merge(left,right).unwrap();

    let splited: Vec<TLV> = merged.split().unwrap();
    assert_eq!(splited.len(),4);
}

#[test]
fn test_split_full() {
    let left_0: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let left_1: TLV = TLV::new(Header::UNKNOWN,Some(vec![0; 508])).unwrap();
    let left: TLV = TLV::merge(left_0,left_1).unwrap();

    let right_0: TLV = TLV::new(Header::UNKNOWN,Some(vec![0; 508])).unwrap();
    let right_1: TLV = TLV::new(Header::UNKNOWN,None).unwrap();
    let right: TLV = TLV::merge(right_0,right_1).unwrap();

    let merged: TLV = TLV::merge(left,right).unwrap();

    let splited: Vec<TLV> = merged.split().unwrap();
    assert_eq!(splited.len(),4);
}