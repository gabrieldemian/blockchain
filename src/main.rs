mod models;
// use log::{debug, info, warn};
use pretty_env_logger;
use tokio::spawn;

use crate::models::p2p::P2P;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut p2p = P2P::new();

    let daemon_handle = spawn(async move {
        p2p.daemon().await;
    });

    let handle = spawn(async move {
        println!("after daemon");
    });

    daemon_handle.await.unwrap();
    handle.await.unwrap();

    // debug!("state of blockchain: {:#?}", blockchain);
    // debug!("checking if blockchain is valid");

    // let blockchain_state = blockchain::Blockchain::validate(&blockchain);

    // match blockchain_state {
    //     Ok(_) => info!("Blockchain is valid."),
    //     Err(e) => warn!("Your blockchain is FAKE bro. {}", e),
    // };
}
