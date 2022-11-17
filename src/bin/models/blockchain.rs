use super::{block::Block, p2p::Event};
use chrono::prelude::*;
use crossbeam_channel::Sender;
use log::{debug, error, info, warn};
use speedy::{Readable, Writable};
use tokio::{
    fs::{File, OpenOptions},
    io::{self, AsyncReadExt, AsyncWriteExt},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::Instant,
};

// #[derive(Clone)]
// pub struct Blockchain {
//     pub difficulty: usize,
// }

pub async fn open() -> File {
    OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("blockchain")
        .await
        .expect("to open blockchain file")
}
// pub async fn new(difficulty: usize) -> io::Result<Self> {
//     let mut buf = Vec::new();
//     let mut chain: Vec<Block> = Vec::new();

//     let mut file = self::open().await;

//     file.read_to_end(&mut buf).await?;

//     if buf.len() == 0 {
//         let genesis = Block {
//             id: 0,
//             timestamp: Utc::now().timestamp_millis() as u64,
//             nonce: u64::default(),
//             previous_hash: String::default(),
//             hash: "0".to_string(),
//             data: "Genesis".to_string(),
//         };
//         // Create chain starting from the genesis chain.
//         chain.push(genesis.clone());

//         // Write the data into the blockchain file.
//         let chain_bytes = chain.write_to_vec()?;
//         file.write_all(&chain_bytes[..]).await?;
//     }

//     Ok(())
// }
pub fn get_channel() -> (UnboundedSender<Event>, UnboundedReceiver<Event>) {
    mpsc::unbounded_channel::<Event>()
}
pub async fn read_all_buf() -> io::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    open().await.read_to_end(&mut buf).await?;
    // println!("read_all_buf buffer: {:?}", buf);

    Ok(buf)
}
pub async fn read_all() -> io::Result<Vec<Block>> {
    let now = Instant::now();
    let buf: Vec<u8> = read_all_buf().await.expect("read all buf");
    let chain = Vec::<Block>::read_from_buffer(&buf[..]).expect("to read from buffer");
    info!(
        "took {}μs to read the blockchain.",
        now.elapsed().as_micros()
    );

    Ok(chain)
}
// a block will only be pushed to the blockchain,
// once it has been validated and mined.
pub async fn add_block(data: String, tx: Sender<Event>) {
    let mut blockchain = read_all()
        .await
        .expect("to read blockchain before add block");

    debug!("\n blockchain before mining new block: {:#?}", blockchain);
    info!(
        "Received block with id \"{}\" and data: \"{}\"",
        blockchain.len(),
        data
    );

    let mut new_block = Block::new(
        blockchain.len() as u64,
        blockchain.last().unwrap().hash.clone(),
        data,
    );

    match new_block.validate().await {
        Ok(_) => {
            new_block.mine();

            debug!("block to be added is valid");

            blockchain.push(new_block.clone());

            let chain = blockchain.write_to_vec().unwrap();

            if let Err(e) = tx.send(Event::BlockMined(chain)) {
                error!(
                    "Failed to send event to the network that the block was mined. Reason: {}",
                    e.to_string()
                );
            };
        }
        Err(_) => {
            debug!("block is invalid!");
            warn!("Could not add new block to the blockchain.");
        }
    }
}
// always choose the longest chain
pub async fn choose_chain(local: &mut Vec<u8>, remote: &mut Vec<u8>) -> Result<Vec<Block>, String> {
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
pub async fn get_latest_block() -> Result<Block, io::Error> {
    let chain = read_all().await?;

    let latest = chain.last().unwrap().to_owned();

    Ok(latest)
}
// Validate entire blockchain
pub async fn validate() -> Result<(), String> {
    // read blockchain from the file.
    let blockchain = read_all().await.unwrap();

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
            let result = curr_block.validate().await;
            if result.is_err() {
                return Err(format!("Block with id {i} is invalid."));
            }
        }
    }
    Ok(())
}
