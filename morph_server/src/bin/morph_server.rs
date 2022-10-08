use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;
use morph_server::{serve, ServerConfig};

#[derive(Parser)]
struct Opts {
    /// Set the socket address of the server API.
    #[clap(short, long, default_value = "0.0.0.0:55555")]
    api_addr: SocketAddr,
    /// Set the socket address of the Lindera server.
    #[clap(short, long, default_value = "0.0.0.0:3333")]
    lindera_addr: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let opts: Opts = Opts::parse();

    serve(ServerConfig {
        addr: opts.api_addr,
        lindera_addr: opts.lindera_addr,
    })
    .await;

    Ok(())
}
