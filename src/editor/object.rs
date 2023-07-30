use egui::vec2;
use nalgebra::Vector3;

use crate::core::primitives::{Mesh, Trans};

use super::state;

pub struct Object {
    pub name: String,

    // Current mesh
    pub mesh: Mesh,
    // Transformations that were applied to the mesh
    pub trans: Vec<Trans>,
    // Mesh before all transformations
    pub premesh: Mesh,

    pub id: u32,
}

impl Object {
    pub fn new(mesh: Mesh, name: String, id: u32) -> Self {
        Self {
            mesh: mesh.clone(),
            premesh: mesh,
            trans: Vec::new(),
            name,
            id,
        }
    }

    /// Reconstructs the current mesh by applying all of the transformations stored in trans to
    /// a clone of the mesh that is stored in the premesh field.
    pub fn reconstruct(&mut self) {
        let mut mesh = self.premesh.clone();
        for trans in self.trans.iter() {
            mesh.apply(trans);
        }
        self.mesh = mesh;
    }

    pub fn snap_to_plate(&mut self) {
        let (inf, _) = self.mesh.inf_sup();

        self.mesh.translate(&Vector3::new(0.0, 0.0, -inf.z));
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut state::State,
        messages: &mut Vec<state::Message>,
    ) {
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(&state.object_icon, vec2(16.0, 16.0)));

            let response =
                ui.selectable_label(state.selection.contains(&self.id), self.name.as_str());

            if response.clicked() {
                messages.push(state::Message::Select(self.id));
            }

            response.context_menu(|ui| {
                if ui.button("Delete").clicked() {
                    ui.close_menu();
                    messages.push(state::Message::Delete(self.id));
                }
            });
        });
    }
}
