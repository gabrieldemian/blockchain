// use std::collections::hash_map::DefaultHasher;

// use super::blockchain::Blockchain;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// use libp2p::gossipsub::MessageId;
// use libp2p::gossipsub::{
//     Gossipsub, GossipsubEvent, GossipsubMessage, IdentTopic as Topic, MessageAuthenticity,
//     ValidationMode,
// };
use libp2p::{
    gossipsub::{Gossipsub, GossipsubMessage, MessageId},
    identity,
    mdns::{GenMdns, Mdns, MdnsConfig, MdnsEvent},
    swarm::SwarmEvent,
    NetworkBehaviour, PeerId, Swarm,
};

// defines the behaviour of the current peer
// on the network
#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    gossipsub: Gossipsub,
    mdns: Mdns,
    // blockchain: Blockchain,
}

impl AppBehaviour {
    // To content-address message, we can take the hash of message and use it as an ID.
    pub fn get_msg_id(msg: GossipsubMessage) -> MessageId {
        let mut s = DefaultHasher::new();
        msg.data.hash(&mut s);
        MessageId::from(s.finish().to_string())
    }

    // pub fn new(blockchain: &mut Blockchain) -> Self {
    //     // Set a custom gossipsub configuration
    //     let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
    //         .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
    //         .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
    //         .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
    //         .build()
    //         .expect("Valid config");

    //     let gossipsub = Gossipsub::new();
    // }
}
