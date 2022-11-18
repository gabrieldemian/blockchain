use blockchain::models::{
    p2p::{Event, P2P},
    TOPIC,
};
use libp2p::PeerId;
use tokio::spawn;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut p2p = P2P::new().await;

    // let daemon_handle = spawn(async move {
    //     p2p.daemon().await;
    // });

    let handle = spawn(async move {
        // add_block("will add from the cli".to_string(), CHANNEL.0.clone()).await;
        p2p.swarm.dial(PeerId::random()).expect("to dial");

        // loop {
        //     let is_connected = p2p.swarm.is_connected(&LOCAL_PEER_ID);
        //     println!("am i connected {is_connected}");
        //     if is_connected {
        //         break;
        //     };
        // }

        if let Err(e) = p2p
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(TOPIC.clone(), *b"sending this message from the client bro.")
        {
            println!("Publish error: {:?}", e);
        }

        if let Err(e) = p2p.s.send(Event::Liebe) {
            println!(
                "Failed to send event to the network that the block was mined. Reason: {}",
                e.to_string()
            );
        };
    });

    handle.await.unwrap();
    // daemon_handle.await.unwrap();
}
