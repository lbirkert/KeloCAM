use crate::icon;

pub struct Sidebar {
    pub selected: u32,

    pub group_icon: egui::TextureHandle,
    pub object_icon: egui::TextureHandle,
    pub toolpath_icon: egui::TextureHandle,
}

impl Sidebar {
    pub fn load(ctx: &egui::Context) -> Self {
        let group_icon = ctx.load_texture("group", icon!("group"), Default::default());
        let object_icon = ctx.load_texture("object", icon!("object"), Default::default());
        let toolpath_icon = ctx.load_texture("toolpath", icon!("toolpath"), Default::default());

        Self {
            selected: 0,

            group_icon,
            object_icon,
            toolpath_icon,
        }
    }
}

pub enum Message {
    Delete(u32),
}
