#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::KeloApp;

pub mod device;
pub mod view;

pub mod editor;

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
