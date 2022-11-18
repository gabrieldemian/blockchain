use blockchain::models::p2p::P2P;
use pretty_env_logger;
use tokio::spawn;

#[tokio::main]
async fn main() {
    // RUST_LOG=info cargo run
    pretty_env_logger::init();

    let mut p2p = P2P::new().await;

    let daemon_handle = spawn(async move {
        p2p.daemon().await;
    });

    // let handle = spawn(async move {});

    daemon_handle.await.unwrap();
    // handle.await.unwrap();
}
