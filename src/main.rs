use hlwall::CachingServer;
use hlwall::ListenServer;
use hlwall::RemoteServer;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = env!("CARGO_PKG_DESCRIPTION"))]
struct Opt {
    /// Cache expiration in milliseconds
    #[structopt(short, long, default_value = "200")]
    ttl: u64,

    /// Interface on which we will listen/answer
    #[structopt(short, long, default_value = "127.0.0.1:27016")]
    listen: String,

    /// Target server we cache queries from
    #[structopt(name = "TARGET_SERVER", default_value = "127.0.0.1:27015")]
    target: String,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Opt::from_args();

    // The target hlds server
    let remote_server = RemoteServer::connect(config.target)?;
    let cache_duration = Duration::from_millis(config.ttl);
    let caching_server = CachingServer::new(remote_server, cache_duration);

    ListenServer::bind(config.listen, caching_server)?.serve()
}
