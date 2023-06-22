use std::collections::HashSet;

use crate::icon;

use super::Editor;

pub struct State {
    pub selected: HashSet<u32>,

    pub group_icon: egui::TextureHandle,
    pub object_icon: egui::TextureHandle,
    pub toolpath_icon: egui::TextureHandle,
}

impl State {
    pub fn load(ctx: &egui::Context) -> Self {
        let group_icon = ctx.load_texture("group", icon!("group"), Default::default());
        let object_icon = ctx.load_texture("object", icon!("object"), Default::default());
        let toolpath_icon = ctx.load_texture("toolpath", icon!("toolpath"), Default::default());

        Self {
            selected: HashSet::new(),

            group_icon,
            object_icon,
            toolpath_icon,
        }
    }
}

pub enum Message {
    Delete(u32),
    Group(),
    Ungroup(),
}

impl Message {
    pub fn process(&self, editor: &mut Editor, state: &mut State) {
        match self {
            Self::Delete(id) => {
                editor.remove(*id);
            }
            Self::Group() => {
                let group_id = editor.uid();
                for entity in editor.entities.iter_mut() {
                    if state.selected.contains(&entity.id()) {
                        entity.set_id(group_id);
                    }
                }
            }
            Self::Ungroup() => {
                // Create temp value because we cannot mutate id_counter while having editor
                // borrowed via iter_mut()
                let mut id_counter = editor.id_counter;
                for entity in editor.entities.iter_mut() {
                    if state.selected.contains(&entity.id()) {
                        id_counter += 1;
                        entity.set_id(id_counter);
                    }
                }

                editor.id_counter = id_counter;
            }
        }
    }
}
