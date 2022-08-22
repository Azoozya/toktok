#![allow(unused_imports)]

use crate::network::host::Host;
use crate::network::udp::Datagram;
use crate::network::network::Network;

use crate::message::tlv::TLV;
use crate::message::signal::Signal;
use crate::message::header::Header;

use crate::memory::shared_fifo::SharedFifo;

pub async fn handler(mut net: Network, mut dg: Datagram, /*mut tracing: Signal<Datagram>,*/ mut outcome: SharedFifo<Datagram,()>) -> std::io::Result<()> {
    let peer: Host = dg.src().unwrap();
    match dg.header() {
        Header::PING => { 
            dg.swap();
            dg.set_header(Header::PONG);
            outcome.push_notice(dg,()).await.ok();
        },
        
        // Sounds like a (re?)newcomer
        Header::HELLO => {
            net.insert(&peer);
            dg.swap();
            dg.set_header(Header::HELLO);
            outcome.push_notice(dg,()).await.ok();
        },

        Header::UNKNOWN => {
            /*let dg = Datagram::new( Some(peer), dg.data(), Some(net.local_addr()) );
            tracing.send(dg).await.ok();*/
        }
        _ => { 
            
        },
    };

    Ok(())
}    
 