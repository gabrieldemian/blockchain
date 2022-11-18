// use crossbeam_channel::{unbounded, Receiver, Sender};
use libp2p::{gossipsub::IdentTopic, identity::Keypair, PeerId};
use once_cell::sync::Lazy;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use self::p2p::Event;

// pub static LOCAL_KEY: Lazy<Keypair> = Lazy::new(|| Keypair::generate_ed25519());
// pub static LOCAL_PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(LOCAL_KEY.public()));
pub static TOPIC: Lazy<IdentTopic> = Lazy::new(|| IdentTopic::new("gossip"));

pub static mut CHANNEL: Lazy<(UnboundedSender<Event>, UnboundedReceiver<Event>)> =
    Lazy::new(|| mpsc::unbounded_channel::<Event>());

pub mod block;
pub mod blockchain;
pub mod p2p;
