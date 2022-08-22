use crate::network::udp::Datagram;
use crate::network::network::Network;

use crate::message::signal::Signal;

use crate::memory::shared_fifo::SharedFifo;

pub async fn dispatcher(net: Network, mut income: SharedFifo<Datagram,()>, mut outcome: SharedFifo<Datagram,()>,/*mut tracing: Signal<Datagram>,*/ mut backbone: Signal<()>) -> std::io::Result<()> {    
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

        match income.notified() {
            Err(err) => {
                break;
            },
            Ok(future) => {
                future.await;
                let mut maybe_dg = income.pop();
                while maybe_dg.is_some() {
                    tokio::task::spawn( 
                        crate::workers::handle::handler(
                            net.clone(),
                            maybe_dg.unwrap(),
                            /*tracing.subscribe(),*/
                            outcome.clone()
                        )
                    );
                    maybe_dg = income.pop();
                }
            }
        }
    }
    
    income.close();
    //tracing.close();
    outcome.close();
    Ok( backbone.close() )
}


#[tokio::test]
async fn test_dispatcher_hello() -> std::io::Result<()>{
    
    let dg = Datagram::from(crate::message::header::Header::HELLO);
    let i_max = 100;

    match crate::server_and_client(dg, i_max,1).await {
        Err(_) => { assert!(false); },
        Ok((_, server, income, outcome)) => {
            for _ in 0..i_max {
                assert_eq!(crate::message::header::Header::HELLO,outcome.pop().unwrap().data().header());
            }
        
            assert!(income.pop().is_none());
            assert!(outcome.pop().is_none());

            assert!(server.contains( &crate::network::host::Host::new( "127.0.0.1:3333" ) ));
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_dispatcher_ping() -> std::io::Result<()>{
    
    let dg = Datagram::from(crate::message::header::Header::PING);
    let i_max = 100;

    let (_, _, income, outcome) = crate::server_and_client(dg, i_max,1).await.unwrap();

    for _ in 0..i_max {
        assert_eq!(crate::message::header::Header::PONG,outcome.pop().unwrap().data().header());
    }

    assert!(income.pop().is_none());
    assert!(outcome.pop().is_none());
    Ok(())
}

#[tokio::test]
async fn test_dispatcher_unknown() -> std::io::Result<()>{
    
    let dg = Datagram::from(crate::message::header::Header::UNKNOWN);
    let i_max = 100;

    let (_, _, income, outcome) = crate::server_and_client(dg, i_max,1).await.unwrap();

    assert!(income.pop().is_none());
    assert!(outcome.pop().is_none());
    Ok(())
}