// use super::blockchain::Blockchain;

use libp2p::{
    core::{transport::MemoryTransport, upgrade},
    gossipsub::{Gossipsub, GossipsubConfig, IdentTopic, MessageAuthenticity},
    identity::Keypair,
    mplex, multiaddr,
    noise::NoiseAuthenticated,
    PeerId, Swarm, Transport,
};
use once_cell::sync::Lazy;

static LOCAL_KEY: Lazy<Keypair> = Lazy::new(|| Keypair::generate_ed25519());
static LOCAL_PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(LOCAL_KEY.public()));

// defines the behaviour of the current peer
// on the network
pub struct P2P {
    swarm: Swarm<Gossipsub>,
}

impl P2P {
    pub fn new() -> Self {
        // encrypted TCP transport over mplex
        let transport = MemoryTransport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(NoiseAuthenticated::xx(&LOCAL_KEY).unwrap())
            .multiplex(mplex::MplexConfig::new())
            .boxed();

        // Set the message authenticity - How we expect to publish messages
        // Here we expect the publisher to sign the message with their key.
        let message_authenticity = MessageAuthenticity::Signed(LOCAL_KEY.clone());

        // Create a Gossipsub topic
        let topic = IdentTopic::new("test-net");

        // Create a Swarm to manage peers and events
        let mut swarm = {
            let gossipsub_config = GossipsubConfig::default();
            let mut gossipsub: Gossipsub = Gossipsub::new(message_authenticity, gossipsub_config)
                .expect("could not create gossipsub");
            gossipsub
                .subscribe(&topic)
                .expect("could not subscribe to topic");
            Swarm::new(transport, gossipsub, LOCAL_PEER_ID.clone())
        };

        // thisd doesnt work: multiaddr not supported
        // why?
        // swarm
        //     .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        //     .unwrap();

        let memory = multiaddr::Protocol::Memory(10).into();
        let addr = swarm.listen_on(memory).unwrap();

        println!("Listening on {:?}", addr);

        Self { swarm }
    }
}
