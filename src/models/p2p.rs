use crate::models::{block::Block, blockchain, TOPIC};
use async_std::io;
use futures::prelude::*;
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    gossipsub::{Gossipsub, GossipsubConfig, GossipsubEvent, MessageAuthenticity, TopicHash},
    identity::Keypair,
    kad::{store::MemoryStore, Kademlia, KademliaEvent},
    mdns::{MdnsEvent, TokioMdns},
    mplex,
    noise::NoiseAuthenticated,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::{self, GenTcpConfig},
    Multiaddr, NetworkBehaviour, PeerId, Swarm, Transport,
};
use log::{debug, info, warn};
// use serde::{Deserialize, Serialize};
use speedy::Readable;
use tokio::{
    io::AsyncWriteExt,
    select,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::Instant,
};

// #[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec<u8>,
    // pub receiver: String,
}

pub enum Event {
    BlockMined(Vec<u8>),
    Liebe,
}

// defines the behaviour of the current peer
// on the network
#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    pub gossipsub: Gossipsub,
    pub kademlia: Kademlia<MemoryStore>,
    // mdns: TokioMdns,
}

pub struct P2P {
    pub swarm: Swarm<AppBehaviour>,
    pub local_key: PeerId,
    pub s: UnboundedSender<Event>,
    pub r: UnboundedReceiver<Event>,
}

impl P2P {
    pub async fn new() -> Self {
        let (s, r) = mpsc::unbounded_channel::<Event>();

        // let mut bytes = std::fs::read("private2.pk8").unwrap();
        // let keypair = Keypair::rsa_from_pkcs8(&mut bytes).unwrap();
        let keypair = Keypair::generate_ed25519();
        let local_key = PeerId::from(keypair.public());

        let transport_config = GenTcpConfig::new().port_reuse(true);
        let transport = tcp::TokioTcpTransport::new(transport_config)
            .upgrade(upgrade::Version::V1)
            .authenticate(
                NoiseAuthenticated::xx(&keypair)
                    .expect("Signing libp2p-noise static DH keypair failed."),
            )
            .multiplex(mplex::MplexConfig::new())
            .boxed();

        // Set the message authenticity - How we expect to publish messages
        // Here we expect the publisher to sign the message with their key.
        let message_authenticity = MessageAuthenticity::Signed(keypair.clone());

        // Peer discovery protocols.
        let mdns = TokioMdns::new(Default::default()).unwrap();
        let kademlia = Kademlia::new(local_key, MemoryStore::new(local_key));

        let gossipsub_config = GossipsubConfig::default();
        let mut gossipsub = Gossipsub::new(message_authenticity, gossipsub_config)
            .expect("could not create gossipsub");

        gossipsub
            .subscribe(&TOPIC)
            .expect("could not subscribe to topic");

        // Create a Swarm to manage peers and events
        let mut swarm = {
            // let mdns = TokioMdns::new(Default::default()).unwrap();
            let behaviour = AppBehaviour {
                gossipsub,
                // mdns,
                kademlia,
            };
            SwarmBuilder::new(transport, behaviour, local_key)
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

        Self {
            swarm,
            local_key,
            s,
            r,
        }
    }

    pub async fn daemon(&mut self) {
        // Listen for user input
        // let (s, r) = unbounded();
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

        // Dial the peer identified by the multi-address given as the second
        // command-line argument, if any.
        // cargo run /ip4/127.0.0.1/tcp/[port]
        if let Some(addr) = std::env::args().nth(1) {
            let remote: Multiaddr = addr.parse().unwrap();
            self.swarm.dial(remote).unwrap();
            println!("Dialed {}", addr);
        }

        let message =
            "Welcome! type \"ls peers/blockchain\" to list, and \"block [data]\" to send a block."
                .to_string();
        let lines: String = message.chars().map(|_| "-").collect();

        println!("\n  {lines}");
        println!("< {message} >");
        println!("  {lines}");
        println!("    \\   ^__^");
        println!("     \\  (oo)\\______");
        println!("        (__)\\      )\\/\\");
        println!("           ||----w |");
        println!("           ||     ||");
        println!("\n");

        // Listen to events on the P2P network, and user input (for now).
        loop {
            select! {
                event = self.r.recv() => {
                    match event.unwrap() {
                        Event::Liebe => {
                            info!("-------------------LIEBE");
                        },
                        Event::BlockMined(mut blocks) => {
                            let rcv_chain =
                            Vec::<Block>::read_from_buffer(&mut blocks[..]).unwrap();

                            info!("validating chain with the new block... {:#?}", rcv_chain);

                            let now = Instant::now();
                            let is_valid = Block::validate_all(&rcv_chain).is_ok();

                            if is_valid {
                                info!("chain is valid and took {}ms to validate", now.elapsed().as_millis());
                                debug!("chain is valid");
                                let mut file = blockchain::open().await;
                                match file.write_all(&blocks[..]).await {
                                    Ok(_) => {
                                        info!(
                                            "The new blockchain was written in {}μs with success",
                                            now.elapsed().as_micros()
                                        );
                                    },
                                    Err(_) => warn!("error trying to write new blockchain to the file")
                                }
                            } else {
                                warn!("chain is invalid");
                            }
                        }
                    };
                }
                line = stdin.select_next_some() => {
                    let msg = line.unwrap();

                    match msg.as_str() {
                        "ls blockchain" => {
                            let chain = blockchain::read_all().await.unwrap();
                            println!("{:#?}", chain);
                        },
                        "ls peers" => {
                            let peers: Vec<(&PeerId, Vec<&TopicHash>)> = self.swarm
                                .behaviour()
                                .gossipsub
                                .all_peers().collect();
                            println!("{:#?}", peers);
                        },
                        _ => {
                            if let Err(e) = self.swarm
                                .behaviour_mut().gossipsub
                                .publish(TOPIC.clone(), msg.as_bytes()) {
                                    println!("Publish error: {:?}", e);
                                }
                        }
                    }
                },
                swarm_event = self.swarm.select_next_some() => match swarm_event {
                    SwarmEvent::NewListenAddr {
                        address,
                        listener_id
                    } => {
                            info!("{:?} listening on {:?}", listener_id, address);
                        },
                    SwarmEvent::IncomingConnection { .. } => {},
                    SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                        if endpoint.is_dialer() {
                            info!("Connection established - peerid: {peer_id}");
                            info!("endpoint is dialer: {:#?}", endpoint);
                        }
                    }
                    SwarmEvent::Behaviour(AppBehaviourEvent::Gossipsub(GossipsubEvent::Message {
                        message,
                        propagation_source: peer,
                        ..
                    })) => {
                            // get the last 7 characters of the peerID
                            let peer = peer.to_string();
                            let truncated_peer_id = peer[peer.len() - 7..].to_string();
                            println!(
                                "\n{truncated_peer_id}: {}",
                                String::from_utf8_lossy(&message.data)
                            );
                        },
                    // SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                    //     for (peer_id, _multiaddr) in list {
                    //         info!("mDNS discovered a new peer: {}", peer_id);
                    //         self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    //     }
                    // },
                    // SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                    //     for (peer_id, _multiaddr) in list {
                    //         info!("mDNS discover peer has expired: {}", peer_id);
                    //         self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    //     }
                    // },
                    SwarmEvent::Dialing(peer_id) => println!("Dialing {peer_id}"),
                    _ => {}
                },
            };
        }
    }
}
