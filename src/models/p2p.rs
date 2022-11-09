// use super::blockchain::Blockchain;

use libp2p::{
    core::transport::MemoryTransport,
    gossipsub::{Gossipsub, GossipsubConfig, IdentTopic, MessageAuthenticity},
    identity, mplex, multiaddr,
    noise::NoiseAuthenticated,
    Multiaddr, PeerId, Swarm, Transport,
};

// defines the behaviour of the current peer
// on the network
pub struct P2P {
    swarm: Swarm<Gossipsub>,
}

impl P2P {
    pub fn new() -> Self {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        // encrypted TCP transport over mplex
        let transport = MemoryTransport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(NoiseAuthenticated::xx(&local_key).unwrap())
            .multiplex(mplex::MplexConfig::new())
            .boxed();

        // Set the message authenticity - How we expect to publish messages
        // Here we expect the publisher to sign the message with their key.
        let message_authenticity = MessageAuthenticity::Signed(local_key);

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
            Swarm::new(transport, gossipsub, local_peer_id)
        };

        // thisd doesnt work: multiaddr not supported
        // why?
        // swarm
        //     .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        //     .unwrap();

        let memory: Multiaddr = multiaddr::Protocol::Memory(10).into();
        let addr = swarm.listen_on(memory).unwrap();

        println!("Listening on {:?}", addr);

        Self { swarm }
    }
}
