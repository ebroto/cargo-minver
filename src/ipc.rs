use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::thread::{self, JoinHandle};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::feature::CrateAnalysis;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Analysis(CrateAnalysis),
    Collect,
}

pub fn send_message<A: ToSocketAddrs>(addr: A, message: &Message) -> Result<()> {
    use std::io::Write;

    let buffer = bincode::serialize(&message)?;
    let mut stream = TcpStream::connect(addr)?;
    stream.write_all(&buffer)?;

    Ok(())
}

#[derive(Debug)]
pub struct Server {
    addr: SocketAddr,
    join_handle: JoinHandle<Result<Vec<CrateAnalysis>>>,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(address: A) -> Result<Self> {
        let listener = TcpListener::bind(address)?;
        let addr = listener.local_addr()?;

        let join_handle = thread::Builder::new().name("server".into()).spawn(
            move || -> Result<Vec<CrateAnalysis>> {
                let mut data = vec![];
                for stream in listener.incoming() {
                    let stream = stream?;
                    let message = bincode::deserialize_from(&stream)?;
                    match message {
                        Message::Analysis(analysis) => {
                            data.push(analysis);
                        }
                        Message::Collect => {
                            break;
                        }
                    }
                }
                Ok(data)
            },
        )?;

        Ok(Self { addr, join_handle })
    }

    pub fn collect(self) -> Result<Vec<CrateAnalysis>> {
        send_message(&self.addr, &Message::Collect)?;
        self.join_handle.join().unwrap()
    }
}
