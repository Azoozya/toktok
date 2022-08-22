use tokio::net::UdpSocket;

use std::sync::{Arc, Mutex};
use std::net::IpAddr;
use std::collections::HashMap;

use crate::network::host::Host;
use crate::network::udp::Datagram;
use crate::message::tlv::TLV;
use crate::message::header::Header;

#[derive(Debug)]
pub struct Network {
    server: Arc<Option<Host>>,
    gateway: Arc<Host>,
    rx: Arc<UdpSocket>,
    tx: Arc<UdpSocket>,
    clients: Arc<Mutex<HashMap<IpAddr,Host>>>,
    broadcastable: Arc<bool>,
}

impl Network {
    pub fn new(sock: Arc<UdpSocket>, sock_tx: Option<Arc<UdpSocket>>,gateway: Host, server: Option<Host>) -> Network {
        let clients: Arc<Mutex<HashMap<IpAddr,Host>>> = Arc::new( Mutex::new( HashMap::new() ) );
        let broadcastable: bool = match sock.set_broadcast(true){
            Err(_) => false,
            Ok(_) => true,
        };

        match sock_tx {
            None => 
                Network { server: Arc::new(server), gateway: Arc::new(gateway), rx: Arc::clone(&sock), tx: sock , clients: clients, broadcastable: Arc::new(broadcastable)},
            Some(sock_tx) => 
                Network { server: Arc::new(server), gateway: Arc::new(gateway), rx: sock, tx: sock_tx , clients: clients, broadcastable: Arc::new(broadcastable)},
        }
    }

    pub async fn send_to(&self, dg: Datagram, override_dst: Option<Host> ) -> usize {

        let dst: Host = match override_dst {
            None => { match dg.dst() {
                None => { return 0; },
                Some(dst) => dst,
            } },
            Some(dst) => dst,
        };
        
        match self.tx.send_to( &dg.to_bytes(), dst.sock() ).await {
            Err(_) => { return 0; },
            Ok(len) => { return len; },
        }
    }
    
    pub async fn recv_from(&self) -> Result<Datagram, std::io::Error> {
        let mut buf: [u8; 1026] = [0; 1026]; 
        
        let (len, addr) = self.rx.recv_from(&mut buf).await?;

        let mut tmp_vec: Vec<u8> = Vec::from(buf);
        tmp_vec.truncate(len);
        
        match Datagram::from_bytes( Some( Host::from(addr) ), tmp_vec, None ) {
            None => Ok( Datagram::new( Some( Host::from(addr) ), TLV::new(Header::UNKNOWN,None).unwrap(), None ) ),
            Some(dg) => Ok(dg),
        }
    }

    pub async fn multicast(&self,dg: Datagram) {
        let clients: HashMap<IpAddr,Host>;
        
        {
            clients = self.clients.lock().unwrap().clone();
        }

        for (_,client) in clients.iter() {
            self.send_to( dg.clone(), Some(*client) ).await;
        }
    }

    // Use the gateway to JOIN the network, in decentralized networks server != gateway
    pub async fn broadcast(&self,dg: Datagram) {
        if *self.broadcastable {
            self.send_to( dg, Some(*self.gateway) ).await;
        }
    }

    pub fn contains(&self, client: &Host) -> bool {
        self.clients.lock().unwrap().contains_key(&client.ip())
    }

    pub fn insert(&mut self, client: &Host) -> bool {
        if self.contains(&client) {
            false
        }
        else {
            self.clients.lock().unwrap().insert( client.ip(), client.clone() );
            true
        }
    }

    pub fn remove(&mut self, client: &Host) -> bool {
        match  self.clients.lock().unwrap().remove( &client.ip() ) {
            None => false,
            Some(_) => true,
        }
    }

    pub fn clone(&self) -> Self {
        Self { 
            server: Arc::clone(&self.server), 
            gateway: Arc::clone(&self.gateway),
            rx: Arc::clone(&self.rx),
            tx: Arc::clone(&self.tx),
            clients: Arc::clone(&self.clients),
            broadcastable: Arc::clone(&self.broadcastable),
        }
    }

    pub fn local_addr(&self) -> Host {
        Host::from( self.rx.local_addr().unwrap() )
    }
}


