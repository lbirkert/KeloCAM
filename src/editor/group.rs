use egui::vec2;
use nalgebra::Vector3;

use super::{sidebar, Entity, Message};

pub struct Group {
    pub name: String,
    pub expanded: bool,

    pub entities: Vec<Entity>,

    pub id: u32,
}

impl Group {
    pub fn scale(&mut self, delta: Vector3<f32>) {
        for entity in self.entities.iter_mut() {
            entity.scale(delta);
        }
    }

    pub fn translate(&mut self, delta: Vector3<f32>) {
        for entity in self.entities.iter_mut() {
            entity.translate(delta);
        }
    }

    pub fn rotate(&mut self, delta: Vector3<f32>) {
        for entity in self.entities.iter_mut() {
            entity.rotate(delta);
        }
    }

    pub fn inf_sup(&self) -> (Vector3<f32>, Vector3<f32>) {
        let mut inf = Vector3::from_element(std::f32::INFINITY);
        let mut sup = Vector3::from_element(std::f32::NEG_INFINITY);

        for entity in self.entities.iter() {
            let (einf, esup) = entity.inf_sup();
            inf = inf.inf(&einf);
            sup = sup.sup(&esup);
        }

        (inf, sup)
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        sidebar: &mut sidebar::Sidebar,
        messages: &mut Vec<Message>,
    ) {
        ui.horizontal(|ui| {
            ui.add(egui::Image::new(&sidebar.group_icon, vec2(16.0, 16.0)));
            let response = ui.selectable_label(sidebar.selected == self.id, self.name.as_str());

            if response.double_clicked() {
                self.expanded ^= true;
            }

            if response.clicked() {
                sidebar.selected = self.id;
            }

            response.context_menu(|ui| {
                if ui.button("Delete").clicked() {
                    messages.push(Message::Delete(self.id))
                }
            });
        });

        if self.expanded {
            ui.indent(0, |ui| {
                for entity in self.entities.iter_mut() {
                    entity.ui(ui, sidebar, messages);
                }
            });
        }
    }
}
