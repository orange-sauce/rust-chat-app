use libp2p::gossipsub::Message;
use tokio::sync::mpsc;
use crate::{client::NetworkCommands, network::User};
mod client;
mod ui;
mod network;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let (sender_to_nw, reciever_to_nw) = mpsc::channel(100);
    let (sender_to_ui, reciever_to_ui) = mpsc::channel(100);
    let mut new_user = User::new(sender_to_ui.clone(), reciever_to_nw).await.unwrap();

    let value = sender_to_nw.clone();
    let handler_ui = tokio::task::spawn_blocking(move || {
        // remember to change this after UI designing
        let _  = ui::run_ui(reciever_to_ui, value).unwrap();
    });
    let handle = tokio::task::spawn(async move {
        let _ = new_user.run().await.unwrap();
    });

    sender_to_nw.send(
        NetworkCommands::Subscribe { topic: "This is how this looks like".to_string() }
    ).await.unwrap();
    sender_to_nw.send(
        NetworkCommands::MessageSent { message: "Hello world".to_string(), topic: "Hello world".to_string() }
    ).await.unwrap();

    handler_ui.await;
    let _ = handle.await;
}
