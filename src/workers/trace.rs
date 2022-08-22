#![allow(unused_imports)]
use sqlite::Connection;
use crate::message::signal::Signal;

use crate::network::host::Host;
use crate::network::udp::Datagram;

use crate::memory::sqlite::SqliteCore;
use crate::memory::shared_fifo::SharedFifo;

pub async fn tracer(mut income: Signal<Datagram>, co: Connection, outcome: SharedFifo<Datagram,()>, gracetime: u64) -> Result<(), std::io::Error> { 
    let stmt_read = "SELECT * FROM Core WHERE Addr = :client_hash";
    let stmt_create = "INSERT INTO Core VALUES (:client_hash)";
    let stmt_write_openssh = " UPDATE Core SET OpensshID = :openssh_id, OpensshPub = :openssh_pub WHERE Addr = :client_hash";
    let stmt_write_activity = " UPDATE Core SET Active = :active,  LastActivity = 'now' WHERE Addr = :client_hash";
    /*
    Open sqlite
    */
    
    //co.prepare(stmt_read).unwrap().clone();
    loop {
        match income.try_recv() {
            Ok(dg) => {
                match dg {
                    None => { 
                        tokio::time::sleep(tokio::time::Duration::from_secs(gracetime)).await;
                    }
                    Some(dg) => {
                        /**/
                    }
                }
            }
            Err(_) => { break; }
        }
    }
    /*
    Close sqlite
    */
    Ok( income.close() )
}
 