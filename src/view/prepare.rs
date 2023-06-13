use crate::widget::viewer::Viewer;

#[derive(Default)]
pub struct PrepareView {
    edit: Option<usize>,
    edit_content: String,

    selected: Option<usize>,
}

impl PrepareView {
    pub fn show(&mut self, ctx: &egui::Context, viewer: &mut Viewer) {
        egui::SidePanel::left("my_left_panel")
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.heading("Editor");

                let mut remove = None;
                for (i, object) in viewer.objects.iter_mut().enumerate() {
                    if self.edit == Some(i) {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.edit_content).lock_focus(true),
                        );
                        if response.lost_focus() {
                            object.name = Some(self.edit_content.clone());
                            self.edit = None;
                        } else if !response.has_focus() {
                            response.request_focus();
                        }
                    } else {
                        let name = object.name.clone().unwrap_or_else(|| format!("Object {i}"));

                        let response = ui.selectable_label(self.selected == Some(i), name.as_str());

                        if response.clicked() {
                            self.selected = Some(i);
                        }

                        if response.double_clicked() {
                            self.edit_content = name;
                            self.edit = Some(i);
                        }

                        response.context_menu(|ui| {
                            if ui.button("Delete").clicked() {
                                self.selected = None;
                                self.edit = None;
                                remove = Some(i);

                                ui.close_menu();
                            }
                        });
                    }
                }

                if let Some(remove) = remove {
                    viewer.objects.remove(remove);
                    viewer.object_changed = true;
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show(ctx, |ui| {
                viewer.ui(ui);
            });
    }
}
