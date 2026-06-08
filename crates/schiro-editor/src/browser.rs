pub struct AssetBrowser;

impl AssetBrowser {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, _ctx: &egui::Context) {}
}

impl Default for AssetBrowser {
    fn default() -> Self {
        Self::new()
    }
}
