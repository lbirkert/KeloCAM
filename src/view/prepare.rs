use crate::widget::viewer::Viewer;

#[derive(Default)]
pub struct PrepareView {}

impl PrepareView {
    pub fn show(&mut self, ctx: &egui::Context, viewer: &mut Viewer) {
        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            ui.heading("Editor");
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show(ctx, |ui| {
                viewer.ui(ui);
            });
    }
}
