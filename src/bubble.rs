#[macro_use]
extern crate futures;
extern crate tokio;
extern crate clips;

use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

use clips::runtime::*;
use clips::storage::Shared;

use protobuf::Message;

fn main() {
    let mut sel = ClipMessage::default();
    sel.set_st_name(3);
    sel.set_st_size(999);
    sel.set_st_type(ClipMessage_msgtype::MSG_PUSH);
    sel.set_st_padding(String::from("image context").into_bytes());

    let mut magic = String::from("aaaa\r\n").into_bytes();
    let mut img = vec!();
    sel.write_to_vec(&mut img).unwrap();
    magic.append(&mut img);

    let target = "127.0.0.1:9092".parse().unwrap();
    let client = TcpStream::connect(&target).and_then(|stream| {
        println!("second stream");

        io::write_all(stream, magic).then(|result| {
            println!("write to stream; success={:?}", result.is_ok());
            Ok(())
        })
    })
    .map_err(|err| {
        println!("connection error = {:?}", err);
    });

    println!("About to create the stream and write to it...");
    tokio::run(client);
    println!("Stream has been created and writen to.");
}