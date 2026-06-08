pub struct HierarchyPanel;

impl HierarchyPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, _ctx: &egui::Context) {}
}

impl Default for HierarchyPanel {
    fn default() -> Self {
        Self::new()
    }
}
