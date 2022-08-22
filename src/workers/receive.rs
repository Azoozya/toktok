use crate::network::udp::Datagram;
use crate::network::network::Network;

use crate::message::signal::Signal;

use crate::memory::shared_fifo::SharedFifo;

pub async fn receiver(net: Network, mut income: SharedFifo<Datagram,()>, mut backbone: Signal<()>) -> std::io::Result<()> {  
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
        
        if let Ok(dg) = net.recv_from().await {
            if let Err(_) = income.push_notice(dg,()).await {
                break;
            };
        }
    }
    
    income.close();
    Ok( backbone.close() )
}


#[tokio::test]
async fn test_receiver_hello() -> std::io::Result<()> {
    
    let dg = Datagram::from(crate::message::header::Header::HELLO);
    let i_max = 100;

    match crate::server_and_client(dg, i_max,0).await {
        Err(_) => { assert!(false); },
        Ok((_, _, income, outcome)) => {

            for _ in 0..i_max {
                assert_eq!(crate::message::header::Header::HELLO,income.pop().unwrap().data().header());
            }
        
            assert!(income.pop().is_none());
            assert!(outcome.pop().is_none());
        }
    }
    
    Ok(())
}

// transformed into pong by dispatcher
#[tokio::test]
async fn test_receiver_ping() -> std::io::Result<()> {
    
    let dg = Datagram::from(crate::message::header::Header::PING);
    let i_max = 100;

    match crate::server_and_client(dg, i_max,0).await {
        Err(_) => { assert!(false); },
        Ok((_, _, income, outcome)) => {
            for _ in 0..i_max {
                assert_eq!(crate::message::header::Header::PING,income.pop().unwrap().data().header());
            }
        
            assert!(income.pop().is_none());
            assert!(outcome.pop().is_none());
        }
    }
    Ok(())
}

// unknown are filtered by dispatcher
#[tokio::test]
async fn test_receiver_unknown() -> std::io::Result<()> {
    
    let dg = Datagram::from(crate::message::header::Header::UNKNOWN);
    let i_max = 100;

    match crate::server_and_client(dg, i_max,0).await {
        Err(_) => { assert!(false); },
        Ok((_, _, income, outcome)) => {
            for _ in 0..i_max {
                assert_eq!(crate::message::header::Header::UNKNOWN,income.pop().unwrap().data().header());
            }
        
            assert!(income.pop().is_none());
            assert!(outcome.pop().is_none());
        }
    }
    Ok(())
}