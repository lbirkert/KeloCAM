use crate::widget::viewer::Viewer;

#[derive(Default)]
pub struct PrepareView {}

impl PrepareView {
    pub fn show(&mut self, ui: &mut egui::Ui, viewer: &mut Viewer) {
        viewer.ui(ui);
    }
}
