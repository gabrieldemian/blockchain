mod models;
use std::io::ErrorKind;

use models::{blockchain::Blockchain, p2p::Event};
use pretty_env_logger;
use tokio::{spawn, sync::mpsc};

use crate::models::p2p::P2P;

#[tokio::main]
async fn main() {
    // RUST_LOG=info cargo run
    pretty_env_logger::init();

    let (tx, rx) = mpsc::unbounded_channel::<Event>();

    let blockchain = Blockchain::new(4, tx.clone()).await;

    if let Err(ref e) = blockchain {
        match e.kind() {
            ErrorKind::PermissionDenied => {
                println!(
                    "The program does not have permission to write data to the blockchain file."
                )
            }
            _ => {
                println!("Untreated error while reading blockchain file.");
                println!("err: {}", e);
            }
        }
        panic!("Breaking program due to fatal error.");
    }

    let blockchain = blockchain.unwrap();
    let mut p2p = P2P::new(rx, blockchain);

    let daemon_handle = spawn(async move {
        p2p.daemon().await;
    });

    daemon_handle.await.unwrap();
}
