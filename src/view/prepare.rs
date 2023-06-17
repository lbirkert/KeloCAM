use crate::editor::Editor;
use crate::icon;

#[derive(Default)]
pub struct PrepareView {
    group_icon: Option<egui::TextureHandle>,
}

impl PrepareView {
    pub fn show(&mut self, ctx: &egui::Context, editor: &mut Editor) {
        let group_icon: &egui::TextureHandle = self
            .group_icon
            .get_or_insert_with(|| ctx.load_texture("group", icon!("group"), Default::default()));

        egui::SidePanel::left("my_left_panel")
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
                egui::ScrollArea::new([false, true]).show(ui, |ui| {
                    ui.heading("Editor");

                    for entity in editor.entities.iter_mut() {
                        entity.ui(ui, group_icon);
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show(ctx, |ui| editor.ui(ui));
    }
}