// For test
async fn echo_server(port: u16) -> std::io::Result<()> {
    let sock = Arc::new( UdpSocket::bind(format!("127.0.0.1:{}",port)).await? );
    let gateway = Host::new( &format!("127.255.255.255:{}",port) );
    let net: Network = Network::new(sock, None, gateway, None);

    let dg: Datagram = net.recv_from().await?;
    println!("Received: {:#?}", dg);

    let dg = Datagram::new(None, dg.data(), dg.src() );
    net.send_to( dg , None ).await;

    Ok(())
}

// For test
async fn echo_client(port: u16, srv: u16) -> std::io::Result<(TLV,TLV)> {
    let gateway: Host = Host::new(  &format!("127.0.0.1:{}",srv) );
    let dst: Host = Host::new(  &format!("127.0.0.1:{}",srv) );
    let data: TLV = TLV::new( Header::UNKNOWN, None).unwrap();
    let to_send: Datagram = Datagram::new( None, data, Some( dst ) );

    let src = Arc::new( UdpSocket::bind( format!("127.0.0.1:{}",port) ).await? );
    let net: Network = Network::new(src, None, gateway, None);
       
    // Sending data
    net.send_to( to_send.clone() , None ).await;

    // Waiting for the echo
    let received: Datagram = net.recv_from().await?;
    
    Ok( ( to_send.data(), received.data() ) )
}

#[tokio::test]
async fn test_echo() {
    let srv: u16 = 8081;
    let s = tokio::task::spawn( echo_server(srv) );
    let c = tokio::task::spawn( echo_client(4242,srv) );

    if let Ok(Ok((input,output))) = c.await {
        assert_eq!(input,output);
        return;
    }
    panic!();
}

// Kind of difficult to assert something cuz datagrams are not retrieved -> manually use tcpdump
#[tokio::test]
async fn test_multicast() -> std::io::Result<()>{
    let sock = Arc::new( UdpSocket::bind( "127.0.0.1:4445" ).await? );
    let mut net: Network = Network::new(sock, None, Host::new("127.255.255.255:4445"),None);

    let one: Host = Host::new("127.0.0.1:1111");
    let two: Host = Host::new("127.0.0.2:2222");
    let three: Host = Host::new("127.0.0.3:3333");

    net.insert(&one);
    net.insert(&two);
    net.insert(&three);

    let dg: Datagram = Datagram::from( TLV::new(Header::UNKNOWN, Some(vec![6,6,6])).unwrap() );

    net.multicast(dg).await;
    Ok(())
}

// Kind of difficult to assert something cuz datagrams are not retrieved -> manually use tcpdump
#[tokio::test]
async fn test_broadcast() -> std::io::Result<()>{
    let sock = Arc::new( UdpSocket::bind( "127.0.0.1:4040" ).await? );
    let net: Network = Network::new(sock, None, Host::new("127.255.255.255:4040"),None);
    let dg: Datagram = Datagram::from( TLV::new(Header::UNKNOWN, Some(vec![1,8,1])).unwrap() );

    net.broadcast(dg).await;
    Ok(())
}

#[tokio::test]
async fn test_insert_contains_remove() -> std::io::Result<()>{
    let sock = Arc::new( UdpSocket::bind( "127.0.0.1:4444" ).await? );
    let mut net: Network = Network::new(sock, None, Host::new("127.255.255.255:4444"),None);

    let one: Host = Host::new("127.0.0.1:1111");
    let two: Host = Host::new("127.0.0.2:2222");
    let three: Host = Host::new("127.0.0.3:3333");

    // Should be inserted successfully, currently the key of the map is the IP i.e can't use same ip for multiple clients
    // It's a choice cuz same device using different ports should be part of different networks
    assert_eq!(true,net.insert(&one));
    assert_eq!(true,net.insert(&two));
    assert_eq!(true,net.insert(&three));

    // Already inserted
    assert_ne!(true,net.insert(&one));

    // Should be removed successfully
    assert_eq!(true,net.remove(&two));
    
    // Already removed
    assert_ne!(true,net.contains(&two));
    
    Ok(())
}

