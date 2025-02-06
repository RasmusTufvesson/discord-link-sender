use tokio::sync::mpsc::Sender;
use eframe::egui::{self, Grid, ViewportBuilder};
use clipboard_win::{formats, get_clipboard, SysResult};
use crate::bot::Packet;

pub struct App {
    window_name: String,
    to_send: Sender<Packet>,
    paste_lines: Vec<Vec<String>>,
    total_paste_lines: Vec<String>,
    channels: Vec<String>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, bot_name: String, to_send: Sender<Packet>, channels: Vec<String>) -> Self {
        return Self {
            window_name: bot_name + " control",
            to_send,
            paste_lines: channels.iter().map(|_| vec![]).collect(),
            total_paste_lines: vec![],
            channels,
        };
    }

    pub fn try_send(&mut self) {
        for channel in 0..self.channels.len() {
            for chunk in self.paste_lines[channel].chunks(5).filter(|x| x.len() == 5) {
                let chunk_string = chunk.join("\n");
                self.to_send.blocking_send(Packet::Send(chunk_string, channel)).unwrap();
            }
            let len = self.paste_lines[channel].len();
            self.paste_lines[channel] = self.paste_lines[channel].split_off(len - len % 5);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout { main_dir: egui::Direction::TopDown, main_wrap: false, main_align: eframe::emath::Align::Min, main_justify: false, cross_align: eframe::emath::Align::Center, cross_justify: true }, |ui: &mut egui::Ui| {
                ui.heading(&self.window_name);
                let mut try_to_send = false;
                let width = (ui.available_width() - ui.spacing().item_spacing.x * (self.channels.len() - 1) as f32) / self.channels.len() as f32;
                Grid::new("channels")
                    .num_columns(self.channels.len())
                    .min_col_width(width)
                    .max_col_width(width)
                    .striped(false)
                    .show(ui, |ui| {
                        for (channel, name) in self.channels.iter().enumerate() {
                            ui.vertical_centered_justified(|ui| {
                                ui.label(name);
                                if ui.button("Paste").clicked() {
                                    let result: SysResult<String> = get_clipboard(formats::Unicode);
                                    match result {
                                        Ok(string) => {
                                            for line in string.split("\n") {
                                                let string = line.to_string();
                                                if line != "" && !self.total_paste_lines.contains(&string) {
                                                    self.paste_lines[channel].push(string.clone());
                                                    self.total_paste_lines.push(string);
                                                } 
                                            }
                                            try_to_send = true;
                                        },
                                        _ => {}
                                    }
                                }
                            });
                        }
                    });
                if try_to_send {
                    self.try_send();
                }
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.paste_lines.len() != 0 {
            let mut chunks = vec![];
            for lines in &self.paste_lines {
                let chunk_string = lines.join("\n");
                chunks.push(chunk_string)
            }
            self.to_send.blocking_send(Packet::SendAndQuit(chunks)).unwrap();
        }
    }
}

pub fn main(bot_name: String, to_send: Sender<Packet>, channels: Vec<String>) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([300.0, 74.0]) // 30.0 + 42.0 * channels.len() as f32
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        &(bot_name.clone() + " control"),
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, bot_name, to_send, channels)))),
    )
}