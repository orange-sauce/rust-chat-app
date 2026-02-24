use egui::{CentralPanel, SidePanel, TopBottomPanel};
use serde_json;
struct App {
    counter: i32,
    message: Vec<String>,
    mahinput: String,
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ctx.style_mut(|style| {
                style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
                style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 60, 60);
                style.visuals.code_bg_color = egui::Color32::from_rgb(0, 0, 0);
                // style.visuals.panel_fill = egui::Color32::from_rgb(0, 0, 0);
            });
            ui.add_space(10.0);
            let label = egui::Label::new(self.counter.to_string());
            ui.add(label);
            let submit_button = egui::Button::new("Send");
            if ui.add(submit_button).clicked() {
            }
            if !self.message.is_empty() {
                for i in self.message.clone() {
                    ui.add(egui::Label::new(format!("{}", i)));
                }
            }
        });
        SidePanel::right(egui::Id::new("right panel")).show(ctx, |ui| {
            ui.label("Hello world");
        });
        TopBottomPanel::bottom(egui::Id::new("my new bottom panel")).show(ctx, |ui| {
            let _ = ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    let response = ui.add(egui::TextEdit::singleline(&mut self.mahinput).hint_text("Type your text here: "));
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.message.push(self.mahinput.clone());
                        self.mahinput.clear();
                    }
                    let submit_button = egui::Button::new("Click me!");
                    if ui.add(submit_button).clicked() {
                    }
                });
                ui.add_space(20.0);
            });
        });
    }
}
impl Default for App {
    fn default() -> Self {
        App { 
            counter: 0,
            message: Vec::new(),
            mahinput: String::new() 
        }
    }
}
pub fn run_ui() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rust Chat App",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<App>::default())
        }),
    )
}
