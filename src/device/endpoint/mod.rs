pub mod serial;

pub trait Endpoint {
    fn open(&mut self) -> Result<(), String>;
    fn write(&mut self, buf: &[u8]) -> Result<(), String>;
    fn read(&mut self) -> Vec<String>;
    fn close(&mut self) -> Result<(), String>;
}
