mod models;

use crossbeam_channel::{unbounded, Receiver, Sender};
use models::{blockchain, p2p::Event};
use once_cell::sync::Lazy;
use pretty_env_logger;
use tokio::spawn;

use crate::models::p2p::P2P;

static CHANNEL: Lazy<(Sender<Event>, Receiver<Event>)> = Lazy::new(|| unbounded::<Event>());

#[tokio::main]
async fn main() {
    // RUST_LOG=info cargo run
    pretty_env_logger::init();

    let mut p2p = P2P::new().await;

    let daemon_handle = spawn(async move {
        p2p.daemon().await;
    });

    let handle = spawn(async {
        let s = CHANNEL.0.clone();
        blockchain::add_block("will add".to_string(), s).await;
    });

    daemon_handle.await.unwrap();
    handle.await.unwrap();
}
