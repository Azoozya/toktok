#![allow(dead_code)]
#![allow(unused_variables)]

mod memory;
mod network;
mod message;
mod workers;
mod crypto;

use clap::{Arg, App};

use crate::network::host::Host;
use crate::network::udp::Datagram;
use crate::network::network::Network;

use crate::message::header::Header;
use crate::message::signal::{Signal,SignalType};

use crate::memory::sqlite::SqliteCore;
use crate::memory::shared_fifo::SharedFifo;

use crate::workers::{heartbeat::heartbeater,receive::receiver,emit::emitter,dispatch::dispatcher,config::Config};
// crate::workers::trace::tracer;

// for dev/test only
async fn server_and_client(dg: Datagram, i_max: usize, stade: u8) -> std::io::Result<(Network,Network,SharedFifo<Datagram,()>, SharedFifo<Datagram,()> )>{
    //////////////////////////// 
    let income: SharedFifo<Datagram,()> = SharedFifo::new(SignalType::notify);
    let outcome: SharedFifo<Datagram,()> = SharedFifo::new(SignalType::notify);
    ////////////////////////////
    let mut backbone: Signal<()> = Signal::new(SignalType::broadcast);
    //let tracing: Signal<Datagram> = Signal::new(SignalType::mpsc);
    ////////////////////////////
    let mut client = Config::from_file("client.config").unwrap();
    let mut server = Config::from_file("server.config").unwrap();
    let co = match SqliteCore::init("toktok.db") {
        None => { return Err( std::io::Error::new(std::io::ErrorKind::Other,"No db file found !")); },
        Some(co) => co,
    };

    if let Ok(key) = crypto::openssh::from(
        "toktok".to_string(),
        Some(
            "lama".to_string()
        )
    ) {
        client.verify(&key).unwrap();
        server.verify(&key).unwrap();
    }

    let client = client.into_network().await.unwrap();
    let server = server.into_network().await.unwrap();
    ////////////////////////////
    // 
    let mut tasks: Vec<tokio::task::JoinHandle<Result<(), std::io::Error>>> = Vec::new();
    tasks.push(tokio::task::spawn( 
        receiver(
            server.clone(),
            income.clone(),
            backbone.subscribe()
        )
    ));

    if stade > 0 {
        tasks.push(tokio::task::spawn( 
            dispatcher(
                server.clone(),
                income.clone(),
                outcome.clone(),
                /*tracing.subscribe(),*/
                backbone.subscribe()
            )
        ));
    }
    
    if stade > 1 {
        tasks.push(tokio::task::spawn( 
            emitter(
                server.clone(),
                outcome.clone(),
                backbone.subscribe()
            )
        ));
    }

    if stade > 2 {
        tasks.push(tokio::task::spawn(
            heartbeater(
                server.clone(),
                backbone.subscribe()
            )
        ));
    }

    /*
    if stade > 3 {
        tasks.push(tokio::task::spawn( 
            tracer(
                tracing,
                co,
                outcome.clone(),
                5
            )
        ));
    }*/
    //
    ////////////////////////////
    ////////////////////////////
    //
    let dst = Host::new( "127.0.0.1:3333" );

    for _ in 1..i_max {
        client.send_to(dg.clone(), Some(dst.clone()) ).await;
    }
    //
    ////////////////////////////
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    backbone.send(()).await.ok();
    client.send_to(dg, Some(dst) ).await;

    while !tasks.is_empty() { tasks.pop().unwrap().await.ok(); }

    Ok((client, server, income, outcome))
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let matches = App::new("Toktok")
        .version("0.0.0")
        .author("John Doe <john@doe.rust>")
        .about("Virtual Decentralized Network")
        .arg(Arg::with_name("config_file")
                 .short('c')
                 .long("config-file")
                 .takes_value(true)
                 .help("Configuration file to use. Default: toktok.config"))
        .arg(Arg::with_name("keyfile")
                 .short('k')
                 .long("keyfile")
                 .takes_value(true)
                 .help("OpenSSH format keyfile. Default: toktok"))
        // for private keyfile with password, should ask the password ... in an env file ? :c
        .arg(Arg::with_name("execution_mode")
                 .short('e')
                 .long("execution-mode")
                 .takes_value(true)
                 .help("execution mod: [C/s]"))
        .get_matches();
    
    let config_file = matches.value_of("config_file").unwrap_or("toktok.config");
    let key_file = matches.value_of("keyfile").unwrap_or("toktok");
    let execution_mode = matches.value_of("execution_mode");
    let force_as_server = match execution_mode {
        None => false,
        Some(s) => {
            match s.parse::<String>() {
                Err(_) => false,
                Ok(str) => (str.to_lowercase().get(0..1) == Some("s") ),
            }
        }
    };

    let dg = Datagram::from(Header::HELLO);
    let i_max = 2;

    let (client, server, income, outcome) = server_and_client(dg, i_max,255).await.unwrap();

    println!("\n\nClient:\n {:#?}",client);
    println!("\n\nServeur:\n {:#?}",server);
    println!("\n\nIncome:\n {:#?}",income);
    println!("\n\nOutcome:\n {:#?}",outcome);
    
    Ok(())
}