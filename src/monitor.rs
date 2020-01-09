use std::net::TcpListener;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::io::Read;
use std::io::Write;
use std::io;
use std::thread;


use clips::runtime::*;
use clips::storage::Shared;

fn handle_monitor(mut stream: TcpStream, state: Arc<Mutex<Shared>>) {
    // let mut buf: Vec<u8> = Vec::new();
    let mut buf = vec![0; 4];
    // let mut null: Vec<u8> = vec![0; 2];
    // stream.read(&mut null); // skip magic
    stream.read_exact(&mut buf); // FIXME: fix long timeout
    println!("got: raw: {:?}", buf);
    let mut sel: ClipMessage = match protobuf::parse_from_bytes(&buf) {
        Ok(m) => m,
        Err(_) => return ()
    };

    println!("got: sel: {:?}", sel.st_padding);

    {
        println!("handle get");
        let mut k = state.lock().unwrap();
        match k.first_blob() {
            Ok(m) => {
                println!("send..to: {:?}", &m);
                stream.write_all(&m);
            },
            Err(_) => ()
        }
        
    }

    println!("handle_monitor: send all ok!");
    
}

pub fn spawn(state: Arc<Mutex<Shared>>) -> io::Result<()> {
    // PULL
    thread::spawn(move || {
        let listen_addr = "0.0.0.0:9091".parse::<SocketAddr>().unwrap();
        let listener = TcpListener::bind(&listen_addr).expect("Cannot bind port 9091!");

        for stream in listener.incoming() {
            let state1 = state.clone();
            thread::spawn(move || {
                handle_monitor(stream.unwrap(), state1)
            });
        }
    });
    Ok(())
}
