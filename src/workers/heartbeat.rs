#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use tokio::time::{Duration,sleep};

use crate::network::udp::Datagram;
use crate::network::network::Network;

use crate::message::header::Header;
use crate::message::signal::Signal;

pub async fn heartbeater(net: Network, mut backbone: Signal<()>) -> Result<(), std::io::Error> {
    let ping: Datagram = Datagram::from( Header::PING );
    
    loop {
        // If received any data => stop the thread
        match backbone.try_recv() {
            Err(_) => { break; },
            Ok(data) => { 
                if let Some(_) = data {
                    break;
                } 
            }
        }

        net.multicast(ping.clone()).await;
        sleep(Duration::from_millis(500)).await;
        net.multicast(ping.clone()).await;
        sleep(Duration::from_millis(500)).await;
        net.multicast(ping.clone()).await;
        sleep(Duration::from_secs(5)).await;
    }

    Ok( backbone.close() )
}