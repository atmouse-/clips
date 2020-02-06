extern crate clips;

use byteorder::{BigEndian, WriteBytesExt};

use clips::runtime::*;
use clips::storage::Shared;

use protobuf::Message;

use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::timer::Interval;

use clips::runtime::*;

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
    let mut magic_size = Vec::new();
    let mut magic = format!("{}\r\n", "").into_bytes();
    let dummy_data = String::from("").into_bytes();

    let mut sel = ClipMessage::default();
    sel.set_st_name(3);
    sel.set_st_size(dummy_data.len() as u32);
    sel.set_st_type(ClipMessage_msgtype::MSG_GET);
    sel.set_st_padding(dummy_data);

    // let mut magic: Vec<u8> = vec![b'\xfe', b'\xff'];
    // let mut magic = String::from("\r\n").into_bytes();
    let mut img = Vec::new();
    sel.write_to_vec(&mut img).unwrap();
    magic_size.write_u32::<BigEndian>(magic.len() as u32 + img.len() as u32).unwrap();

    // println!("magic_size: {:?}", &magic_size);
    // println!("magic: {:?}", &magic);

    let mut post_data: Vec<u8> = Vec::new();
    post_data.append(&mut magic_size);
    post_data.append(&mut magic);
    post_data.append(&mut img);

    // println!("post_data: {}, {:?}", post_data.len(), &post_data);

    let target = "127.0.0.1:9092".parse().unwrap();
    // let target = "127.0.0.1:9091".parse().unwrap();
    let client = TcpStream::connect(&target)
    .and_then(|stream| {
        // println!("second stream");

        io::write_all(stream, post_data)
    })
    .and_then(|(stream, buf)| {
        // writed buffer
        // println!("send buf: {:?}", buf);

        let buf: Vec<u8> = Vec::new();
        // let mut buf = vec![0; 8];
        io::read_to_end(stream, buf)
        // io::read_to_end(reader, buf).then(|result| {
        //     let result = result.unwrap();
        //     println!("read to stream; success={:?}", result.1);
        //     Ok(())
        // })
        // io::read_to_end(reader, buf).and_then(|(_, result)| {
        //     println!("read to stream; success={:?}", result);
        //     Ok(())
        // })

        // tokio::spawn(task);
        // println!("read: {:?}", &buf);
        // Ok(())
        // io::read_to_end(reader, buf)
        // .then(|result| {
        //     println!("read to stream; success={:?}", result.is_ok());
        //     Ok(())
        // })
        // Ok(())
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

    // println!("About to create the stream and write to it...");
    tokio::run(client);
    // println!("Stream has been created and writen to.");
}