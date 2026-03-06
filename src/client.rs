use libp2p::{PeerId, gossipsub::TopicHash};

#[derive(Debug, Clone)]
pub enum NetworkCommands {
    Subscribe { topic: libp2p::gossipsub::IdentTopic, username: Vec<u8> },
    MessageSent { message: String, topic: TopicHash },
    MessageRecieved { message: String, topic: TopicHash },
    TopicRecieved {}
}
#[derive(Debug, Clone)]
pub enum MessageStages {
    MessageSent,
    MessageRecieved
}
#[derive(Debug, Clone)]
pub struct Friend {
    peer_id: libp2p::PeerId,
    username: Option<String>,
}
impl Friend {
   pub fn new(peer_id: PeerId) -> Self {
       Friend { peer_id: peer_id, username: None }
   }
}
