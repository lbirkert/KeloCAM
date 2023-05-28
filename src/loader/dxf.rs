use dxf::Drawing;
use dxf::entities::*;

pub fn load(path: &str) {
    let drawing = Drawing::load_file(path)?;
    for e in drawing.entities() {
        println!("found entity on layer {}", e.common.layer);
        match e.specific {
            EntityType::Circle(ref circle) => {
                // do something with the circle
            },
            EntityType::Line(ref line) => {
                // do something with the line
            },
            _ => (),
        }
    }
}
