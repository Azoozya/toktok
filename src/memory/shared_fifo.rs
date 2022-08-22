
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use tokio::sync::futures::Notified;

use crate::message::signal::{Signal,SignalType,SignalErr};

#[derive(Debug)]
pub struct SharedFifo<T,V>  {
    fifo: Arc<Mutex<VecDeque<T>>>,
    signal: Signal<V>,
}

impl <T,V: std::clone::Clone> SharedFifo<T,V> {

    pub fn new(signal_mod: SignalType) -> SharedFifo<T,V> {
        let data: VecDeque<T> = VecDeque::new();
        let fifo = Arc::new(Mutex::new(data));
        let signal: Signal<V> = Signal::new(signal_mod);
        SharedFifo { fifo, signal }        
    }

    // assuming it's a fifo => push_front
    pub fn push(&mut self, data: T) -> () {
        {
            let mut fifo = self.fifo.lock().unwrap();
            fifo.push_front(data);
        }
    }
    pub async fn push_notice(&mut self, data: T, value: V) -> Result< (),SignalErr >{
        self.push(data);
        self.send(value).await
    }
    // assuming it's a fifo => pop_back
    pub fn pop(&self) -> Option<T> {
        let mut fifo = self.fifo.lock().unwrap();
        fifo.pop_front()
    }

    pub fn try_recv(&mut self) -> Result< Option<V>,SignalErr > {
        self.signal.try_recv()
    }
    pub async fn recv(&mut self) -> Result< Option<V>,SignalErr > {
        self.signal.recv().await
    }
    pub async fn send(&mut self, data: V) -> Result< (),SignalErr > {
        self.signal.send(data).await
    }   
    pub fn notified(&self) -> Result< Notified<'_>,SignalErr > {
        self.signal.notified()
    }
    pub fn notify_one(&self) -> Result< (),SignalErr > {
        self.signal.notify_one()
    }

    pub fn close(&mut self) {
        self.signal.close()
    }
}

impl<T,V: std::clone::Clone>  Clone for SharedFifo<T,V> {
    fn clone(&self) -> Self {
        SharedFifo { 
            fifo: Arc::clone(&self.fifo), 
            signal: self.signal.subscribe() 
        }
    }
}

#[tokio::test]
async fn test_shared_fifo_ping_pong() {
    let mut sfifo: SharedFifo<usize,()> = SharedFifo::new(SignalType::notify);
    let mut ssfifo = sfifo.clone();
    let debug = sfifo.clone();

    let dg1 = 111;
    let dg2 = 222;

    let d = tokio::task::spawn(async move {   
        ssfifo.notified() .unwrap();
        ssfifo.push_notice(dg2.clone(),()).await.unwrap();
    });

    let t = tokio::task::spawn(async move {
        sfifo.push(dg1.clone());
        sfifo.notify_one() .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        sfifo.notified() .unwrap();
    });

    d.await.unwrap();
    t.await.unwrap();

    assert_eq!(Some(dg1),debug.pop());
    assert_eq!(Some(dg2),debug.pop());
    assert_eq!(None,debug.pop());
}

#[tokio::test]
async fn test_shared_fifo_broadcast() {
    let mut tasks: Vec<tokio::task::JoinHandle<_>> = Vec::new();

    let sfifo: SharedFifo<usize,usize> = SharedFifo::new(SignalType::broadcast);
    let mut debug = sfifo.clone();

    let dg1 = 123;
    let i_max = 1000;
    for _ in 1..i_max {
        let mut ssfifo = sfifo.clone();
        tasks.push(
            tokio::task::spawn(
                async move {   
                    let data = ssfifo.recv().await.unwrap().unwrap();
                    ssfifo.push(data);
                }
            )
        );
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    assert!(debug.send(dg1).await.is_ok());

    let mut i = 1;
    while !tasks.is_empty() {
      let task = tasks.pop().unwrap();
      assert!(task.await.is_ok());
      assert_eq!(Some(dg1),debug.pop());
      i += 1;
    }
    
    assert_ne!(1,i);
    assert_eq!(i_max,i);
    assert_eq!(None,debug.pop());
}
