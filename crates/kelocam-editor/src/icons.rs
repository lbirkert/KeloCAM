#[macro_export]
macro_rules! icon {
    ($name:tt) => {{
        let image = image::load_from_memory(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/icons/",
            $name,
            ".png"
        )))
        .expect("Failed to load image");
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }};
}

pub struct Icons {
    pub group: egui::TextureHandle,
    pub object: egui::TextureHandle,
    pub toolpath: egui::TextureHandle,
}

impl Icons {
    pub fn load(ctx: &egui::Context) -> Self {
        let group = ctx.load_texture("group", icon!("group"), Default::default());
        let object = ctx.load_texture("object", icon!("object"), Default::default());
        let toolpath = ctx.load_texture("toolpath", icon!("toolpath"), Default::default());

        Self {
            group,
            object,
            toolpath,
        }
    }
}
