use eframe::egui::Vec2;

pub struct Viewer {
    camera_pos: Vec2,
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
}
