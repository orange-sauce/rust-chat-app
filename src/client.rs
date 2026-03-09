use libp2p::{PeerId};

#[derive(Debug, Clone)]
pub enum NetworkCommands {
    Subscribe { topic: String },
    MessageSent { message: String, topic: String },
    MessageRecieved { message: String, topic: String },
    TopicRecieved { topic: String }
}
#[derive(Debug, Clone)]
pub enum MessageStages {
    MessageSent,
    MessageRecieved
}
#[derive(Debug, Clone)]
pub struct Friend {
    permission_to_talk: bool,
    peer_id: libp2p::PeerId,
    username: Option<String>,
}
impl Friend {
   pub fn new(peer_id: PeerId) -> Self {
       Friend { peer_id: peer_id, username: None, permission_to_talk: true}
   }
}
