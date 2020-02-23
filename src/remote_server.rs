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

    pub fn request(&mut self, item: &GoldSrcPacket) -> io::Result<Vec<u8>> {
        debug!("Requesting info update from remote server");
        self.socket.send(item.as_ref())?;

        let mut buf = [0; 1024];
        let bytes_read = self.socket.recv(&mut buf)?;
        let received_buf = (&buf[0..bytes_read]).to_owned();

        if log_enabled!(Level::Trace) {
            trace!(
                "From (remote server?) received: {:?}",
                Bytes::from((&buf[0..bytes_read]).to_owned())
            );
        }

        Ok(received_buf)
    }
}
