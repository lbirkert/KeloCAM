use crate::device::{get_devices, Device, DeviceInfo};

#[derive(Default)]
pub struct MonitorView {
    devices: Option<Vec<DeviceInfo>>,
    selected: Option<DeviceInfo>,
    device: Option<Device>,

    command: String,
    log: String,
}

impl MonitorView {
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MonitorView");

            if let Some(ref devices) = self.devices {
                for device in devices {
                    if ui
                        .selectable_label(
                            Some(device.clone()) == self.selected,
                            device.descriptor.as_str(),
                        )
                        .clicked()
                    {
                        self.selected = Some(device.clone());
                    }
                }
            } else {
                let devices = get_devices();

                if let Some(ref selected) = self.selected {
                    'label: {
                        for device in &devices {
                            if device == selected {
                                break 'label;
                            }
                        }
                        self.selected = None;
                    }
                }

                self.devices = Some(devices);
            }

            if ui.button("Rescan").clicked() {
                self.devices = None;
            }

            ui.set_enabled(self.selected.is_some());
            if ui.button("Connect").clicked() {
                let mut device = Device::from(self.selected.as_ref().unwrap());
                device.connect().unwrap();
                self.device = Some(device);
            }

            if let Some(device) = self.device.as_mut() {
                ui.add(egui::TextEdit::singleline(&mut self.command).hint_text("Enter command"));

                if ui.button("Send").clicked() {
                    self.log += self.command.as_str();
                    self.log += "\n";
                    device.endpoint.write(self.command.as_bytes()).unwrap();
                    device.endpoint.write("\r\n".as_bytes()).unwrap();
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show_viewport(ui, |ui, _| {
                        ui.add(egui::TextEdit::multiline(&mut self.log.as_str()));
                    });

                self.log += &mut device.endpoint.read().join("\n");
            }

            println!("DEVICES: {:?}", self.devices);
        });
    }
}
