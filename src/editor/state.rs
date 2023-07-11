use std::collections::HashSet;

use crate::icon;

use super::Editor;

pub enum Tool {
    Move,
    Rotate,
    Scale,
}

pub struct State {
    pub selected: HashSet<u32>,

    pub group_icon: egui::TextureHandle,
    pub object_icon: egui::TextureHandle,
    pub toolpath_icon: egui::TextureHandle,

    pub tool: Tool,
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
            tool: Tool::Move,
        }
    }
}

pub enum Message {
    Delete(u32),
}

impl Message {
    pub fn process(&self, editor: &mut Editor, _state: &mut State) {
        match self {
            Self::Delete(id) => {
                editor.remove(*id);
            }
        }
    }
}
