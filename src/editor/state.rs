use std::collections::HashMap;

use nalgebra::Vector3;

use crate::{
    core::primitives::{Mesh, Trans},
    icon,
};

use super::{
    object::Object,
    tool::{Action, Tool},
    Editor,
};

/// Represents a selection
pub struct Selection {
    // Contains the meshes before the current transformation of the selected objects.
    selected: HashMap<u32, Mesh>,
    // The transformation that is to be applied.
    pub trans: Trans,

    pub inf: Vector3<f32>,
    pub sup: Vector3<f32>,
    pub origin: Vector3<f32>,
}

impl Selection {
    pub fn new(trans: Trans) -> Self {
        Self {
            selected: HashMap::new(),
            origin: Vector3::zeros(),
            inf: Vector3::zeros(),
            sup: Vector3::zeros(),
            trans,
        }
    }

    pub fn update_origin(&mut self, objects: &[Object]) {
        self.inf = Vector3::from_element(std::f32::INFINITY);
        self.sup = Vector3::from_element(std::f32::NEG_INFINITY);
        for object in objects.iter().filter(|o| self.selected.contains_key(&o.id)) {
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
        self.selected.contains_key(id)
    }

    pub fn valid(&self) -> bool {
        !self.selected.is_empty()
    }

    pub fn push_transformation(&mut self, objects: &mut [Object]) {
        for object in objects.iter_mut() {
            object.trans.push(self.trans.clone());
            self.selected
                .entry(object.id)
                .and_modify(|e| *e = object.mesh.clone());
        }
    }

    pub fn apply(&self, objects: &mut [Object]) {
        for object in objects.iter_mut() {
            if let Some(mesh) = self.selected.get(&object.id) {
                object.mesh = mesh.clone();
                object.mesh.apply(&self.trans);
            }
        }
    }
}

pub struct State {
    pub group_icon: egui::TextureHandle,
    pub object_icon: egui::TextureHandle,
    pub toolpath_icon: egui::TextureHandle,

    pub action: Option<Action>,

    pub selection: Selection,
    pub tool: Tool,
}

impl State {
    pub fn load(ctx: &egui::Context) -> Self {
        let group_icon = ctx.load_texture("group", icon!("group"), Default::default());
        let object_icon = ctx.load_texture("object", icon!("object"), Default::default());
        let toolpath_icon = ctx.load_texture("toolpath", icon!("toolpath"), Default::default());

        let tool = Tool::Move;

        Self {
            group_icon,
            object_icon,
            toolpath_icon,
            action: None,
            selection: Selection::new(tool.default_trans()),
            tool,
        }
    }

    pub fn switch_tool(&mut self, tool: Tool, objects: &mut [Object]) {
        self.selection.push_transformation(objects);
        self.selection.trans = tool.default_trans();
        self.tool = tool;
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
                        && state.selection.selected.contains_key(id)
                    {
                        continue;
                    }

                    state.selection.push_transformation(&mut editor.objects);

                    if !shift {
                        state.selection.selected.clear();
                    }

                    if state.selection.selected.contains_key(id) {
                        state.selection.selected.remove(id);
                    } else {
                        let object = editor.objects.iter_mut().find(|o| o.id == *id).unwrap();
                        state.selection.selected.insert(*id, object.mesh.clone());
                    }
                }
            }
        }
    }
}
