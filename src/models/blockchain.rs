use chrono::prelude::*;
// Internal module
use super::block::Block;
use log::debug;

type Blocks = Vec<Block>;

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub chain: Blocks,
    // Minimum amount of work required to validate a block.
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let genesis_block = Block {
            id: 0,
            timestamp: Utc::now().timestamp_millis() as u64,
            nonce: u64::default(),
            previous_hash: String::default(),
            hash: "000000000".to_string(),
            data: "gott mit uns.".to_string(),
        };
        // Create chain starting from the genesis chain.
        let mut chain = Vec::new();
        chain.push(genesis_block.clone());
        let blockchain = Blockchain { chain, difficulty };
        blockchain
    }
    // a block will only be pushed to the blockchain,
    // once it has been validated and mined.
    pub fn add_block(&mut self, data: String) {
        let mut new_block = Block::new(
            self.chain.len() as u64,
            self.chain.last().unwrap().hash.clone(),
            data,
        );
        // todo: validate block before minning
        new_block.mine(self.clone());
        self.chain.push(new_block.clone());

        debug!("New block added to chain -> {:?}", new_block);
    }
}
