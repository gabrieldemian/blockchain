mod models;
use log::debug;
use models::*;
use pretty_env_logger;

fn main() {
    pretty_env_logger::init();

    // run this command (after building):
    // RUST_LOG=blockchain=info RUST_LOG=blockchain=debug ./target/debug/blockchain

    let difficulty = 2;
    let mut blockchain = blockchain::Blockchain::new(difficulty);

    blockchain::Blockchain::add_block(&mut blockchain, "erste Block".to_string());
    blockchain::Blockchain::add_block(&mut blockchain, "Ich liebe Fleisch".to_string());
    blockchain::Blockchain::add_block(&mut blockchain, "und du?".to_string());

    debug!("state of blockchain: {:#?}", blockchain);
}
