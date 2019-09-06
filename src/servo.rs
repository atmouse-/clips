#[macro_use]
extern crate futures;
extern crate tokio;
extern crate tokio_uds;
extern crate bytes;
extern crate clips;

use clips::runtime::*;
use clips::storage::Shared;
use clips::session::Session;
use clips::peer::Peer;

use std::thread;
use std::fs;
use std::str;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::timer::Interval;
use std::io::{Read};
use std::sync::{Arc, Mutex};
// use std::sync::mpsc;

use tokio::io;
use tokio::prelude::*;
use tokio::runtime::current_thread::Runtime;
use tokio_uds::{UnixStream, UnixListener};
use tokio_tcp::{TcpStream, TcpListener};

use futures::sync::oneshot;
use futures::sync::mpsc;
use futures::future::{self, Either};
use futures::Async;
use futures::Sink;
use futures::{Future, Stream};

use bytes::{BufMut, BytesMut};

use protobuf::Message;

mod monitor;

fn handle_session(stream: TcpStream, state: Arc<Mutex<Shared>>) {
    let Session = Session::new(stream);

    let connection = Session
        .into_future()
        .map_err(|(e, _)| e)
        .and_then(move |(name, mut session)| {
            let name = match name {
                Some(name) => name,
                None => {
                    println!("Session first {:?} got", session.rd);
                    return Either::A(future::ok(()));
                }
            };

            println!("{:?} join the chat", name);

            println!("session second {:?} got", session.rd);

            {
                // let first_num = session.rd[0];
                let mut k = state.lock().unwrap();
                println!("got session.rd size: {}", session.rd.len());
                let mut sel: ClipMessage = match protobuf::parse_from_bytes(&session.rd.to_vec()) {
                    Ok(m) => m,
                    Err(_) => {
                        println!("parse ClipMessage error");
                        return Either::A(future::ok(()))
                    }
                };
                
                sel.handle();
                match sel.st_type {
                    ClipMessage_msgtype::MSG_PUSH => {
                        println!("push selection ok!");
                        k.push_selection(sel)
                    },
                    ClipMessage_msgtype::MSG_GET => {
                        println!("get selection!");
                        let mut img = vec!();
                        sel.set_st_padding(k.first_blob().unwrap());
                        println!("get selection! 2");
                        sel.write_to_vec(&mut img).unwrap();
                        true
                    }
                    _ => false
                };

                //k.input.keyin(first_num);
            }

            // Create the peer.
            //
            // This is also a future that processes the connection, only
            // completing when the socket closes.
            // let peer = Peer::new(name, state, session);

            // Wrap `peer` with `Either::B` to make the return type fit.
            // Either::B(peer)
            Either::B(future::ok(()))
        })
        .map_err(|e| {
            println!("connection error = {:?}", e);
        });

    tokio::spawn(connection);
}


fn main() {
    let mut rt = Runtime::new().unwrap();
    let (tx, rx) = mpsc::channel::<u8>(1024);

    let state = Arc::new(Mutex::new(Shared::new()));

    let listen_addr = "0.0.0.0:9092".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&listen_addr).expect("Cannot bind port 9092!");
    let state1 = state.clone();
    rt.spawn({
        listener
            .incoming()
            .map_err(|e| println!("err={:?}", e))
            .for_each(move |stream| {
                handle_session(stream, state1.clone());
                Ok(())
            })
        
    });

    // PULL
    monitor::spawn(state.clone());

    println!("3");
    // let server = rt.block_on(rx).unwrap();
    rt.run().unwrap();
    // let (_, buf) = rt.block_on(io::read_to_end(server, vec![])).unwrap();
    
}