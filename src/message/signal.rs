#![allow(non_camel_case_types)]

use std::sync::Arc;
use tokio::sync::{Notify,mpsc,broadcast};
use tokio::sync::futures::Notified;

/*
match self.signal {
    SignalType::stub => {},

    SignalType::notify => {},

    SignalType::mpsc => {},

    SignalType::broadcast => {},
}
*/

#[derive(Debug)]
pub enum SignalErr {
    WrongSignal,
    MPSCErr,
    BroadcastError,
    NoReceiver,
    NoSender,
    NoHandler,
}

#[derive(Debug,Clone,Copy)]
pub enum SignalType {
    notify,
    mpsc,
    broadcast,
    stub,
}

#[derive(Debug)]
pub struct Signal<T> {
    signal: SignalType,
    inner_notify: Option<Arc<Notify>>,
    inner_mpsc: ( Option<mpsc::Sender<T>>, Option<mpsc::Receiver<T>>),
    inner_broadcast: ( Option<broadcast::Sender<T>>, Option<broadcast::Receiver<T>>),

}

impl<T: std::clone::Clone> Signal<T> {

    // ctor
    pub fn new(signal_type: SignalType) -> Signal<T> {
        match signal_type {
            SignalType::stub => {
                Signal { 
                    signal: SignalType::stub, 
                    inner_notify: None,
                    inner_mpsc: (None,None),
                    inner_broadcast: (None,None)
                } 
            },
            SignalType::notify => {
                Signal { 
                    signal: SignalType::notify, 
                    inner_notify: Some( Arc::new( Notify::new() ) ),
                    inner_mpsc: (None,None),
                    inner_broadcast: (None,None)
                }
            },
            SignalType::mpsc => { 
                let (tx, rx): (mpsc::Sender<T>, mpsc::Receiver<T>) = mpsc::channel(255);                
                Signal { 
                    signal: SignalType::mpsc, 
                    inner_notify: None,
                    inner_mpsc: (Some(tx),Some(rx)),
                    inner_broadcast: (None,None)
                }
            },
            SignalType::broadcast => {
                let (tx, rx): (broadcast::Sender<T>, broadcast::Receiver<T>) = broadcast::channel(255);               
                Signal { 
                    signal: SignalType::broadcast, 
                    inner_notify: None,
                    inner_mpsc: (None,None),
                    inner_broadcast: (Some(tx),Some(rx))
                }
            },
        }
    }
    
    // clone notify, clone mpsc, clone-subscribe broadcast
    pub fn subscribe(&self) -> Signal<T> {
        match self.signal {
            SignalType::stub => {
                Signal { 
                    signal: SignalType::stub, 
                    inner_notify: None,
                    inner_mpsc: (None,None),
                    inner_broadcast: (None,None)
                }
            },
            // Increase arc
            SignalType::notify => {
                Signal { 
                    signal: SignalType::notify, 
                    inner_notify: Some( Arc::clone(&self.inner_notify.as_ref().unwrap() ) ),
                    inner_mpsc: (None,None),
                    inner_broadcast: (None,None)
                }
            },
            // Clone sender
            SignalType::mpsc => {
                let sender =  self.inner_mpsc.0.as_ref().unwrap();                
                Signal {
                    signal: SignalType::mpsc, 
                    inner_notify: None,
                    inner_mpsc: (
                        Some( 
                            sender
                            .clone()
                        ),
                        None),
                    inner_broadcast: (None,None)
                }
            },
            // Clone sender and subscribe a new receiver
            SignalType::broadcast => {      
                let sender =  self.inner_broadcast.0.as_ref().unwrap();      
                Signal {
                    signal: SignalType::broadcast, 
                    inner_notify: None,
                    inner_mpsc: (None,None),
                    inner_broadcast: (
                        Some( 
                            sender
                            .clone() 
                        ),
                        Some(
                            sender
                            .subscribe()
                        )
                    ),
                }
            },
        }
    }

    pub fn close(&mut self) {
        match self.signal {
            SignalType::stub => {
                /* Nothing to do */
            },

            SignalType::notify => {
                self.inner_notify = None;
            },

            SignalType::mpsc => {
                self.inner_mpsc.0 = None;
                if let Some(ptr) = self.inner_mpsc.1.as_mut() {
                    ptr.close();
                };
                self.inner_mpsc.1 = None;
            },

            SignalType::broadcast => {
                self.inner_broadcast = (None,None);
            },
        }
        self.signal = SignalType::stub;
    }

    // For Notify
    pub fn notify_one(&self) -> Result< (),SignalErr > {

        match self.signal {
            SignalType::stub => {
                Err( SignalErr::NoHandler )
            },

            SignalType::notify => { 
                Ok(
                    self.inner_notify
                    .as_ref()
                    .unwrap()
                    .notify_one()
                )
            },

            _ => {
                Err( SignalErr::WrongSignal )
            },
        }
    }

