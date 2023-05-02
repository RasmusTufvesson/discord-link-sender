use tokio::sync::mpsc::Sender;
use eframe::egui;
use clipboard_win::{formats, get_clipboard, SysResult};

pub struct App {
    window_name: String,
    to_send: Sender<String>,
    paste_lines: Vec<String>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, bot_name: String, to_send: Sender<String>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        return Self {
            window_name: bot_name + " control",
            to_send,
            paste_lines: vec![],
        };
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.with_layout(egui::Layout { main_dir: egui::Direction::TopDown, main_wrap: false, main_align: eframe::emath::Align::Min, main_justify: false, cross_align: eframe::emath::Align::Center, cross_justify: true }, |ui: &mut egui::Ui| {
                ui.heading(&self.window_name);
                ui.label(self.paste_lines.len().to_string() + " messages");
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
                        },
                        _ => {}
                    }
                }
                if ui.button("Send").clicked() {
                    for chunk in self.paste_lines.chunks(5) {
                        let chuck_string = chunk.join("\n");
                        self.to_send.blocking_send(chuck_string).unwrap();
                    }
                    self.paste_lines.clear();
                }
            });
        });
    }
}

pub fn main(bot_name: String, to_send: Sender<String>) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        always_on_top: true,
        maximized: false,
        decorated: true,
        fullscreen: false,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_pos: None,
        initial_window_size: Some(egui::vec2(300.0,100.0)),
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