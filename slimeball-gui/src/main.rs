use std::{fs::File, io::BufReader, path::PathBuf};

use eframe::egui::{self, Frame};
use mimalloc::MiMalloc;
use rfd::FileDialog;
use tracing::Level;
use tracing_subscriber::{EnvFilter, prelude::*};

#[macro_use]
extern crate tracing;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod anvil;
mod proto;
mod viewer;

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Slimeball",
        native_options,
        Box::new(|cc| Ok(Box::new(SlimeballGui::new(cc)))),
    )
    .unwrap();
}

#[derive(Clone)]
enum World {
    Slime(PathBuf),
    Anvil(PathBuf),
}

#[derive(Default)]
struct SlimeballGui {
    world: Option<World>,
}

impl eframe::App for SlimeballGui {
    fn update(&mut self, ui: &egui::Context, frame: &mut eframe::Frame) {
        match self.world {
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

                if ui.button("Open Anvil world").clicked() {
                    self.world = FileDialog::new().pick_folder().map(World::Anvil);
                }

                if ui.button("Open Slime world").clicked() {
                    let path = FileDialog::new()
                        .add_filter("Slime worlds", &["slime"])
                        .pick_file();
                    self.world = path.clone().map(World::Slime);

                    if let Some(path) = path {
                        let file = File::open(path).unwrap();
                        let mut buf = BufReader::new(file);
                        match slimeball_lib::SlimeWorld::deserialize(&mut buf) {
                            // TODO: async?
                            Ok(v) => proto::load_chunk_tops(v),
                            Err(why) => error!("{:?}", why),
                        }
                    }
                }
            });
        });
    }

    fn show_main(&mut self, ui: &egui::Context) {
        // asserted Some in match arm for render
        let world = self.world.as_ref().unwrap();
        let world_path = match world {
            World::Anvil(p) => p,
            World::Slime(p) => p,
        };

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

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(world_path.to_string_lossy());
        });
    }
}
