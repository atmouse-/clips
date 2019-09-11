use bytes::{BufMut, Bytes, BytesMut};
use byteorder::{BigEndian, ReadBytesExt};
use futures::future::{self, Either};
use futures::sync::mpsc;
use futures::try_ready;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

fn transform_u32_to_array_of_u8(x:u32) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}

fn transform_u8_to_u32(s: [u8; 4]) -> u32 {
    let b1: u32 = s[0] as u32;
    let b2: u32 = s[1] as u32 * 256;
    let b3: u32 = s[2] as u32 * 65536;
    let b4: u32 = s[3] as u32 * 16777216;
    let bx: u32 = b1 + b2 + b3 + b4;
    bx as u32
}

#[derive(Debug)]
pub struct Session {
    pub stream: TcpStream,
    pub rd: BytesMut,
    pub wr: BytesMut,
    pub magic_size: u32,
    pub cap_size: u32,
}

impl Session {
    pub fn new(stream: TcpStream) -> Self {
        Session {
            stream,
            rd: BytesMut::new(),
            wr: BytesMut::new(),
            magic_size: 0,
            cap_size: 0,
        }
    }

    /// Buffer a line.
    ///
    /// This writes the line to an internal buffer. Calls to `poll_flush` will
    /// attempt to flush this buffer to the socket.
    pub fn buffer(&mut self, line: &[u8]) {
        // Ensure the buffer has capacity. Ideally this would not be unbounded,
        // but to keep the example simple, we will not limit this.
        self.wr.reserve(line.len());

        // Push the line onto the end of the write buffer.
        //
        // The `put` function is from the `BufMut` trait.
        println!("write ok {:?}", self.wr);
        self.wr.put(line);
    }

    /// Flush the write buffer to the socket
    pub fn poll_flush(&mut self) -> Poll<(), io::Error> {
        // As long as there is buffered data to write, try to write it.
        while !self.wr.is_empty() {
            // Try to write some bytes to the socket
            let n = try_ready!(self.stream.poll_write(&self.wr));

            // As long as the wr is not empty, a successful write should
            // never write 0 bytes.
            assert!(n > 0);

            // This discards the first `n` bytes of the buffer.
            let _ = self.wr.split_to(n);
        }

        Ok(Async::Ready(()))
    }

    fn fill_read_buf(&mut self) -> Poll<(), io::Error> {
        self.rd.reserve(1024*64);

        if self.magic_size == 0 {
            let mut buf = BytesMut::new();
            buf.reserve(1024*64);
            let mut n: u32 = 0;
            while buf.len() < 4 {
                let ni = try_ready!(self.stream.read_buf(&mut buf)) as u32;
                if ni == 0 {
                    return Ok(Async::Ready(()));
                }
                n += ni
            }
            println!("get buf {:?}", &buf);
            let mut magic_size = buf.get(0..4).unwrap();
            self.magic_size = magic_size.read_u32::<BigEndian>().unwrap();
            self.cap_size = self.magic_size + 4;
            println!("get magic_size {}", self.magic_size);
            println!("get cap_size {}", self.cap_size);

            self.rd.put(buf);
            self.cap_size = self.cap_size - n;
        }

        loop {
            println!("in loop");
            self.rd.reserve(1024*64);

            let mut buf = BytesMut::new();
            buf.reserve(1024*64);

            // let n = try_ready!(self.stream.read_buf(&mut self.rd));

            // match self.stream.read_buf(&mut buf)
            let n = try_ready!(self.stream.read_buf(&mut buf)) as u32;

            if n == 0 {
                println!("read ok {:?}", self.rd);
                return Ok(Async::Ready(()));
            }

            if self.cap_size > 0 {
                println!("get buf_size {}", n);
                println!("rel cap_size {}", self.cap_size);
                self.rd.put(buf);
                self.cap_size = self.cap_size - n;
                // return Ok(Async::NotReady)
            } else {
                // read end of message
                println!("end of message");
                self.cap_size = self.cap_size - n;
                self.rd.put(buf);
            }
        }
    }
}

impl Stream for Session {
    type Item = BytesMut;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // let sock_closed = self.fill_read_buf()?.is_ready();
        let is_ready = self.fill_read_buf()?.is_ready();

        println!("in poll once");
        if is_ready {
            let pos = self
                .rd
                .windows(2)
                .enumerate()
                .find(|&(_, bytes)| bytes == b"\r\n")
                .map(|(i, _)| i);
            if let Some(pos) = pos {
                let mut line = self.rd.split_to(pos + 2);
                line.split_off(pos);
                Ok(Async::Ready(Some(line)))
            } else {
                Ok(Async::Ready(None))
            }
        } else {
            println!("in poll not ready");
            Ok(Async::NotReady)
        }
    }
}