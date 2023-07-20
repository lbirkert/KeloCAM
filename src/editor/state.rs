use std::collections::HashSet;

use nalgebra::Vector3;

use crate::icon;

use super::{
    object::{ApplyableTransformation, Object, Transformation},
    tool, Editor,
};

/// Represents a selection
pub struct Selection {
    selected: HashSet<u32>,

    pub transformation: Transformation,
    pub pre_inf: Vector3<f32>,
    pub pre_sup: Vector3<f32>,
    pub origin: Vector3<f32>,
}

impl Selection {
    pub fn new(tool: &tool::Tool) -> Self {
        Self {
            selected: HashSet::new(),

            transformation: tool.selection_transformation(),
            origin: Vector3::zeros(),
            pre_inf: Vector3::zeros(),
            pre_sup: Vector3::zeros(),
        }
    }

    pub fn update_origin(&mut self, objects: &[Object]) {
        self.pre_inf = Vector3::from_element(std::f32::INFINITY);
        self.pre_sup = Vector3::from_element(std::f32::NEG_INFINITY);
        for object in objects.iter().filter(|o| self.selected.contains(&o.id)) {
            let (oinf, osup) = object.inf_sup();
            self.pre_inf = self.pre_inf.inf(&oinf);
            self.pre_sup = self.pre_sup.sup(&osup);
        }

        let pre_origin = (self.pre_sup + self.pre_inf).scale(0.5);

        self.origin = match self.transformation {
            Transformation::Scale { .. } | Transformation::Rotate { .. } => pre_origin,
            Transformation::Translate { translate } => pre_origin + translate,
        };
    }

    pub fn clear(&mut self, objects: &mut [Object]) {
        self.apply(objects);
        self.selected.clear();
    }

    pub fn contains(&self, id: &u32) -> bool {
        self.selected.contains(id)
    }

    pub fn valid(&self) -> bool {
        !self.selected.is_empty()
    }

    pub fn to_applyable(&self) -> ApplyableTransformation {
        self.transformation.to_applyable(self.origin)
    }

    pub fn apply(&mut self, objects: &mut [Object]) {
        let applyable = self.to_applyable();
        for object in objects.iter_mut().filter(|o| self.selected.contains(&o.id)) {
            object.transform(&applyable);
            object.snap_to_plate();
        }

        self.transformation.reset();
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
            selection: Selection::new(&tool),
            tool,
        }
    }

    pub fn switch_tool(&mut self, tool: tool::Tool, objects: &mut [Object]) {
        if self.tool != tool {
            self.selection.apply(objects);
            self.selection.transformation = tool.selection_transformation();
        }
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
        let mut applied = false;

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

                    if !applied {
                        applied = true;
                        state.selection.apply(&mut editor.objects);
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
