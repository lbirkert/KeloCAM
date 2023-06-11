use pollster::FutureExt;

use rfd::{AsyncFileDialog, FileHandle};
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use crate::widget::viewer::Viewer;

use crate::view::{monitor::MonitorView, prepare::PrepareView, View};

pub struct KeloApp {
    file_dialog: Option<Pin<Box<dyn Future<Output = Option<FileHandle>>>>>,

    view: View,

    monitor: MonitorView,
    prepare: PrepareView,

    viewer: Viewer,
}

impl KeloApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let viewer = Viewer::new(cc).expect("Error while creating viewer");

        Self {
            file_dialog: None,
            view: View::Prepare,
            monitor: MonitorView::default(),
            prepare: PrepareView::default(),
            viewer,
        }
    }
}

impl eframe::App for KeloApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(file_dialog) = &mut self.file_dialog {
            if let Poll::Ready(handle) = async { futures::poll!(file_dialog.as_mut()) }.block_on() {
                self.file_dialog = None;

                println!("{handle:?}");
            };
        }

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.file_dialog = Some(Box::pin(
                            AsyncFileDialog::new()
                                .add_filter("text", &["txt", "rs"])
                                .add_filter("rust", &["rs", "toml"])
                                .set_directory("/")
                                .pick_file(),
                        ));
                    }
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Monitor").clicked() {
                        self.view = View::Monitor;
                    }
                    if ui.button("Prepare").clicked() {
                        self.view = View::Prepare;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            match self.view {
                View::Monitor => self.monitor.show(ui),
                View::Prepare => self.prepare.show(ui, &mut self.viewer),
                _ => {}
            };
        });
    }
}
