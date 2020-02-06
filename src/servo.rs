#[macro_use]
extern crate futures;
extern crate tokio;
extern crate tokio_uds;
extern crate bytes;
extern crate clips;

use clips::runtime::*;
use clips::storage::Shared;
use clips::session::Session;
use clips::session::StreamWriter;
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


fn handle_session(stream: TcpStream, state: Arc<Mutex<Shared>>) {
    let Session = Session::new(stream);

    let connection = Session
        .into_future()
        .map_err(|(e, _)| e)
        .and_then(move |(name, mut session)| {
            // let name = match name {
            //     Some(name) => name,
            //     None => {
            //         println!("Session first {:?} got", session.rd);
            //         return Either::A(future::ok(()));
            //     }
            // };

            // println!("{:?} join the chat", name);

            let (rx, mut tx) = session.stream.split();
            let mut writer = StreamWriter::new(tx);

            let dummy_data = String::from("").into_bytes();
            let mut msg = ClipMessage::default();
            msg.set_st_name(3);
            msg.set_st_size(dummy_data.len() as u32);
            msg.set_st_type(ClipMessage_msgtype::MSG_GET);
            msg.set_st_padding(dummy_data);

            println!("session second {:?} got", session.rd);

            {
                // let first_num = session.rd[0];
                let mut k = state.lock().unwrap();
                println!("got session.rd size: {}", session.rd.len());
                let mut sel: ClipMessage = match protobuf::parse_from_bytes(&session.rd.to_vec()) {
                    Ok(m) => m,
                    Err(_) => {
                        println!("parse ClipMessage error");
                        let mut bin = vec!();
                        msg.write_to_vec(&mut bin).unwrap();
                        writer.buffer(&bin);
                        return future::ok(writer)
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
                        if let Some(sel) = k.get_selection() {
                            let mut bin = vec!();
                            sel.write_to_vec(&mut bin).unwrap();
                            writer.buffer(&bin);
                            return future::ok(writer)
                        }
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
            // Either::B(future::ok(()))
            let mut bin = vec!();
            msg.write_to_vec(&mut bin).unwrap();
            writer.buffer(&bin);
            future::ok(writer)
        })
        .and_then(move |writer| { // write back
            // session.stream.poll_write(&bin).unwrap();
            tokio::spawn(writer.map_err(|e| println!("{}", e)));
            future::ok(())
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
    // monitor::spawn(state.clone());

    println!("3");
    // let server = rt.block_on(rx).unwrap();
    rt.run().unwrap();
    // let (_, buf) = rt.block_on(io::read_to_end(server, vec![])).unwrap();
    
}