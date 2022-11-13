mod models;
use std::io::ErrorKind;

use models::{blockchain::Blockchain, p2p::Event};
// use log::{debug, info, warn};
use pretty_env_logger;
use tokio::{spawn, sync::mpsc};

use crate::models::p2p::P2P;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let (tx, rx) = mpsc::unbounded_channel::<Event>();

    let blockchain = Blockchain::new(2, tx.clone()).await;

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
        // panic!("Breaking program due to fatal error.");
    }

    let mut blockchain = blockchain.unwrap();
    let mut p2p = P2P::new(&mut blockchain, rx);

    let daemon_handle = spawn(async move {
        p2p.daemon().await;
    });

    let handle = spawn(async move {
        blockchain.add_block("The vampire feed on the wars of mankind.".to_string());
    });

    daemon_handle.await.unwrap();
    handle.await.unwrap();

    // debug!("state of blockchain: {:#?}", blockchain);
    // debug!("checking if blockchain is valid");

    // match blockchain_state {
    //     Ok(_) => info!("Blockchain is valid."),
    //     Err(e) => warn!("Your blockchain is FAKE bro. {}", e),
    // };
}
