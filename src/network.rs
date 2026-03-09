use std::{
    collections::HashMap, hash::{DefaultHasher, Hasher}
};
use std::hash::{Hash};
use crate::client::{Friend, MessageStages, NetworkCommands};
use futures::channel::mpsc::SendError;
use libp2p::{
    PeerId, SwarmBuilder, futures::StreamExt,
    gossipsub::{self, IdentTopic, Message, MessageId, Topic, TopicHash},
    identity::Keypair, mdns, noise,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux 
};
use tokio::time::{timeout, Duration};
#[derive(NetworkBehaviour)]
struct Behaviour {
    mdns: libp2p::mdns::tokio::Behaviour,
    gossipsub: libp2p::gossipsub::Behaviour,
}
pub struct User {
    username: String,
    swarm: Swarm<Behaviour>,
    reciever: tokio::sync::mpsc::Receiver<crate::client::NetworkCommands>,
    sender: tokio::sync::mpsc::Sender<crate::client::NetworkCommands>,
    messages: Vec<libp2p::gossipsub::Message>,
    friends: Vec<crate::client::Friend>,
    topics: Vec<IdentTopic>
}
impl User {
    pub async fn new(
        sender: tokio::sync::mpsc::Sender<crate::client::NetworkCommands>,
        reciever: tokio::sync::mpsc::Receiver<crate::client::NetworkCommands>
    ) -> Result<Self, Box<dyn std::error::Error>> {

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
                    .heartbeat_interval(std::time::Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .build()
                    .expect("Something is wrong with building this gossipsub config.");
                let peer_id: PeerId = PeerId::from_public_key(&key.public());
                let gossipsub = gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(key.clone()), gossipsub_config)?;
                // let mdns_config = libp2p::mdns::Config::default();
                let mdns = libp2p::mdns::Behaviour::new(mdns::Config::default(), peer_id)?;
                Ok( Behaviour {mdns, gossipsub})
            })?
            .with_swarm_config(|cfg: libp2p::swarm::Config| {
               cfg.with_idle_connection_timeout(std::time::Duration::from_mins(5))
            })
            .build();
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(User {
            username: String::new(),
            swarm: swarm,
            messages: Vec::new(),
            sender: sender,
            reciever: reciever,
            friends: Vec::new(),
            topics: Vec::new()
        })

    }
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            tokio::select! {
                Some(cmd) = self.reciever.recv() => match cmd {
                        NetworkCommands::MessageSent { message , topic  } => {
                            // self.sender
                            //     .send(crate::client::NetworkCommands::MessageRecieved {
                            //         message: String::from_utf8_lossy(&message.data).to_string(),
                            //         topic: topic,
                            //     })
                            //     .await?;
                            if let Some(topic) = self.topics.first() && !(self.friends.is_empty()) {
                                self.swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.as_bytes()).unwrap();
                            }
                            log::info!("Message was sent: {}", message);
                            ()
                        },
                        NetworkCommands::Subscribe { topic: topic } => {
                            let new_topic = IdentTopic::new(topic);
                            self.topics.push(new_topic.clone());
                            self.swarm.behaviour_mut().gossipsub.subscribe(&new_topic).unwrap();
                            ()
                        },
                        _ => {}
                    },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            log::info!("mDNS discoverd a new peer: {peer_id}");
                            let my_friend: Friend = Friend::new(peer_id);
                            self.friends.push(my_friend);
                            // Adding an peer, should i engage in a conversation with them?
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            log::info!("finding a friend");
                        }
                    },
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            log::info!("mDNS discover peer has expired: {peer_id}");
                            self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => {
                        log::info!("Got message from {peer_id} with message_id = {id}: message = {}", String::from_utf8_lossy(&message.data))
                    },
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Local node is listening on {address}");
                    },
                    _ => {}
                }
            }
        }
    }
}
