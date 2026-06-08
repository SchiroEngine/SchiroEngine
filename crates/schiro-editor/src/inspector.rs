pub struct InspectorPanel;

impl InspectorPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, _ctx: &egui::Context) {}
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}
