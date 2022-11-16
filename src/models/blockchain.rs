use super::{block::Block, p2p::Event};
use chrono::prelude::*;
use log::{debug, error, info, warn};
use speedy::{Readable, Writable};
use tokio::{
    fs::{File, OpenOptions},
    io::{self, AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};

#[derive(Clone)]
pub struct Blockchain {
    pub difficulty: usize,
    pub tx: mpsc::UnboundedSender<Event>,
}

impl Blockchain {
    pub async fn open() -> File {
        OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open("blockchain")
            .await
            .expect("to open blockchain file")
    }
    pub async fn new(difficulty: usize, tx: mpsc::UnboundedSender<Event>) -> io::Result<Self> {
        let mut buf = Vec::new();
        let mut chain: Vec<Block> = Vec::new();

        let mut file = self::Blockchain::open().await;

        file.read_to_end(&mut buf).await?;

        if buf.len() == 0 {
            let genesis = Block {
                id: 0,
                timestamp: Utc::now().timestamp_millis() as u64,
                nonce: u64::default(),
                previous_hash: String::default(),
                hash: "0".to_string(),
                data: "Genesis".to_string(),
            };
            // Create chain starting from the genesis chain.
            chain.push(genesis.clone());

            // Write the data into the blockchain file.
            let chain_bytes = chain.write_to_vec()?;
            file.write_all(&chain_bytes[..]).await?;
        }

        let blockchain = Blockchain { difficulty, tx };

        Ok(blockchain)
    }
    pub async fn read_all_buf(&mut self) -> io::Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::new();
        Blockchain::open().await.read_to_end(&mut buf).await?;

        // println!("read_all_buf buffer: {:?}", buf);

        Ok(buf)
    }
    async fn read_all(&mut self) -> io::Result<Vec<Block>> {
        let buf: Vec<u8> = self.read_all_buf().await.expect("read all buf");
        let chain = Vec::<Block>::read_from_buffer(&buf[..]).expect("to read from buffer");

        Ok(chain)
    }
    // a block will only be pushed to the blockchain,
    // once it has been validated and mined.
    pub async fn add_block(&mut self, data: String) {
        let mut blockchain = self
            .read_all()
            .await
            .expect("to read blockchain before add block");

        println!("\ninitial state of the blockchain: {:#?}", blockchain);

        let mut new_block = Block::new(
            blockchain.len() as u64,
            blockchain.last().unwrap().hash.clone(),
            data,
        );

        match new_block.validate(self).await {
            Ok(_) => {
                new_block.mine(self);

                println!("block is valid ----------------");

                blockchain.push(new_block.clone());

                let chain = blockchain.write_to_vec().unwrap();

                if let Err(_) = self.tx.send(Event::BlockMined(chain)) {
                    error!("Failed to send event to the network that the block was mined.");
                };

                debug!("New block added to chain -> {:?}", new_block);
                info!("Block with id: {} was added to the chain.", new_block.id);
            }
            Err(_) => {
                println!("block is invalid!!");
                warn!("Could not add new block to the blockchain.");
            }
        }
    }
    // always choose the longest chain
    pub async fn choose_chain(
        &self,
        local: &mut Vec<u8>,
        remote: &mut Vec<u8>,
    ) -> Result<Vec<Block>, String> {
        let local = Vec::<Block>::read_from_buffer(&mut local[..]).unwrap();
        let remote = Vec::<Block>::read_from_buffer(&mut remote[..]).unwrap();

        let is_local_valid = Block::validate_all(&local).is_ok();
        let is_remote_valid = Block::validate_all(&remote).is_ok();

        if is_local_valid && is_remote_valid {
            if local.len() > remote.len() {
                return Ok(local);
            } else {
                return Ok(remote);
            }
        } else if is_local_valid {
            return Ok(local);
        } else {
            return Ok(remote);
        }
    }
    pub async fn get_latest_block(&mut self) -> Result<Block, io::Error> {
        let chain = self.read_all().await?;

        let latest = chain.last().unwrap().to_owned();

        Ok(latest)
    }
    // pub async fn get_previous_block(&mut self) -> Result<Block, io::Error> {
    //     let chain = self.read_all().await?;

    //     let previous_block = if chain.last().unwrap().id == 0 { chain.last }

    //     Ok(previous_block)
    // }
    // Validate entire blockchain
    pub async fn validate(&mut self) -> Result<(), String> {
        // read blockchain from the file.
        let blockchain = self.read_all().await.unwrap();

        if blockchain.len() < 1 {
            return Err("Blockchain has zero blocks and need at least 1 block.".to_string());
        };

        for i in 0..blockchain.len() {
            let block = blockchain.get(i);

            // genesis block cant be validated
            if i == 0 {
                continue;
            };

            if let Some(curr_block) = block {
                let result = curr_block.validate(self).await;
                if result.is_err() {
                    return Err(format!("Block with id {i} is invalid."));
                }
            }
        }
        Ok(())
    }
}
