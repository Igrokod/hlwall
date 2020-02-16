mod caching_server;
mod listen_server;
mod packet;
mod remote_server;

use caching_server::CachingServer;
use listen_server::ListenServer;
use remote_server::RemoteServer;
use std::time::Duration;

const CACHE_REQUESTS_FOR: u64 = 3;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cache_duration = Duration::from_secs(CACHE_REQUESTS_FOR);
    let remote_server = RemoteServer::connect("127.0.0.1:27015").await?;
    let caching_server = CachingServer::new(remote_server, cache_duration);
    let listen_server = ListenServer::bind("0.0.0.0:27016").await?;
    listen_server.serve(caching_server).await?;

    Ok(())
}
