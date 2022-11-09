// use super::blockchain::Blockchain;
use async_std::io;
use futures::prelude::*;
use libp2p::{
    core::upgrade,
    futures::{select, StreamExt},
    gossipsub::{Gossipsub, GossipsubConfig, GossipsubEvent, IdentTopic, MessageAuthenticity},
    identity::Keypair,
    mdns::{MdnsEvent, TokioMdns},
    mplex,
    noise::NoiseAuthenticated,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::{self, GenTcpConfig},
    Multiaddr, NetworkBehaviour, PeerId, Swarm, Transport,
};
use once_cell::sync::Lazy;

static LOCAL_KEY: Lazy<Keypair> = Lazy::new(|| Keypair::generate_ed25519());
static LOCAL_PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(LOCAL_KEY.public()));
static TOPIC: Lazy<IdentTopic> = Lazy::new(|| IdentTopic::new("gossip"));

// defines the behaviour of the current peer
// on the network
#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    gossipsub: Gossipsub,
    mdns: TokioMdns,
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

        gossipsub
            .subscribe(&TOPIC)
            .expect("could not subscribe to topic");

        // Create a Swarm to manage peers and events
        let mut swarm = {
            let mdns = TokioMdns::new(Default::default()).unwrap();
            let behaviour = AppBehaviour { gossipsub, mdns };
            SwarmBuilder::new(transport, behaviour, LOCAL_PEER_ID.clone())
                // We want the connection background tasks to be spawned
                // onto the tokio runtime.
                .executor(Box::new(|fut| {
                    tokio::spawn(fut);
                }))
                .build()
        };

        let addr: Multiaddr = "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("could not parse multiaddr");

        println!("Listening on {:?}", addr);
        println!("{:?}", swarm.local_peer_id());

        swarm.listen_on(addr).expect("could not listen on swarm");
        swarm
            .connected_peers()
            .for_each(|peer| println!("peer: {:?}", peer));

        Self { swarm }
    }

    pub async fn listen_io(&mut self) {
        // Listen for user input
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

        // Dial the peer identified by the multi-address given as the second
        // command-line argument, if any.
        // cargo run /ip4/127.0.0.1/tcp/[port]
        if let Some(addr) = std::env::args().nth(1) {
            let remote: Multiaddr = addr.parse().unwrap();
            self.swarm.dial(remote).unwrap();
            println!("Dialed {}", addr)
        }

        // Listen for events on the P2P network, and react to them
        loop {
            select! {line = stdin.select_next_some() => {
                if let Err(e) = self.swarm
                    .behaviour_mut().gossipsub
                    .publish(TOPIC.clone(), line.expect("Stdin not to close").as_bytes()) {
                        println!("Publish error: {:?}", e);
                    }
                },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr {
                        address,
                        listener_id
                    } => {
                        println!("{:?} listening on {:?}", listener_id, address);
                    },
                    SwarmEvent::Behaviour(AppBehaviourEvent::Gossipsub(GossipsubEvent::Message {
                        message,
                        message_id,
                        propagation_source: peer
                    })) => {
                        // get the last 7 characters of the peerID
                        let peer = peer.to_string();
                        let truncated_peerid = peer[peer.len() - 7..].to_string();
                        println!(
                                "\n{truncated_peerid}: {}",
                                String::from_utf8_lossy(&message.data)
                            );
                        // self.swarm
                        //     .behaviour()
                        //     .gossipsub
                        //     .all_peers()
                        //     .for_each(|peer| println!("peer {:?}", peer));
                    },
                    SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discovered a new peer: {}", peer_id);
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discover peer has expired: {}", peer_id);
                            self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}
