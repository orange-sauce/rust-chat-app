use std::{
    collections::HashMap, error::Error, hash::{DefaultHasher, Hasher}
};
use std::hash::{Hash};
use crate::client::{Friend, MessageStages, NetworkCommands};
use futures::channel::mpsc::SendError;
use libp2p::{
    futures::StreamExt,
    PeerId, SwarmBuilder, gossipsub,
    gossipsub::{IdentTopic, MessageId, TopicHash, Message},
    identity::Keypair,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    mdns, noise, tcp, yamux, 
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
}
impl User {
    pub async fn new(
        sender: tokio::sync::mpsc::Sender<crate::client::NetworkCommands>,
        reciever: tokio::sync::mpsc::Receiver<crate::client::NetworkCommands>
    ) -> Result<Self, Box<dyn Error>> {

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
                let mdns = libp2p::mdns::Behaviour::new(mdns::Config::default(), peer_id)?;
                Ok( Behaviour {mdns, gossipsub})
            })?
            .with_swarm_config(|cfg: libp2p::swarm::Config| {
               cfg.with_idle_connection_timeout(std::time::Duration::from_mins(5))
            })
            .build();;
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        Ok(User {
            username: String::new(),
            swarm: swarm,
            messages: Vec::new(),
            sender: sender,
            reciever: reciever,
            friends: Vec::new(),
        })

    }
    async fn finding_nemo(&mut self, peer_id: PeerId) -> HashMap<PeerId, Vec<&TopicHash>>{
        let mut new_vector: Vec<&TopicHash> = Vec::new();
        let mut new_hash_map = HashMap::new();
        for (peer_id, topic_hashes) in self.swarm.behaviour_mut().gossipsub.all_peers() {
            println!("Peer {:?} is subscribed to topics:", peer_id);
            for topic_hash in topic_hashes {
               new_vector.push(topic_hash);
            }
            new_hash_map.insert(*peer_id, new_vector.clone());
        }
        new_hash_map
    }
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let result_processing = async || {
            loop {
                if let Ok(Some(NetworkCommands::MessageRecieved { message, topic })) = timeout(Duration::from_secs(5), self.reciever.recv()).await {
                    self.swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.as_bytes());
                    return (message, topic)
                }
            }
        };
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            log::info!("mDNS discoverd a new peer: {peer_id}");
                            // Adding an peer, should i engage in a conversation with them?
                            let my_friend: Friend = Friend::new(peer_id);
                            self.friends.push(my_friend);
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
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
    async fn sending_messages(
        &mut self,
        message: Message,
        topic: TopicHash,
        peerid: PeerId,
        message_id: MessageId,
    ) -> Result<MessageStages, tokio::sync::mpsc::error::SendError<NetworkCommands>> {
        let sent_topic = message.topic.into_string();
        let logging_message: String = format!( "Got message: '{}' with id: {message_id} from peer: {peerid}",
            String::from_utf8_lossy(&message.data)
        );
        self.sender
            .send(crate::client::NetworkCommands::MessageRecieved {
                message: String::from_utf8_lossy(&message.data).to_string(),
                topic: topic,
            })
            .await?;
        log::info!("{}", logging_message);
        Ok(MessageStages::MessageSent)
    }

    pub fn listing_all_people(&mut self) -> Vec<TopicHash>{
        let mut new_vec: Vec<TopicHash> = Vec::new();
        for (peer_id, topic_hashes) in self.swarm.behaviour_mut().gossipsub.all_peers() {
            log::info!("The peer id is {peer_id}, their topics are {:?}", topic_hashes);
            for topic_hash in topic_hashes {
                new_vec.push(topic_hash.clone());
            }
        };
        new_vec
    }
    async fn adding_new_people(&mut self, peer_id: &PeerId) {
        let attempt = self.swarm.behaviour_mut().gossipsub.add_explicit_peer(peer_id);
    }
}


