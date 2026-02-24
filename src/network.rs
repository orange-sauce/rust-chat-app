use std::hash::{Hash, Hasher};
use futures::StreamExt;
use libp2p::{PeerId, Swarm, SwarmBuilder, gossipsub::{self}, identify::Info, identity::Keypair, mdns, noise, swarm::{NetworkBehaviour, SwarmEvent}, tcp, yamux};
#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: libp2p::mdns::tokio::Behaviour,
    gossipsub: libp2p::gossipsub::Behaviour,
}
struct User {
    username: String,
    swarm: Swarm<Behaviour>,
    topics_made: Vec<libp2p::gossipsub::IdentTopic>,
    topics_joined: Vec<libp2p::gossipsub::IdentTopic>,
    messages: Vec<libp2p::gossipsub::Message>,
    friends: Vec<Friend>
}
struct Friend {
    peerId: libp2p::PeerId,
    // Please remind me to change this after i know how to send that first packet
    peer_username: Option<String>,
    multiaddr: Option<libp2p::Multiaddr>,
}
impl Friend {
    fn new(peer_id: PeerId, peer_username: String, multi_address: libp2p::Multiaddr) -> Self {
       Friend {
           peerId: peer_id,
           peer_username: Some(peer_username),
           multiaddr: Some(multi_address)
       } 
    }
}
impl User {
    async fn new(username: String) -> Result<Self, Box<dyn std::error::Error>> {
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
            .with_swarm_config(|cfg: libp2p::swarm::Config| {
               cfg.with_idle_connection_timeout(std::time::Duration::from_mins(5))
            })
            .build();
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        Ok( User {
           username: username,
           swarm: swarm,
           messages: Vec::new(),
           friends: Vec::new(),
           topics_made: Vec::new(),
           topics_joined: Vec::new()
        })
    }
    async fn run(&mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discovered a new peer: {peer_id}");
                            // Adding an peer
                            let my_friend: Friend = Friend::new(peer_id, String::new(), _multiaddr);
                            self.friends.push(my_friend);
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discover peer has expired: {peer_id}");
                            // Please also remind me to remove this when i figure out what to do
                            self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
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
    async fn making_a_new_topic(swarm: &mut Swarm<Behaviour>, topic_name: String)
        -> Result<libp2p::gossipsub::IdentTopic, libp2p::gossipsub::SubscriptionError> {
        let new_topic = gossipsub::IdentTopic::new(topic_name);
        swarm.behaviour_mut().gossipsub.subscribe(&new_topic);
        Ok(new_topic)
    }
    async fn sending_message(&mut self, message: String, topic_id: usize) {
        let current_topic: libp2p::gossipsub::IdentTopic = self.topics_made[topic_id].clone();
        let attempt = self.swarm.behaviour_mut().gossipsub.publish(current_topic, message.as_bytes());
        match attempt {
           Ok(msg) => println!("Message sent"),
           Err(e) => {
               match e {
                   libp2p::gossipsub::PublishError::NoPeersSubscribedToTopic =>
                       { println!("No peers are currently subsribed to this, your just sending it to nobody rn.") },
                   libp2p::gossipsub::PublishError::MessageTooLarge =>
                       { println!("I dont think you can sent your mom through this, she might be too big") },
                   libp2p::gossipsub::PublishError::AllQueuesFull(value) => {
                       println!("Pls get some more friends")
                   },
                   _ => {}
               }
           }
        }
    }
    async fn sending_file() {}
}
