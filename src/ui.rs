use egui::{CentralPanel, SidePanel, TextEdit, TopBottomPanel};

use crate::client::{Friend, NetworkCommands};
pub struct App {
    reciever: tokio::sync::mpsc::Receiver<NetworkCommands>,
    sender: tokio::sync::mpsc::Sender<NetworkCommands>,
    messages_recieved: Vec<String>,
    friends: Vec<Friend>,
    message_inputted: Vec<String>,
    current_topic: Option<usize>
}

impl App {
   fn new(
        reciever: tokio::sync::mpsc::Receiver<NetworkCommands>,
        sender: tokio::sync::mpsc::Sender<NetworkCommands>) -> Self {
            App {
                reciever: reciever,
                sender: sender,
                messages_recieved: Vec::new(),
                message_inputted: Vec::new(),
                friends: Vec::new(),
                current_topic: Some(0)
            }
   }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
       while let Ok(cmd) = self.reciever.try_recv() {
          match cmd {
              NetworkCommands::MessageSent { message: message, topic: topic } => {},
              NetworkCommands::Subscribe { topic: topic } => {},
              NetworkCommands::TopicRecieved { topic: topic } => {},
              NetworkCommands::MessageRecieved { message: message, topic: topic } => {},
              _ => {}
           };
        };
        CentralPanel::default().show(ctx, |ui| {
            ctx.style_mut(|style| {
                style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
                style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 60, 60);
                style.visuals.code_bg_color = egui::Color32::from_rgb(0, 0, 0);
                // style.visuals.panel_fill = egui::Color32::from_rgb(0, 0, 0);
            
            })
        });
        SidePanel::right(egui::Id::new("right panel")).show(ctx, |ui| {});
        TopBottomPanel::bottom(egui::Id::new("my new bottom panel")).show(ctx, |ui| {
            let mut message: String = String::new();
            let message_bar = ui.add(egui::TextEdit::singleline(&mut message));
            if message_bar.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                log::info!("Message was inputted: {}", message);
                message.clear();
            }
        });
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Stoping the networking stack
        todo!()
    }
}

pub fn run_ui(
        reciver: tokio::sync::mpsc::Receiver<NetworkCommands>,
        sender: tokio::sync::mpsc::Sender<NetworkCommands>
    ) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rust Chat App",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(App::new(reciver, sender)))
        }),
    )
}
