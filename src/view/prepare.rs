use crate::editor::Editor;

#[derive(Default)]
pub struct PrepareView {}

impl PrepareView {
    pub fn show(&mut self, ctx: &egui::Context, editor: &mut Editor) {
        egui::SidePanel::left("my_left_panel")
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
                egui::ScrollArea::new([false, true]).show(ui, |ui| {
                    ui.heading("Editor");

                    editor.sidebar(ui);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show(ctx, |ui| editor.ui(ui));
    }
}
