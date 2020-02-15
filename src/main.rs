mod codec;
mod util;

use codec::GoldSourceQuery;
use codec::GoldSourceQueryCodec;
use futures::{SinkExt, StreamExt};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

#[tokio::main]
async fn main() {
    let codec = GoldSourceQueryCodec::default();
    let socket = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to udp socket");
    let mut connection = UdpFramed::new(socket, codec);
    connection
        .send((
            GoldSourceQuery::A2sInfoRequest,
            "127.0.0.1:27015".parse().unwrap(),
        ))
        .await
        .unwrap();
    dbg!(connection.next().await);
}
