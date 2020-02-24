use crate::caching_server::CachingServer;
use crate::packet::{GoldSrcPacket, MAX_INSPECTED_SIZE};
use anyhow::Context;
use bytes::Bytes;
use log::{info, log_enabled, trace, warn, Level};
use std::convert::TryFrom;
use std::io;
use std::net::{ToSocketAddrs, UdpSocket};

pub struct ListenServer {
    socket: UdpSocket,
    remote_server: CachingServer,
}

impl ListenServer {
    pub fn bind<A: ToSocketAddrs>(addr: A, remote_server: CachingServer) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        Ok(ListenServer {
            socket,
            remote_server,
        })
    }

    pub fn serve(mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; MAX_INSPECTED_SIZE];

        loop {
            let (bytes_read, client_addr) = self
                .socket
                .recv_from(&mut buf)
                .with_context(|| "Failed to read next udp request")?;

            let raw_packet = &buf[0..bytes_read];

            if log_enabled!(Level::Trace) {
                trace!(
                    "From {} received: {:?} (truncated to packet max inspected size of {})",
                    client_addr,
                    Bytes::copy_from_slice(raw_packet),
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
            let mut response_buf = [0u8; 1024];
            let bytes_written = self.remote_server.request(&packet, &mut response_buf)?;
            let packet = &response_buf[0..bytes_written];

            if log_enabled!(Level::Trace) {
                trace!(
                    "Sending {:?} response to {}, contents: {:?}",
                    packet,
                    client_addr,
                    Bytes::copy_from_slice(packet)
                )
            }

            self.socket.send_to(&packet, client_addr)?;
        }
    }
}
