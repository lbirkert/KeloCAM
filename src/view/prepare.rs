use crate::editor::icons::Icons;
use crate::editor::Editor;

#[derive(Default)]
pub struct PrepareView {
    icons: Option<Icons>,
}

impl PrepareView {
    pub fn show(&mut self, ctx: &egui::Context, editor: &mut Editor) {
        let icons = self.icons.get_or_insert_with(|| Icons::load(ctx));
        let mut messages = Vec::new();

        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(150.0)
            .width_range(150.0..=250.0)
            .show_separator_line(true)
            .show(ctx, |ui| {
                egui::ScrollArea::new([false, true]).show(ui, |ui| {
                    editor.sidebar(ui, icons, &mut messages);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show(ctx, |ui| editor.ui(ui, &mut messages));

        for mut message in messages {
            editor.state.apply(&mut message);
            editor.log.push(message);
        }
    }
}
