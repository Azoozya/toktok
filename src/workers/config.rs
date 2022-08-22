use std::fs;
use std::net::IpAddr;
use std::collections::HashMap;

use crate::crypto::asymetric::KeyPair;

use ssh_key::Signature;
use signature::{Verifier,Signer};

use serde::{Deserialize, Serialize};

use crate::network::{host::Host,network::Network,service::Service};

#[derive(Debug,Clone,Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    server: Option<Host>,
    gateway: Host,
    rx: Host,
    tx: Option<Host>,
    clients: Option<HashMap<IpAddr,Host>>,
    services: Option<Vec<Service>>,
    signature: Option<Vec<u8>>,
}


#[derive(Debug, PartialEq, Eq)]
pub enum ConfigErr {
    FileReadingError,
    FileWritingError,
    DeserializingError,
    SerializingError,
    BindingRxError,
    BindingTxError,
    UnableToReadSignature,
    InvalidSignature,
}

impl Config {

    fn set_signature(&mut self,signature: Option<&Signature>) {
        self.signature = match signature.clone(){
            None => None,
            Some(signature) => Some(Vec::from(signature.as_bytes())),
        };
    }

    pub fn from_file(filename: &str) -> Result<Config,ConfigErr> {
        match fs::read_to_string(filename) {
            Err(_) => Err( ConfigErr::FileReadingError ),
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Err(_) => Err( ConfigErr::DeserializingError ),
                    Ok(config) => Ok(config),
                }
            },
        }
    }

    pub fn into_file(&self, filename: &str) -> Result<(),ConfigErr> {
        match serde_json::to_string(self) {
            Err(_) => Err( ConfigErr::SerializingError ),
            Ok(ma) => {
                match fs::write(filename, ma) {
                    Err(_) => Err( ConfigErr::FileWritingError ),
                    Ok(_) => Ok(()),
                }
            },
        }
    }

    pub async fn into_network(&self) -> Result<Network,ConfigErr> {
        let sock = match tokio::net::UdpSocket::bind( self.rx.sock() ).await {
            Err(_) => { return Err( ConfigErr::BindingRxError ); },
            Ok(sock) => std::sync::Arc::new(sock),
        };
        
        let sock_tx = match self.tx {
            None => None,
            Some(host) => {
                match tokio::net::UdpSocket::bind( host.sock() ).await {
                    Err(_) => { return Err( ConfigErr::BindingTxError ); },
                    Ok(sock_tx) => Some(std::sync::Arc::new(sock_tx)),
                } 
            },
        };
        

        Ok( Network::new(sock,sock_tx,self.gateway,self.server) )
    }

    pub fn verify(&mut self,key: &KeyPair) -> Result<(),ConfigErr> {
        if self.signature.is_none() {
            return Err( ConfigErr::InvalidSignature );
        }
        
        let signature = match Signature::new(
                key.algorithm(),
                self.signature.clone().unwrap()
            ) {
            Err(_) => { return Err(ConfigErr::UnableToReadSignature); },
            Ok(signature) => signature,
        };

        self.set_signature(None);
        let data = serde_json::to_vec(self).unwrap();
        self.set_signature(Some(&signature));

        match key.verify(&data,&signature) {
            Err(_) => Err(ConfigErr::InvalidSignature),
            Ok(()) => Ok(()),
        }
    }

    pub fn sign(&mut self,key: &KeyPair) -> Result<(),ConfigErr> {
        let if_fail = self.signature.clone();

        self.set_signature(None);
        let data = serde_json::to_vec(self).unwrap();

        let signature = key.try_sign(&data);
        match signature {
            Ok(signature) => {
                self.set_signature( Some( &signature ) );
                Ok(())
            },
            Err(err) => { 
                self.signature = if_fail;
                Err( ConfigErr::InvalidSignature )
            }
        }
    }
}

#[test]
fn test_serde() -> serde_json::Result<()> {

    let c = Config {
        server: Some( Host::new( "127.0.0.1:1111" ) ),
        gateway: Host::new( "127.0.0.1:22222" ),
        rx: Host::new( "127.0.0.1:3333" ),
        tx: Some(Host::new( "127.0.0.1:4444" )),
        clients: None,
        services: None,
        signature: None,
    };

    // Serialize it to a JSON string.
    let j = serde_json::to_string(&c)?;
    let cc = serde_json::from_str(&j)?;

    assert_eq!(c,cc);
    Ok(())
}

#[test]
fn test_sign_verify_from_into() {
    let c = Config {
        server: Some( Host::new( "127.0.0.1:3333" ) ),
        gateway: Host::new( "127.255.255.255:3333" ),
        rx: Host::new( "127.0.0.1:3334" ),
        tx: Some(Host::new( "127.0.0.1:3335" )),
        clients: None,
        services: None,
        signature: None,
    };

    let s = Config {
        server: None,
        gateway: Host::new( "127.255.255.255:3333" ),
        rx: Host::new( "127.0.0.1:3333" ),
        tx: None,
        clients: None,
        services: None,
        signature: None,
    };

    let key = crate::crypto::openssh::from(
        "toktok".to_string(),
        Some(
            "lama".to_string()
        )
    );
    assert!(key.is_ok());
    let key = key.unwrap();
    let mut c_signed = c.clone();
    let mut s_signed = s.clone();

    assert!( c_signed.sign(&key).is_ok() );
    assert!( c_signed.verify(&key).is_ok() );

    assert!( s_signed.sign(&key).is_ok() );
    assert!( s_signed.verify(&key).is_ok() );

    assert!(c_signed.into_file("client.config").is_ok() );
    assert_eq!(Ok(c_signed),Config::from_file("client.config"));
    assert!(s_signed.into_file("server.config").is_ok() );
    assert_eq!(Ok(s_signed),Config::from_file("server.config"));
}

/*
{
    "server":"127.0.0.1:1111",
    "gateway":"127.0.0.1:22222",
    "rx":"127.0.0.1:3333",
    "tx":"127.0.0.1:4444",
    "clients":null,
    "signature":null
}

"services":[
    "lama":"1111"
]
*/