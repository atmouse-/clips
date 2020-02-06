#[macro_use]
extern crate futures;
extern crate tokio;
extern crate clips;

use byteorder::{BigEndian, WriteBytesExt};

use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

use clips::runtime::*;
use clips::storage::Shared;

use protobuf::Message;

use std::fs::File;
use std::io::Read;

use std::env;
use std::path::Path;
use std::ffi::OsStr;

fn prog() -> Option<String> {
    env::args().next()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
}

fn main() {
    let mut sel = ClipMessage::default();
    let (data, paddingtype) = match prog().unwrap().as_str() {
        "clipc-push-png" => {
            let mut data = Vec::new();
            std::io::stdin().read_to_end(&mut data).unwrap();
            (data, ClipMessage_paddingtype::PNG)
        },
        _ => { // contain "clipc-push-txt"
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer).unwrap();
            let data = buffer.into_bytes();
            (data, ClipMessage_paddingtype::TXT)
        },
    };
    sel.set_st_name(3);
    sel.set_st_size(data.len() as u32);
    sel.set_st_type(ClipMessage_msgtype::MSG_PUSH);
    sel.set_st_padding(data);
    sel.set_st_paddingtype(paddingtype);

    let mut magic = String::from("aaaa\r\n").into_bytes();
    let mut img = vec!();
    sel.write_to_vec(&mut img).unwrap();

    let mut magic_size = Vec::new();
    magic_size.write_u32::<BigEndian>(magic.len() as u32 + img.len() as u32).unwrap();

    let mut post_data: Vec<u8> = Vec::new();
    post_data.append(&mut magic_size);
    post_data.append(&mut magic);
    post_data.append(&mut img);

    let target = "127.0.0.1:9092".parse().unwrap();
    let client = TcpStream::connect(&target)
    .and_then(|stream| {
        println!("second stream");

        let (rx, tx) = stream.split();
        io::write_all(tx, post_data).then(|result| {
            println!("write to stream; success={:?}", result.is_ok());
            Ok(rx)
        })
    })
    .and_then(|rx| {
        let buf: Vec<u8> = Vec::new();
        io::read_to_end(rx, buf)

    })
    .and_then(|(_stream, buf)| {
        // println!("read to stream; success={:?}", &buf);
        let mut sel: ClipMessage = match protobuf::parse_from_bytes(&buf) {
            Ok(m) => m,
            Err(_) => {
                eprintln!("parse ClipMessage error");
                return Ok(())
            }
        };
        
        match sel.get_st_paddingtype() {
            ClipMessage_paddingtype::PNG => {/* TODO PNG */},
            ClipMessage_paddingtype::TXT => {println!("{}", std::str::from_utf8(&sel.take_st_padding()).unwrap());},
        }
        
        Ok(())
    })
    .map_err(|err| {
        eprintln!("connection error = {:?}", err);
    });

    println!("About to create the stream and write to it...");
    tokio::run(client);
    println!("Stream has been created and writen to.");
}