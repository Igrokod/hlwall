use crate::packet::GoldSrcPacket;
use log::debug;
use std::io;
use tokio::net::{ToSocketAddrs, UdpSocket};
use bytes::Bytes;
use log::{log_enabled, trace, Level};

pub(crate) struct RemoteServer {
    socket: UdpSocket,
}

impl RemoteServer {
    pub(crate) async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;

        Ok(RemoteServer { socket })
    }

    pub(crate) async fn request(&mut self, item: &GoldSrcPacket) -> io::Result<Vec<u8>> {
        debug!("Requesting info update from remote server");
        self.socket.send(item.as_ref()).await?;

        let mut buf = [0; 1024];
        let bytes_read = self.socket.recv(&mut buf).await?;
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
