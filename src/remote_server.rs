use crate::packet::GoldSrcPacket;
use bytes::Bytes;
use log::debug;
use log::{log_enabled, trace, Level};
use std::io;
use std::net::{ToSocketAddrs, UdpSocket};

pub struct RemoteServer {
    socket: UdpSocket,
}

impl RemoteServer {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;

        Ok(RemoteServer { socket })
    }

    pub fn request(
        &mut self,
        item: &GoldSrcPacket,
        mut return_buf: &mut [u8],
    ) -> io::Result<usize> {
        debug!("Requesting info update from remote server");
        let mut buf = [0u8; 1024];
        let bytes_written = item.serialize(&mut buf)?;
        let packet = &buf[0..bytes_written];
        self.socket.send(packet)?;

        if log_enabled!(Level::Trace) {
            trace!(
                "Sending to remote server: {:?}",
                Bytes::copy_from_slice(packet)
            );
        }

        let bytes_written = self.socket.recv(&mut return_buf)?;

        if log_enabled!(Level::Trace) {
            trace!(
                "From (remote server?) received: {:?}",
                Bytes::copy_from_slice(&return_buf[0..bytes_written])
            );
        }

        Ok(bytes_written)
    }
}
