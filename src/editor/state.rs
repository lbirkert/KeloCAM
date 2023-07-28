use std::collections::HashSet;

use nalgebra::Vector3;

use crate::icon;

use super::{object::Object, tool, Editor};

/// Represents a selection
pub struct Selection {
    selected: HashSet<u32>,

    pub inf: Vector3<f32>,
    pub sup: Vector3<f32>,
    pub origin: Vector3<f32>,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
            origin: Vector3::zeros(),
            inf: Vector3::zeros(),
            sup: Vector3::zeros(),
        }
    }

    pub fn update_origin(&mut self, objects: &[Object]) {
        self.inf = Vector3::from_element(std::f32::INFINITY);
        self.sup = Vector3::from_element(std::f32::NEG_INFINITY);
        for object in objects.iter().filter(|o| self.selected.contains(&o.id)) {
            let (oinf, osup) = object.mesh.inf_sup();
            self.inf = self.inf.inf(&oinf);
            self.sup = self.sup.sup(&osup);
        }

        self.origin = (self.inf + self.sup).scale(0.5);
    }

    pub fn clear(&mut self) {
        self.selected.clear();
    }

    pub fn contains(&self, id: &u32) -> bool {
        self.selected.contains(id)
    }

    pub fn valid(&self) -> bool {
        !self.selected.is_empty()
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

pub struct State {
    pub group_icon: egui::TextureHandle,
    pub object_icon: egui::TextureHandle,
    pub toolpath_icon: egui::TextureHandle,

    pub action: Option<tool::Action>,

    pub selection: Selection,
    pub tool: tool::Tool,
}

impl State {
    pub fn load(ctx: &egui::Context) -> Self {
        let group_icon = ctx.load_texture("group", icon!("group"), Default::default());
        let object_icon = ctx.load_texture("object", icon!("object"), Default::default());
        let toolpath_icon = ctx.load_texture("toolpath", icon!("toolpath"), Default::default());

        let tool = tool::Tool::Move;

        Self {
            group_icon,
            object_icon,
            toolpath_icon,
            action: None,
            selection: Selection::new(),
            tool,
        }
    }
}

pub enum Message {
    Delete(u32),
    Select(u32),
}

impl Message {
    pub fn process(
        editor: &mut Editor,
        state: &mut State,
        messages: &[Message],
        ctx: &egui::Context,
    ) {
        for message in messages {
            match message {
                Self::Delete(id) => {
                    let index = editor.objects.iter().position(|x| x.id == *id).unwrap();
                    editor.objects.remove(index);
                }
                Self::Select(id) => {
                    let shift = ctx.input(|i| i.modifiers.shift);

                    if !shift
                        && state.selection.selected.len() == 1
                        && state.selection.selected.contains(id)
                    {
                        continue;
                    }

                    if !shift {
                        state.selection.selected.clear();
                    }

                    if state.selection.selected.contains(id) {
                        state.selection.selected.remove(id);
                    } else {
                        state.selection.selected.insert(*id);
                    }
                }
            }
        }
    }
}