    pub fn notified(&self) -> Result< Notified<'_>,SignalErr > {
        match self.signal {
            SignalType::stub => {
                Err( SignalErr::NoHandler )
            },

            SignalType::notify => { 
                Ok( 
                    self.inner_notify
                    .as_ref()
                    .unwrap()
                    .notified() 
                )
            },

            _ => {
                Err( SignalErr::WrongSignal )
            },
        }
    }

    // For mpsc/broadcast
    pub async fn send(&mut self, data: T) ->  Result< (),SignalErr >{
        match self.signal {
            SignalType::stub => {
                Err( SignalErr::NoHandler )
            },
            SignalType::notify => { 
                self.notify_one()
            },
            SignalType::mpsc => {
                // Result<(), SendError<T>>
                // https://docs.rs/tokio/1.20.1/tokio/sync/mpsc/struct.Sender.html#method.send
                match &mut self.inner_mpsc.0 {
                    None => Err(SignalErr::NoSender),
                    Some(sender) => {
                        match sender.send(data).await {
                            Ok(_) => Ok(()),
                            /*
                            If the receive half of the channel is closed, 
                            either due to close being called or the Receiver handle dropping, 
                            the function returns an error. 
                            */
                            Err(_) => Err( SignalErr::NoReceiver ),
                        }
                    },
                }
            },
            SignalType::broadcast => {
                // Result<usize, SendError<T>>
                // https://docs.rs/tokio/1.20.1/tokio/sync/broadcast/struct.Sender.html#method.send
                match &mut self.inner_broadcast.0 {
                    None => Err(SignalErr::NoSender),
                    Some(sender) => {
                        match sender.send(data) {
                            Ok(_) => Ok( () ),
                            /*
                            An unsuccessful send would be one where 
                            all associated Receiver handles have already been dropped.
                            */
                            Err(_) => Err( SignalErr::NoReceiver ),
                        }
                    }                   
                }
            },
        }
    }

    pub async fn recv(&mut self) -> Result< Option<T>,SignalErr > {
        match self.signal {
            SignalType::stub => {
                Err( SignalErr::NoHandler )
            },
            SignalType::notify => { 
                match self.notified() {
                    Err(err) => Err(err),
                    Ok(future) => {
                        future.await;
                        Ok( None )
                    },
                }
            },
            SignalType::mpsc => {
                // No error can occur
                // https://docs.rs/tokio/1.20.1/tokio/sync/mpsc/struct.Receiver.html
                match &mut self.inner_mpsc.1 {
                    None => Err(SignalErr::NoReceiver),
                    Some(receiver) => {
                        Ok( receiver.recv().await )
                    },
                }
            },
            SignalType::broadcast => {
                // Result<T, RecvError>
                // https://docs.rs/tokio/1.20.1/tokio/sync/broadcast/struct.Receiver.html#method.recv
                match &mut self.inner_broadcast.1 {
                    None => Err(SignalErr::NoReceiver),
                    Some(receiver) => {
                        match receiver.recv().await {
                            Err(err) => Err(SignalErr::BroadcastError),
                            Ok(data) => Ok( Some(data) ),
                        }
                    }                   
                }
            }
        }
    }

    // Must find how to test if a future is resolved without await it
    // to implement try_recv for SignalType::notify
    pub fn try_recv(&mut self) ->  Result< Option<T>,SignalErr > {
        match self.signal {
            SignalType::stub => {
                Err( SignalErr::NoHandler )
            },
            SignalType::notify => { 
                Err( SignalErr::WrongSignal )
            },
            SignalType::mpsc => {
                //Result<T, RecvError>
                match &mut self.inner_mpsc.1 {
                    None => Err(SignalErr::NoReceiver),
                    Some(receiver) => {
                        match receiver.try_recv() {
                            Ok(data) => Ok( Some(data) ),
                            Err(err) => {
                                match err {
                                    mpsc::error::TryRecvError::Empty => Ok( None ),
                                    mpsc::error::TryRecvError::Disconnected => Err( SignalErr::NoSender ),
                                }
                            },
                        }
                    },
                }
            },
            SignalType::broadcast => {
                //Result<T, RecvError>
                match &mut self.inner_broadcast.1 {
                    None => Err(SignalErr::NoReceiver),
                    Some(receiver) => {
                        match receiver.try_recv() {
                            Ok(data) => Ok( Some(data) ),
                            Err(err) => {
                                match err {
                                    broadcast::error::TryRecvError::Empty => Ok( None ),
                                    broadcast::error::TryRecvError::Closed => Err( SignalErr::NoSender ),
                                    broadcast::error::TryRecvError::Lagged(_) => Err( SignalErr::BroadcastError ),
                                }
                            },
                        }
                    },
                }
            }
        }
    }
}

impl <T: std::clone::Clone> Clone for Signal<T> {
    fn clone(&self) -> Self {
        self.subscribe()
    }
}