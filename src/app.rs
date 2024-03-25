use tokio::sync::mpsc::Sender;
use eframe::egui;
use clipboard_win::{formats, get_clipboard, SysResult};
use crate::bot::Packet;

pub struct App {
    window_name: String,
    to_send: Sender<Packet>,
    paste_lines: Vec<String>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, bot_name: String, to_send: Sender<Packet>) -> Self {
        return Self {
            window_name: bot_name + " control",
            to_send,
            paste_lines: vec![],
        };
    }

    pub fn try_send(&mut self) {
        for chunk in self.paste_lines.chunks(5).filter(|x| x.len() == 5) {
            let chuck_string = chunk.join("\n");
            self.to_send.blocking_send(Packet::Send(chuck_string)).unwrap();
        }
        self.paste_lines = self.paste_lines.split_off(self.paste_lines.len() - self.paste_lines.len() % 5);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout { main_dir: egui::Direction::TopDown, main_wrap: false, main_align: eframe::emath::Align::Min, main_justify: false, cross_align: eframe::emath::Align::Center, cross_justify: true }, |ui: &mut egui::Ui| {
                ui.heading(&self.window_name);
                if ui.button("Paste").clicked() {
                    let result: SysResult<String> = get_clipboard(formats::Unicode);
                    match result {
                        Ok(string) => {
                            for line in string.split("\n") {
                                let string = line.to_string();
                                if line != "" && !self.paste_lines.contains(&string) {
                                    self.paste_lines.push(string);
                                } 
                            }
                            self.try_send();
                        },
                        _ => {}
                    }
                }
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.paste_lines.len() != 0 {
            let chuck_string = self.paste_lines.join("\n");
            self.to_send.blocking_send(Packet::SendAndQuit(chuck_string)).unwrap();
        }
    }
}

pub fn main(bot_name: String, to_send: Sender<Packet>) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        always_on_top: true,
        maximized: false,
        decorated: true,
        fullscreen: false,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_pos: None,
        initial_window_size: Some(egui::vec2(300.0,58.0)),
        min_window_size: None,
        max_window_size: None,
        resizable: false,
        transparent: false,
        mouse_passthrough: false,
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Glow,
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        run_and_return: true,
        event_loop_builder: None,
        shader_version: None,
        centered: true
    };
    eframe::run_native(
        &(bot_name.clone() + " control"),
        native_options,
        Box::new(|cc| Box::new(App::new(cc, bot_name, to_send))),
    )
}