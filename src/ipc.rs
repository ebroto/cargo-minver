use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream};
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::feature::CrateAnalysis;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Analysis(CrateAnalysis),
    Collect,
}

#[derive(Debug)]
pub struct Server {
    address: SocketAddr,
    join_handle: JoinHandle<Result<Vec<CrateAnalysis>>>,
}

impl Server {
    pub fn new(address: SocketAddr) -> Result<Self> {
        let listener = TcpListener::bind(address).context("could not bind to local address")?;
        let join_handle = thread::Builder::new()
            .name("server".into())
            .spawn(|| Server::serve(listener))
            .context("error serving")?;

        Ok(Self {
            address,
            join_handle,
        })
    }

    fn serve(listener: TcpListener) -> Result<Vec<CrateAnalysis>> {
        let mut data = Vec::new();
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
    }

    pub fn collect(self) -> Result<Vec<CrateAnalysis>> {
        send_message(self.address, &Message::Collect).context("could not stop server")?;
        self.join_handle.join().unwrap()
    }
}

pub fn send_message(addr: SocketAddr, message: &Message) -> Result<()> {
    let buffer = bincode::serialize(&message)?;
    let mut stream = TcpStream::connect(addr)?;
    stream.write_all(&buffer)?;
    Ok(())
}

pub fn server_address(port: u16) -> SocketAddr {
    SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).into()
}
