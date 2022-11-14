use super::{block::Block, p2p::Event};
use chrono::prelude::*;
use log::{debug, error, info, warn};
use speedy::Writable;
use tokio::{
    fs::{File, OpenOptions},
    io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    sync::mpsc,
};

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub tx: mpsc::UnboundedSender<Event>,
    pub rd: ReadHalf<File>,
    pub wt: WriteHalf<File>,
}

impl Blockchain {
    pub async fn open() -> io::Result<(ReadHalf<File>, WriteHalf<File>)> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open("blockchain")
            .await?;
        Ok(io::split(file))
    }
    pub async fn new(difficulty: usize, tx: mpsc::UnboundedSender<Event>) -> io::Result<Self> {
        let mut buf = Vec::new();
        let mut chain: Vec<Block> = Vec::new();

        let (mut rd, mut wt) = self::Blockchain::open().await.unwrap();

        rd.read_to_end(&mut buf).await?;

        if buf.len() == 0 {
            let genesis = Block {
                id: 0,
                timestamp: Utc::now().timestamp_millis() as u64,
                nonce: u64::default(),
                previous_hash: String::default(),
                hash: "000000000".to_string(),
                data: "Genesis".to_string(),
            };
            // Create chain starting from the genesis chain.
            chain.push(genesis.clone());

            // Write the data into the blockchain file.
            let chain_bytes = chain.write_to_vec()?;
            wt.write_all(&chain_bytes[..]).await?;

            println!("chain: {:?}", chain);
            println!("chain bytes: {:?}", chain_bytes);
        }

        let blockchain = Blockchain {
            chain,
            difficulty,
            tx,
            rd,
            wt,
        };

        Ok(blockchain)
    }
    // a block will only be pushed to the blockchain,
    // once it has been validated and mined.
    pub fn add_block(&mut self, data: String) {
        let last_block = self.get_previous_block();

        print!("last block is {:?}", self.chain);

        let mut new_block = Block::new(
            self.chain.len() as u64,
            self.chain.last().unwrap().hash.clone(),
            data,
        );

        match new_block.validate(self) {
            Ok(_) => {
                new_block.mine(self);
                self.chain.push(new_block.clone());

                let chain = self.chain.write_to_vec().unwrap();

                if let Err(_) = self.tx.send(Event::BlockMined(chain)) {
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
    // always choose the longest chain
    pub fn choose_chain(&self, local: Blockchain, remote: Blockchain) -> Self {
        let is_local_valid = local.validate().is_ok();
        let is_remote_valid = local.validate().is_ok();

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
    pub fn get_previous_block(&self) -> Option<&Block> {
        let last = self.chain.last();

        let previous_block = if self.chain.len() < 2 {
            last
        } else {
            self.chain.get((last.unwrap().id - 1 as u64) as usize)
        };

        previous_block
    }
    // Validate entire blockchain
    pub fn validate(&self) -> Result<(), String> {
        let chain = &self.chain;

        if chain.len() < 1 {
            return Err("Blockchain has zero blocks and need at least 1 block.".to_string());
        };

        for i in 0..chain.len() {
            // genesis block cant be validated
            if i == 0 {
                continue;
            };

            let curr_block = chain.get(i);

            if let Some(curr_block) = curr_block {
                let result = curr_block.validate(self);
                if result.is_err() {
                    return Err(format!("Block with id {i} is invalid."));
                }
            }
        }
        Ok(())
    }
}
