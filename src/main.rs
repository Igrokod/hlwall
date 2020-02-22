mod caching_server;
mod listen_server;
mod packet;
mod remote_server;

use caching_server::CachingServer;
use listen_server::ListenServer;
use remote_server::RemoteServer;
use std::time::Duration;

const CACHE_REQUESTS_FOR: u64 = 3;

#[tokio::main(basic_scheduler)]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // The target hlds server
    let remote_server = RemoteServer::connect("127.0.0.1:27015").await?;
    // TODO: Ping target hlds server to reduce outage

    let cache_duration = Duration::from_secs(CACHE_REQUESTS_FOR);
    let caching_server = CachingServer::new(remote_server, cache_duration);

    let bind_to = "127.0.0.1:27016";
    ListenServer::bind(bind_to, caching_server)
        .await?
        .serve()
        .await
}
