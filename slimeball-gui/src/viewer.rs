use color_eyre::eyre::ensure;
use eframe::{
    egui::{self, Color32, Id, LayerId, Order, Pos2, Rect, Vec2},
    wgpu::Color,
};

pub struct Viewer {
    camera_pos: Pos2,
    camera_zoom: f32,
}

impl Default for Viewer {
    fn default() -> Self {
        Self {
            camera_pos: Default::default(),
            camera_zoom: 1.0,
        }
    }
}

impl Viewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, ui: &egui::Ui) {
        let painter = ui
            .ctx()
            .layer_painter(LayerId::new(Order::Foreground, Id::new("viewer_target")))
            .with_clip_rect(ui.clip_rect());

        let anchor = painter.clip_rect().min;
        self.draw_chunk(&painter, anchor, vec![Color32::ORANGE; 256]);
    }

    pub fn draw_chunk(&mut self, painter: &egui::Painter, anchor: Pos2, chunk: Vec<Color32>) {
        if chunk.len() != 256 {
            panic!(
                "Length of chunk colors provided was not 256 (got {})",
                chunk.len()
            );
        }

        for x in 0..=15 {
            for y in 0..=15 {
                let index = x * 15 + y;
                let min = anchor + Vec2::new(x as f32 * 10.0, y as f32 * 10.0);

                painter.rect_filled(
                    Rect::from_min_max(min, min + Vec2::new(10., 10.)),
                    0.0,
                    chunk[index],
                );
            }
        }
    }
}
