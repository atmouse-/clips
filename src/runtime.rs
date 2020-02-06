pub use super::msg::*;
use tokio::io;
use tokio::prelude::*;
use tokio::net::{TcpListener, TcpStream};

pub trait Select {
    fn handle(&self) -> ();
}

impl Select for ClipMessage {
    fn handle(&self) -> () {
        match self.st_type {
            ClipMessage_msgtype::MSG_GET => {
                ;
            },
            ClipMessage_msgtype::MSG_GET => {
                ;
            }
            _ => {
                ;
            }
        }
    }

}