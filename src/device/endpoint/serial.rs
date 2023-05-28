use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use crate::device::endpoint::Endpoint;

use std::time::Duration;

pub const DEFAULT_BAUD_RATE: u32 = 115200;

pub struct SerialEndpoint {
    pub port: String,
    pub baud_rate: u32,

    serial_port: Option<Box<dyn SerialPort>>,
}

impl SerialEndpoint {
    pub fn from(port: String, baud_rate: u32) -> Self {
        Self {
            port,
            baud_rate: baud_rate,

            serial_port: None
        }
    }

    pub fn find() -> Vec<SerialPortInfo> {
        match serialport::available_ports() {
            Ok(serial_ports) => {
                println!("{:#?}", serial_ports);
                serial_ports.into_iter()
                .filter(|port| matches!(port.port_type, SerialPortType::UsbPort(_))).collect()
            },
            Err(_) => Vec::new(),
        }
    }
}

impl Endpoint for SerialEndpoint {
    fn open(&mut self) -> Result<(), String> {
        self.serial_port = match serialport::new(self.port.clone(), self.baud_rate)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .timeout(Duration::from_millis(1)).open() {
            Ok(serial_port) => Some(serial_port),
            Err(_) => return Err("Error while opening device".into()),
        }; 

        Ok(())
    }

    fn read(&mut self) -> Vec<String> {
        let serial_port = self.serial_port.as_mut().expect("Not open");

        if let Ok(bytes) = serial_port.bytes_to_read() {
            let mut byte_buffer: Vec<u8> = vec![0; bytes as usize];
            let _ = serial_port.read(&mut byte_buffer);

            match String::from_utf8(byte_buffer) {
                Ok(s) => {
                    return s.split("\r\n").map(|s| String::from(s)).collect();        
                },
                Err(_) => {},
            }
        }

        return Vec::new();
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), String> {
        match self.serial_port.as_mut().expect("Not open").write(buf) {
            Ok(_) => Ok(()),
            Err(_) => Err("Couldn't write".into())
        }
    }
    
    fn close(&mut self) -> Result<(), String> {
        // TODO: find out how to close
        Ok(())
    }
}

impl Drop for SerialEndpoint {
    fn drop(&mut self) {
        if let Some(_) = self.serial_port {
            if let Err(_) = self.close() {
                panic!("Could not close serial endpoint");
            }
        }
    }
}
