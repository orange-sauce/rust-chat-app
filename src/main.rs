use tokio::sync::mpsc;
use crate::{client::NetworkCommands, network::User};
mod client;
mod ui;
mod network;
#[tokio::main]
async fn main() {
    let (sender_to_nw, reciever_to_nw) = mpsc::channel::<NetworkCommands>(100);
    let ui_handle = tokio::spawn(async move {
        let ui = ui::run_ui();
    });
    let handle = tokio::task::spawn(async move {
        let mut new_user = User::new(sender_to_nw, reciever_to_nw).await.unwrap();
        for i in new_user.listing_all_people() {
            log::info!("This new person is: {:?}", i);
        }
        let _ = new_user.run().await.unwrap();
        sender_to_nw.clone().send(NetworkCommands::MessageSent { message: "Hello world".to_string(), topic: new_user.listing_all_people()[0].clone() });
        
    });
    let asld = handle.await;
    let asd = ui_handle.await;
}

