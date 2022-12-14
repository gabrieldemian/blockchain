use super::blockchain;
use chrono::prelude::*;
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
use speedy::{Readable, Writable};
use tokio::time::Instant;

static DIFFICULTY: Lazy<usize> = Lazy::new(|| 4);

#[derive(Debug, Clone, Writable, Readable)]
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

        let serialized_block_data = block_data.write_to_vec().unwrap();

        let mut hasher = Sha256::new();
        hasher.update(serialized_block_data);

        let result = hasher.finalize();

        format!("{:x}", result)
    }
    pub fn mine(&mut self) {
        let now = Instant::now();
        loop {
            if !self.hash.starts_with(&"0".repeat(*DIFFICULTY)) {
                self.nonce += 1;
                self.hash = self.calculate_hash();
            } else {
                info!(
                    "block mined in {}s with nonce: \"{}\"",
                    now.elapsed().as_secs(),
                    self.nonce
                );
                break;
            }
        }
    }
    pub async fn validate(&self) -> Result<(), String> {
        let previous_block = blockchain::get_latest_block()
            .await
            .map_err(|e| e.to_string())?;

        debug!("-- validating new block --");
        debug!("prev {:#?}", previous_block);
        debug!("curent to be added {:#?}", self);

        if self.previous_hash != previous_block.hash {
            warn!("block with id: {} passed invalid previous_hash.", self.id);
            return Err("block passed invalid previous_hash.".to_string());
        }
        if self.id != previous_block.id + 1 {
            warn!("invalid block id: {}", self.id);
            return Err("invalid block id.".to_string());
        }
        info!("valid block, beginning to mine now...");
        return Ok(());
    }
    pub fn validate_all(blocks: &Vec<Block>) -> Result<(), String> {
        for i in 0..blocks.len() {
            // genesis block cant be validated
            if i == 0 {
                continue;
            };

            let current_block = blocks.get(i);

            let previous_block = blocks
                .get((current_block.unwrap().id - 1 as u64) as usize)
                .unwrap()
                .to_owned();

            // println!("current {:#?}", current_block.unwrap());
            // println!("previous {:#?}", previous_block);

            if let Some(current_block) = current_block {
                if current_block.previous_hash != previous_block.hash {
                    warn!(
                        "block with id: {} passed invalid previous_hash.",
                        current_block.id
                    );
                    return Err("block passed invalid previous_hash.".to_string());
                }
                if current_block.id != previous_block.id + 1 {
                    warn!("invalid block id: {}", current_block.id);
                    return Err("invalid block id.".to_string());
                }
            } else {
                return Err("Could not get block with id {i}".to_string());
            }
        }
        Ok(())
    }
}
