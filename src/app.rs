use egui_file::FileDialog;

use crate::view::{monitor::MonitorView, View};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct KeloApp {
    #[serde(skip)]
    file_dialog: Option<FileDialog>,

    #[serde(skip)]
    view: View,

    monitor: MonitorView,
}

impl Default for KeloApp {
    fn default() -> Self {
        Self {
            file_dialog: None,
            view: View::Monitor,
            monitor: MonitorView::default(),
        }
    }
}

impl KeloApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for KeloApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // For inspiration and more examples, go to https://emilk.github.io/egui

        if let Some(dialog) = &mut self.file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    println!("FILE: {}", file.display());
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        println!("kekws");
                        let mut dialog = FileDialog::open_file(None);
                        dialog.open();
                        self.file_dialog = Some(dialog);
                    }
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Monitor").clicked() {
                        self.view = View::Monitor;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            match self.view {
                View::Monitor => self.monitor.show(ui),  
                _ => {}
            };
        });
    }
}
