pub struct Tool {
    pub name: String,
    pub radius: f32,
}

impl Tool {
    pub fn new(name: String, radius: f32) -> Self {
        Self { name, radius }
    }
}
