mod models;
// use log::{debug, info, warn};
use pretty_env_logger;

use crate::models::p2p::P2P;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // run this command (after building):
    // RUST_LOG=blockchain=info RUST_LOG=blockchain=debug ./target/debug/blockchain

    let mut p2p = P2P::new();

    p2p.blockchain.add_block("genesis".to_string());
    p2p.blockchain
        .add_block("Du wirst der Beste sein.".to_string());

    p2p.daemon().await;

    // debug!("state of blockchain: {:#?}", blockchain);
    // debug!("checking if blockchain is valid");

    // let blockchain_state = blockchain::Blockchain::validate(&blockchain);

    // match blockchain_state {
    //     Ok(_) => info!("Blockchain is valid."),
    //     Err(e) => warn!("Your blockchain is FAKE bro. {}", e),
    // };
}
