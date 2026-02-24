use std::hash::{Hash, Hasher};
use futures::StreamExt;
use libp2p::{PeerId, Swarm, SwarmBuilder, gossipsub::{self, Topic}, identify::Info, identity::Keypair, mdns, noise, swarm::{NetworkBehaviour, SwarmEvent}, tcp, yamux};
#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: libp2p::mdns::tokio::Behaviour,
    gossipsub: libp2p::gossipsub::Behaviour,
}
struct User {
    username: String,
    swarm: Swarm<Behaviour>,
    topics: Vec<libp2p::gossipsub::IdentTopic>,
    messages: Vec<libp2p::gossipsub::Message>,
    friends: Vec<Friend>
}
struct Friend {
    peerId: libp2p::PeerId,
    peer_username: String
}
impl Friend {}
impl User {
    async fn start_networking() -> Result<(), Box<dyn std::error::Error>>{
        let _ = tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).try_init();
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_quic()
            .with_behaviour(|key: &Keypair| {
                let message_id_fn = |message: &libp2p::gossipsub::Message| {
                    let mut s = std::collections::hash_map::DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(std::time::Duration::from_millis(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .build()
                    .expect("Something is wrong with building this gossipsub config.");
                let peer_id: PeerId = PeerId::from_public_key(&key.public());
                let gossipsub = gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(key.clone()), gossipsub_config)?;
                let mdns = libp2p::mdns::Behaviour::new(mdns::Config::default(), peer_id)?;
                Ok( Behaviour {mdns, gossipsub})
            })?
            .build();
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        loop {
            tokio::select! {
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discovered a new peer: {peer_id}");
                            // Adding an peer
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discover peer has expired: {peer_id}");
                            // The person has left
                            swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => println!(
                            "Got message: '{}' with id: {id} from peer: {peer_id}",
                            // Got a new message
                            String::from_utf8_lossy(&message.data),
                        ),
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Local node is listening on {address}");
                    }
                    _ => {}
                }
            }
        }
    }
    async fn making_a_new_topic(swarm: &mut Swarm<Behaviour>, topic_name: String) {
        let new_topic = gossipsub::IdentTopic::new(topic_name);
        swarm.behaviour_mut().gossipsub.subscribe(&new_topic);
    }
    async fn sending_message() {}
    async fn sending_file() {}
}
// fn sending_message(message: String, topic: libp2p::gossipsub::IdentTopic, swarm: &mut Swarm<Behaviour>) {
//     if let Ok(Some(message)) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.as_bytes()) {
//         info!["Sending messages failed!"]
//     };
// }
pub fn init() {}
