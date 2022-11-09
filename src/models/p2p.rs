// use super::blockchain::Blockchain;
use libp2p::{
    core::upgrade,
    gossipsub::{Gossipsub, GossipsubConfig, IdentTopic, MessageAuthenticity},
    identity::Keypair,
    mdns::{Mdns, MdnsConfig},
    mplex,
    noise::NoiseAuthenticated,
    tcp::{self, GenTcpConfig},
    Multiaddr, NetworkBehaviour, PeerId, Swarm, Transport,
};
use once_cell::sync::Lazy;

static LOCAL_KEY: Lazy<Keypair> = Lazy::new(|| Keypair::generate_ed25519());
static LOCAL_PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(LOCAL_KEY.public()));

// defines the behaviour of the current peer
// on the network
#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    gossipsub: Gossipsub,
    mdns: Mdns,
}

pub struct P2P {
    pub swarm: Swarm<AppBehaviour>,
}

impl P2P {
    pub fn new() -> Self {
        // encrypted TCP transport over mplex

        let transport_config = GenTcpConfig::new().port_reuse(true);
        let transport = tcp::TokioTcpTransport::new(transport_config)
            .upgrade(upgrade::Version::V1)
            .authenticate(
                NoiseAuthenticated::xx(&LOCAL_KEY)
                    .expect("Signing libp2p-noise static DH keypair failed."),
            )
            .multiplex(mplex::MplexConfig::new())
            .boxed();

        // Set the message authenticity - How we expect to publish messages
        // Here we expect the publisher to sign the message with their key.
        let message_authenticity = MessageAuthenticity::Signed(LOCAL_KEY.clone());

        let gossipsub_config = GossipsubConfig::default();
        let mut gossipsub = Gossipsub::new(message_authenticity, gossipsub_config)
            .expect("could not create gossipsub");

        // Create a Gossipsub topic
        let topic = IdentTopic::new("gossip");

        gossipsub
            .subscribe(&topic)
            .expect("could not subscribe to topic");

        let mdns = Mdns::new(MdnsConfig::default()).unwrap();
        let behaviour = AppBehaviour { gossipsub, mdns };

        // Create a Swarm to manage peers and events
        let mut swarm = Swarm::new(transport, behaviour, LOCAL_PEER_ID.clone());

        let addr: Multiaddr = "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("could not parse multiaddr");

        println!("Listening on {:?}", addr);
        println!("peerID {:?}", swarm.local_peer_id());

        swarm.listen_on(addr).expect("could not listen on swarm");

        swarm
            .connected_peers()
            .for_each(|peer| println!("peer {}", peer));

        Self { swarm }
    }
}
