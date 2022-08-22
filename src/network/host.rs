use std::net::{ToSocketAddrs, SocketAddr, IpAddr};
use serde::{Deserialize, Serialize, Deserializer, Serializer};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Host {
    sock: SocketAddr,
    ip: IpAddr,
    port: u16,
}

impl Host {
    pub fn new(sock: &str) -> Host {
        Host::try_from(sock).unwrap()
    }

    pub fn sock(&self) -> SocketAddr {
        self.sock.clone()
    }

    pub fn ip(&self) -> IpAddr {
        self.ip.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn local_addr(&self) -> String {
        format!("{}:{}",self.ip(),self.port())
    }

    /*pub fn to_socket_addrs(&self) ->  {
        std::iter::once(self.sock())
    }*/

}

impl TryFrom<&str> for Host {
    type Error = &'static str;
    fn try_from(sock: &str) ->  Result<Self, Self::Error> {
        match sock.parse::<SocketAddr>() {
            Ok(sock_addr) => Ok( Host::from(sock_addr) ),
            Err(_) => Err("Incorrect format, should be IPAddr:u16"),
        }
    }
}

impl From<SocketAddr> for Host {
    fn from(sock: SocketAddr) -> Self {
        let ip = sock.ip();
        let port = sock.port();
        Host { sock, ip, port }
    }
}

impl ToSocketAddrs for Host {
    type Iter= std::iter::Once<SocketAddr>;
    fn to_socket_addrs(&self) -> Result<std::iter::Once<SocketAddr>,std::io::Error> {
        Ok( std::iter::once( self.sock() ) )
    }
}

impl Serialize for Host {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.local_addr())
    }
}

impl<'de> Deserialize<'de> for Host {
    fn deserialize<D>(deserializer: D) -> Result<Host, D::Error>
        where D: Deserializer<'de>
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match Host::try_from(s) {
            Ok(host) => Ok(host),
            Err(err) => Err(serde::de::Error::custom(err)),
        }
    }
}