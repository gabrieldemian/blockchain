use super::{block::Block, p2p::Event};
use chrono::prelude::*;
use log::{debug, error, info, warn};
use tokio::sync::mpsc;

type Blocks = Vec<Block>;

#[derive(Clone)]
pub struct Blockchain {
    pub chain: Blocks,
    // Minimum amount of work required to validate a block.
    pub difficulty: usize,
    pub tx: mpsc::UnboundedSender<Event>,
}

impl Blockchain {
    pub fn new(difficulty: usize, tx: mpsc::UnboundedSender<Event>) -> Self {
        let genesis = Block {
            id: 0,
            timestamp: Utc::now().timestamp_millis() as u64,
            nonce: u64::default(),
            previous_hash: String::default(),
            hash: "000000000".to_string(),
            data: "Genesis".to_string(),
        };
        // Create chain starting from the genesis chain.
        let mut chain = Vec::new();
        chain.push(genesis.clone());

        let blockchain = Blockchain {
            chain,
            difficulty,
            tx,
        };
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

        match new_block.validate(self) {
            Ok(_) => {
                new_block.mine(self);
                self.chain.push(new_block.clone());

                if let Err(_) = self.tx.send(Event::Liebe) {
                    error!("Failed to send event to the network that the block was mined.");
                };

                debug!("New block added to chain -> {:?}", new_block);
                info!("Block with id: {} was added to the chain.", new_block.id);
            }
            Err(_) => {
                warn!("Could not add new block to the blockchain.");
            }
        }
    }
    // Validate entire blockchain
    pub fn validate(&self) -> Result<(), String> {
        if self.chain.len() < 1 {
            return Err("Blockchain has zero blocks and need at least 1 block.".to_string());
        };
        for i in 0..self.chain.len() {
            // genesis block cant be validated
            if i == 0 {
                continue;
            };

            let curr_block = self.chain.get(i);

            match curr_block {
                Some(block) => {
                    let result = Block::validate(block, self);

                    if let Some(_) = result.err() {
                        return Err(format!("Block with id {i} is invalid."));
                    }
                }
                None => return Err(format!("Could not get the block {i}")),
            };
        }
        Ok(())
    }
    // always choose the longest chain
    pub fn choose_chain(&self, local: Blockchain, remote: Blockchain) -> Self {
        let is_local_valid = local.validate().is_ok();
        let is_remote_valid = remote.validate().is_ok();

        if is_local_valid && is_remote_valid {
            if local.chain.len() > remote.chain.len() {
                return local;
            } else {
                return remote;
            }
        } else if is_local_valid {
            return local;
        } else {
            return remote;
        }
    }
}
