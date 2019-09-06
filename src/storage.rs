use std::collections::VecDeque;
use std::collections::HashMap;
use std::net::SocketAddr;
use futures::sync::mpsc;
use bytes::{BufMut, Bytes, BytesMut};

use super::msg::ClipMessage;
use std::io;

const VD_LENGTH: usize = 10;

/// Shorthand for the transmit half of the message channel.
type Tx = mpsc::UnboundedSender<Bytes>;

// Shared State
pub struct Shared {
    vd: VecDeque<ClipMessage>,
    pub peers: HashMap<SocketAddr, Tx>,
}

impl Shared {
    pub fn new() -> Self {
        Shared {
            vd: VecDeque::new(),
            peers: HashMap::new(),
        }
    }

    pub fn push_selection(&mut self, c: ClipMessage) -> bool {
        if self.vd.len() >= VD_LENGTH {
            println!("Dequefull pop!");
            self.pop_selection();
        }
        self.vd.push_back(c);
        true
    }

    pub fn get_selection(&self) -> Option<ClipMessage> {
        // TODO: get back
        match self.vd.back() {
            Some(m) => {
                Some(m.clone())
            },
            None => {
                None
            }
        }
    }

    pub fn pop_selection(&mut self) -> Result<(), io::Error> {
        // pop front
        self.vd.pop_front();
        Ok(())
    }

    pub fn first_blob(&self) -> Result<Vec<u8>, io::Error> {
        let sel = self.get_selection();
        match sel {
            Some(s) => {
                Ok(s.st_padding)
            },
            None => {
                Err(io::Error::from(io::ErrorKind::NotFound))
            }
        }
    }
}
