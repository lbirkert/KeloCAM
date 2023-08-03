use crate::core::Mesh;
use egui::vec2;

use super::{icons::Icons, log::Message, state::State};

#[derive(Clone)]
pub struct Object {
    pub name: String,
    pub mesh: Mesh,
}

impl Object {
    pub fn new(mesh: Mesh, name: String) -> Self {
        Self { mesh, name }
    }

    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        id: u32,
        state: &State,
        icons: &Icons,
        messages: &mut Vec<Message>,
    ) {
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(&icons.object, vec2(16.0, 16.0)));

            let response = ui.selectable_label(state.selection.contains(&id), self.name.as_str());

            if response.clicked() {
                messages.push(state.perform_selection(ui, id));
            }

            response.context_menu(|ui| {
                if ui.button("Delete").clicked() {
                    ui.close_menu();
                    messages.push(state.delete_object(id));
                }
            });
        });
    }
}
