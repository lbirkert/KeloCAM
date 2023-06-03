mod endpoint;

use endpoint::serial::SerialEndpoint;
use endpoint::Endpoint;

#[derive(PartialEq, Debug, Clone)]
pub struct DeviceInfo {
    pub descriptor: String,
    pub device_type: DeviceType,
}

#[derive(PartialEq, Debug, Clone)]
pub enum DeviceType {
    SERIAL,
}

pub fn get_devices() -> Vec<DeviceInfo> {
    SerialEndpoint::find()
        .iter()
        .map(|p| DeviceInfo {
            descriptor: p.port_name.clone(),
            device_type: DeviceType::SERIAL,
        })
        .collect()
}

pub struct Device {
    pub endpoint: Box<dyn Endpoint>,
}

impl Device {
    pub fn from(info: &DeviceInfo) -> Self {
        let endpoint = Box::new(match info.device_type {
            DeviceType::SERIAL => {
                SerialEndpoint::from(info.descriptor.clone(), endpoint::serial::DEFAULT_BAUD_RATE)
            }
        });

        Self { endpoint }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        self.endpoint.open()
    }
}
