mod packet;

use bytes::Bytes;
use log::{debug, info, log_enabled, trace, Level};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut listen_socket = UdpSocket::bind("0.0.0.0:27016").await.unwrap();

    let mut server_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    server_socket.connect("127.0.0.1:27015").await.unwrap();

    let mut buf = [0u8; 1024];
    loop {
        let (bytes_read, addr) = listen_socket.recv_from(&mut buf).await.unwrap();
        info!("Request from {}", addr);

        debug!("Calling remote server for fresh data");
        server_socket.send(&buf[0..bytes_read]).await.unwrap();
        let bytes_read = server_socket.recv(&mut buf).await.unwrap();

        if log_enabled!(Level::Trace) {
            let bytes = Bytes::from((&buf[0..bytes_read]).to_owned());
            trace!("Bytes from remote server: {:?}", bytes);
        }

        debug!("Returning results to client");
        listen_socket
            .send_to(&buf[0..bytes_read], addr)
            .await
            .unwrap();
    }
}
