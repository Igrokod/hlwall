use crate::caching_server::CachingServer;
use crate::packet::GoldSrcPacket;
use bytes::Bytes;
use log::{info, log_enabled, trace, warn, Level};
use std::convert::TryFrom;
use std::io;
use tokio::net::{ToSocketAddrs, UdpSocket};

pub(crate) struct ListenServer {
    socket: UdpSocket,
}

impl ListenServer {
    pub(crate) async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(ListenServer { socket })
    }

    pub(crate) async fn serve(mut self, mut remote_server: CachingServer) -> anyhow::Result<()> {
        loop {
            let mut buf = [0u8; 1024];
            let (bytes_read, client_addr) = self.socket.recv_from(&mut buf).await?;

            if log_enabled!(Level::Trace) {
                trace!(
                    "From {} received: {:?}",
                    client_addr,
                    Bytes::from((&buf[0..bytes_read]).to_owned())
                );
            }

            let packet = match GoldSrcPacket::try_from(&buf[0..bytes_read]) {
                Ok(v) => v,
                Err(e) => {
                    warn!(
                        "Failed to parse incoming packet from {}: {}",
                        client_addr, e
                    );
                    continue;
                }
            };

            info!("{:?} request from {}", packet, client_addr);
            let response = remote_server.request(packet).await?;
            self.socket.send_to(&response, client_addr).await?;
        }
    }
}
