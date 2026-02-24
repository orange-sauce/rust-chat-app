mod ui;
mod network;
#[tokio::main]
async fn main() {
    tokio::task::spawn(async {
        let _ = network::init();
    });
    tokio::task::spawn(async {
        ui::run_ui();
    });
}
