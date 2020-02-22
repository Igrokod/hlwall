use crate::caching_server::CachingServer;
use crate::packet::{GoldSrcPacket, MAX_INSPECTED_SIZE};
use anyhow::Context;
use bytes::Bytes;
use log::{info, log_enabled, trace, warn, Level};
use std::convert::TryFrom;
use std::io;
use tokio::net::{ToSocketAddrs, UdpSocket};

pub(crate) struct ListenServer {
    socket: UdpSocket,
    remote_server: CachingServer,
}

impl ListenServer {
    pub(crate) async fn bind<A: ToSocketAddrs>(
        addr: A,
        remote_server: CachingServer,
    ) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(ListenServer {
            socket,
            remote_server,
        })
    }

    pub(crate) async fn serve(mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; MAX_INSPECTED_SIZE];

        loop {
            let (bytes_read, client_addr) = self
                .socket
                .recv_from(&mut buf)
                .await
                .with_context(|| "Failed to read next udp request")?;

            let raw_packet = &buf[0..bytes_read];

            if log_enabled!(Level::Trace) {
                trace!(
                    "From {} received: {:?} (truncated to packet max inspected size of {})",
                    client_addr,
                    Bytes::from(raw_packet.to_owned()),
                    MAX_INSPECTED_SIZE
                );
            }

            let packet = match GoldSrcPacket::try_from(&raw_packet[..]) {
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
            let response = self.remote_server.request(&packet).await?;

            if log_enabled!(Level::Trace) {
                trace!(
                    "Sending {:?} response to {}, contents: {:?}",
                    packet,
                    client_addr,
                    Bytes::from(response.clone())
                )
            }

            self.socket.send_to(&response, client_addr).await?;
        }
    }
}
