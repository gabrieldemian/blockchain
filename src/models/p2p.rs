use crate::models::{block::Block, blockchain, TOPIC};
use async_std::io;
use futures::prelude::*;
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    gossipsub::{Gossipsub, GossipsubConfig, GossipsubEvent, MessageAuthenticity, TopicHash},
    identity::Keypair,
    kad::{
        record::Key, store::MemoryStore, AddProviderOk, Kademlia, KademliaEvent, PeerRecord,
        PutRecordOk, QueryResult, Quorum, Record,
    },
    mdns::{MdnsEvent, TokioMdns},
    mplex,
    noise::NoiseAuthenticated,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp::{self, GenTcpConfig},
    Multiaddr, NetworkBehaviour, PeerId, Swarm, Transport,
};
use log::{debug, info, warn};
use speedy::Readable;
use tokio::{
    io::AsyncWriteExt,
    select,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::Instant,
};

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
    pub mdns: TokioMdns,
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

        // let kademilia_config =
        //     KademliaConfig::default().set_protocol_names(vec![Cow::from(b"demian".to_owned())]);
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
                mdns,
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

        swarm.listen_on(addr).expect("could not listen on swarm");
        info!("Your PeerID is {local_key}");

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
                    let args = line.unwrap();
                    let mut args = args.split(' ');

                    match args.next() {
                        Some("ls_blocks") => {
                            let chain = blockchain::read_all().await.unwrap();
                            println!("{:#?}", chain);
                        },
                        Some("ls_peers") => {
                            let peers: Vec<(&PeerId, Vec<&TopicHash>)> = self.swarm
                                .behaviour()
                                .gossipsub
                                .all_peers().collect();

                            println!("{:#?}", peers);

                            let key = {
                                match args.next() {
                                    Some(key) => Key::new(&key),
                                    None => {
                                        eprintln!("Expected key");
                                        return;
                                    }
                                }
                            };
                            self.swarm.behaviour_mut().kademlia.get_providers(key);
                        },
                        Some("GET") => {
                            let key = {
                                match args.next() {
                                    Some(key) => Key::new(&key),
                                    None => {
                                        eprintln!("Expected key");
                                        return;
                                    }
                                }
                            };
                            self.swarm.behaviour_mut().kademlia.get_record(key, Quorum::One);
                        },
                        Some("PUT") => {
                            let key = {
                                match args.next() {
                                    Some(key) => Key::new(&key),
                                    None => {
                                        eprintln!("Expected key");
                                        return;
                                    }
                                }
                            };
                            let value = {
                                match args.next() {
                                    Some(value) => value.as_bytes().to_vec(),
                                    None => {
                                        eprintln!("Expected value");
                                        return;
                                    }
                                }
                            };
                            let record = Record {
                                key,
                                value,
                                publisher: None,
                                expires: None,
                            };
                            self.swarm.behaviour_mut().kademlia
                                .put_record(record, Quorum::One)
                                .expect("Failed to store record locally.");
                        }
                        Some("PUT_PROVIDER") => {
                            let key = {
                                match args.next() {
                                    Some(key) => Key::new(&key),
                                    None => {
                                        eprintln!("Expected key");
                                        return;
                                    }
                                }
                            };

                            self.swarm.behaviour_mut().kademlia
                                .start_providing(key)
                                .expect("Failed to start providing key");
                        }
                        _ => {
                            eprintln!("expected GET, GET_PROVIDERS, PUT or PUT_PROVIDER");
                            // if let Err(e) = self.swarm
                            //     .behaviour_mut().gossipsub
                            //     .publish(TOPIC.clone(), args.next().unwrap().as_bytes()) {
                            //         println!("Publish error: {:?}", e);
                            //     }
                        }
                    }
                },
                swarm_event = self.swarm.select_next_some() => match swarm_event {
                    SwarmEvent::NewListenAddr {
                        address,
                        listener_id
                    } => {
                            info!("{:?} listening on {:?}", listener_id, address);
                            // self.swarm.behaviour_mut().kademlia.add_address(&self.local_key, address);
                        },
                    SwarmEvent::IncomingConnection { .. } => {},
                    SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                        if endpoint.is_dialer() {
                            info!("Connection established - peerId: {peer_id}");
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
                    // will notify RoutingUpdated if kademilia_add_address is successfull.
                    SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                        for (peer_id, multiaddr) in list {
                            info!("mDNS discovered a new peer: {peer_id} multiaddr: {multiaddr}");
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            self.swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                        }
                    },
                    SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            info!("mDNS discover peer has expired: {}", peer_id);
                            self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(AppBehaviourEvent::Kademlia(KademliaEvent::RoutingUpdated{ peer, addresses, .. })) => {
                        info!("routing updated with {peer}");
                        info!("addresses known of this peer {:#?}", addresses);
                    },
                    SwarmEvent::Dialing(peer_id) => info!("Dialing {peer_id}"),
                    SwarmEvent::Behaviour(AppBehaviourEvent::Kademlia(KademliaEvent::OutboundQueryCompleted { result, ..})) => {
                        match result {
                            QueryResult::GetProviders(Ok(ok)) => {
                                for peer in ok.providers {
                                    println!(
                                        "Peer {:?} provides key {:?}",
                                        peer,
                                        std::str::from_utf8(ok.key.as_ref()).unwrap()
                                    );
                                }
                            },
                            QueryResult::GetRecord(Ok(ok)) => {
                                for PeerRecord {
                                    record: Record { key, value, .. },
                                    ..
                                } in ok.records
                                {
                                    println!(
                                        "Got record {:?} {:?}",
                                        std::str::from_utf8(key.as_ref()).unwrap(),
                                        std::str::from_utf8(&value).unwrap(),
                                    );
                                }
                            },
                            QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                                println!(
                                    "Successfully put record {:?}",
                                    std::str::from_utf8(key.as_ref()).unwrap()
                                );
                            },
                            QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                                println!(
                                    "Successfully put provider record {:?}",
                                    std::str::from_utf8(key.as_ref()).unwrap()
                                );
                            }
                            _ => {}
                        };
                        }
                    _ => {}
                },
            };
        }
    }
}
