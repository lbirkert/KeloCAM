use crate::widget::viewer::Viewer;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PrepareView {}

impl PrepareView {
    pub fn show(self: &mut Self, ui: &mut egui::Ui, viewer: &mut Viewer) {
        viewer.ui(ui);
    }
}

impl Default for PrepareView {
    fn default() -> Self {
        Self {}
    }
}
