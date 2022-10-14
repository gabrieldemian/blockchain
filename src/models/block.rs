use super::blockchain::Blockchain;
use chrono::prelude::*;
use log::{info, trace};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: String,
    pub nonce: u64,
}

impl Block {
    // Create a new block. The hash will be calculated and set automatically.
    pub fn new(id: u64, previous_hash: String, data: String) -> Self {
        trace!("creating block with id: {}", id);

        Block {
            id,
            hash: String::default(),
            previous_hash,
            timestamp: Utc::now().timestamp_millis() as u64,
            data,
            nonce: u64::default(),
        }
    }
    pub fn calculate_hash(&self) -> String {
        let mut block_data = self.clone();
        block_data.hash = String::default();

        let serialized_block_data = serde_json::to_string(&block_data).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(serialized_block_data);

        let result = hasher.finalize();

        format!("{:x}", result)
    }
    pub fn mine(&mut self, blockchain: Blockchain) {
        info!("mining block...");

        loop {
            if !self.hash.starts_with(&"0".repeat(blockchain.difficulty)) {
                self.nonce += 1;
                self.hash = self.calculate_hash();
            } else {
                info!("block mined! nonce found: {}", self.nonce);
                break;
            }
        }
    }
}
