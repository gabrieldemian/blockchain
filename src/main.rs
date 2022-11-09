mod models;
use std::error::Error;

use log::{debug, info, warn};
use models::*;
use pretty_env_logger;

use crate::models::p2p::P2P;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    // run this command (after building):
    // RUST_LOG=blockchain=info RUST_LOG=blockchain=debug ./target/debug/blockchain

    let difficulty = 2;
    let mut blockchain = blockchain::Blockchain::new(difficulty);
    let p2p = P2P::new();

    blockchain::Blockchain::add_block(&mut blockchain, "erste Block".to_string());
    blockchain::Blockchain::add_block(&mut blockchain, "Ich liebe Fleisch".to_string());
    // blockchain::Blockchain::add_block(&mut blockchain, "und du?".to_string());

    // debug!("state of blockchain: {:#?}", blockchain);
    // debug!("checking if blockchain is valid");

    // let blockchain_state = blockchain::Blockchain::validate(&blockchain);

    // match blockchain_state {
    //     Ok(_) => info!("Blockchain is valid."),
    //     Err(e) => warn!("Your blockchain is FAKE bro. {}", e),
    // };

    Ok(())
}
