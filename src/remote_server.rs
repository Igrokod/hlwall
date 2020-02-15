use std::io;
use tokio::net::{ToSocketAddrs, UdpSocket};
use log::debug;

const A2S_INFO_REQUEST: &[u8] = b"\xff\xff\xff\xffTSource Engine Query\0";

pub(crate) struct RemoteServer {
    socket: UdpSocket,
}

impl RemoteServer {
    pub(crate) async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;

        Ok(RemoteServer { socket })
    }

    pub(crate) async fn request_info(&mut self) -> io::Result<Vec<u8>> {
        debug!("Requesting info update from remote server");
        self.socket.send(A2S_INFO_REQUEST).await?;

        let mut buf = [0; 1024];
        let bytes_read = self.socket.recv(&mut buf).await?;

        Ok((&buf[0..bytes_read]).to_owned())
    }
}
