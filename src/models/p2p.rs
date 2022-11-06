use async_std::io;
use futures::{prelude::*, select};
use libp2p::gossipsub::{
    Gossipsub, GossipsubEvent, GossipsubMessage, IdentTopic as Topic, MessageAuthenticity,
    MessageId, ValidationMode,
};
use libp2p::{
    gossipsub, identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::SwarmEvent,
    NetworkBehaviour, PeerId, Swarm,
};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use super::blockchain::Blockchain;

pub static KEYS: Identity::Keypair = identity::Keypair::generate_ed25519();
pub static PEER_ID: Identity::Keypair = identity::Keypair::generate_ed25519();
pub static BLOCKCHAIN_TOPIC: Topic = Topic::new("blockchain");
pub static BLOCK_TOPIC: Topic = Topic::new("block");

println!("Local peer id: {}", PEER_ID);

// defines the behaviour of the current peer
// on the network
#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    gossipsub: Gossipsub,
    mdns: Mdns,
    blockchain: Blockchain,
}

impl AppBehaviour {
    pub fn new(blockchain: &mut blockchain) -> Self {}
}
