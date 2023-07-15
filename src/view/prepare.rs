use crate::editor::state::{Message, State};
use crate::editor::Editor;

#[derive(Default)]
pub struct PrepareView {
    editor_state: Option<State>,
}

impl PrepareView {
    pub fn show(&mut self, ctx: &egui::Context, editor: &mut Editor) {
        let state = self.editor_state.get_or_insert_with(|| State::load(ctx));
        let mut messages = Vec::new();

        egui::SidePanel::left("my_left_panel")
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
                egui::ScrollArea::new([false, true]).show(ui, |ui| {
                    ui.heading("Editor");

                    editor.sidebar(ui, state, &mut messages);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show(ctx, |ui| editor.ui(ui, state, &mut messages));

        Message::process(editor, state, &messages, ctx);
    }
}
