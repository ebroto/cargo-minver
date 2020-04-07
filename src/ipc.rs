use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::feature::{Analysis, CrateAnalysis};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    AnalysisResult(CrateAnalysis),
    Collect,
}

#[derive(Debug)]
pub struct Server {
    port: u16,
    join_handle: JoinHandle<Result<Analysis>>,
}

impl Server {
    pub fn new() -> Result<Self> {
        let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let listener = TcpListener::bind(address).context("could not bind to local address")?;
        let port = listener.local_addr().context("could not retrieve local port")?.port();
        let join_handle = thread::Builder::new() //
            .name("server".into())
            .spawn(|| Server::serve(listener))
            .context("error serving")?;

        Ok(Self { port, join_handle })
    }

    fn serve(listener: TcpListener) -> Result<Analysis> {
        let mut data = Vec::new();
        for stream in listener.incoming() {
            let stream = stream?;
            let message = bincode::deserialize_from(&stream)?;
            match message {
                Message::AnalysisResult(analysis) => {
                    data.push(analysis);
                },
                Message::Collect => {
                    break;
                },
            }
        }
        Ok(data.into())
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn into_analysis(self) -> Result<Analysis> {
        send_message(self.port, &Message::Collect).context("could not stop server")?;
        self.join_handle.join().unwrap()
    }
}

pub fn send_message(port: u16, message: &Message) -> Result<()> {
    let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let mut stream = TcpStream::connect(address)?;
    let buffer = bincode::serialize(&message)?;
    stream.write_all(&buffer)?;
    Ok(())
}
