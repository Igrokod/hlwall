mod packet;
mod remote_server;
mod listen_server;

use remote_server::RemoteServer;
use listen_server::ListenServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let remote_server = RemoteServer::connect("127.0.0.1:27015").await?;
    let listen_server = ListenServer::bind("0.0.0.0:27016").await?;
    listen_server.serve(remote_server).await?;

    Ok(())
}
