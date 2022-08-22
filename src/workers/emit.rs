use crate::network::udp::Datagram;
use crate::network::network::Network;

use crate::message::signal::Signal;

use crate::memory::shared_fifo::SharedFifo;

pub async fn emitter(net: Network, mut outcome: SharedFifo<Datagram,()>, mut backbone: Signal<()>) -> std::io::Result<()> {  
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

        match outcome.notified() {
            Err(err) => {
                break;
            },
            Ok(future) => {
                future.await;
                let mut dg: Datagram;
                let mut maybe_dg = outcome.pop();
                while maybe_dg.is_some() {
                    dg = maybe_dg.unwrap();
                    //    
                    net.send_to(dg,None).await;
                    //
                    maybe_dg = outcome.pop();
                }
            }
        }
    }
    
    outcome.close();
    Ok( backbone.close() )
}

#[tokio::test]
async fn test_emitter() -> std::io::Result<()>{

    let dg = Datagram::from(crate::message::header::Header::HELLO);
    let i_max = 100;

    let (_, _, income, outcome) = crate::server_and_client(dg, i_max,2).await.unwrap();

    assert!(income.pop().is_none());
    assert!(outcome.pop().is_none());
    Ok(())
}