use std::{env, future::Future, net::SocketAddr};

use morph_client::AsyncClient;
use morph_server::{serve, ServerConfig};

pub async fn test_with_server<F, Fut>(test: F) -> anyhow::Result<()>
where
    F: FnOnce(SocketAddr) -> Fut,
    Fut: Future<Output = ()>,
{
    let api_addr = get_unused_tcp_socket_addr();
    let lindera_addr = env::var("LINDERA_ADDR")
        .unwrap_or("0.0.0.0:3333".to_owned())
        .parse()
        .unwrap();
    let api_config = morph_server::ServerConfig {
        addr: api_addr,
        lindera_addr,
    };

    let server = spawn_server(api_config).await.unwrap();

    tokio::select! {
        _ = server => {
            panic!("the server terminated before the test finished");
        }
        _ = test(api_addr) => {}
    };

    Ok(())
}

/// Returns an unused IPv4 TCP socket address on the loopback interface.
fn get_unused_tcp_socket_addr() -> SocketAddr {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}

/// Spawns a server and waits for its API to be ready.
pub async fn spawn_server(api_config: ServerConfig) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let server = { tokio::spawn(serve(api_config)) };

    // Wait for the server to serve its API.
    // Abort if a connection can not be established within 2 seconds.
    let mut attempts = 0;
    loop {
        match AsyncClient::connect(api_config.addr).await {
            Ok(_) => break,
            Err(e) => {
                attempts += 1;

                if attempts == 16 {
                    anyhow::bail!(e);
                } else {
                    tokio::time::sleep(std::time::Duration::from_millis(125)).await;
                    continue;
                }
            }
        }
    }

    Ok(server)
}
