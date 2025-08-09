use std::path::PathBuf;

use eframe::egui::{self, Color32, Frame};
use mimalloc::MiMalloc;
use rfd::FileDialog;
use tracing::Level;
use tracing_subscriber::{EnvFilter, prelude::*};

#[macro_use]
extern crate tracing;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod anvil;

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        );

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Slimeball",
        native_options,
        Box::new(|cc| Ok(Box::new(SlimeballGui::new(cc)))),
    )
    .unwrap();
}

#[derive(Default)]
struct SlimeballGui {
    file: Option<PathBuf>,
}

impl eframe::App for SlimeballGui {
    fn update(&mut self, ui: &egui::Context, frame: &mut eframe::Frame) {
        match self.file {
            None => self.show_welcome(ui),
            Some(_) => self.show_main(ui),
        }
    }
}

impl SlimeballGui {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }

    fn show_welcome(&mut self, ui: &egui::Context) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("No world selected");
                if ui.button("Open a world...").clicked() {
                    self.file = FileDialog::new().pick_folder();
                }
            });
        });
    }

    fn show_main(&mut self, ui: &egui::Context) {
        // asserted Some in match arm for render
        let path = self.file.as_ref().unwrap();

        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(ui.style().visuals.panel_fill)
                    .inner_margin(16),
            )
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Settings");
                });
            });

        egui::CentralPanel::default().show(ui, |ui| ui.heading(path.to_string_lossy()));
    }
}
