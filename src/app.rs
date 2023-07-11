use pollster::FutureExt;

use rfd::{AsyncFileDialog, FileHandle};
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use crate::{
    editor::{object::Object, Editor},
    view::{monitor::MonitorView, prepare::PrepareView, View},
};

pub struct KeloApp {
    file_dialog: Option<Pin<Box<dyn Future<Output = Option<FileHandle>>>>>,

    view: View,

    monitor: MonitorView,
    prepare: PrepareView,

    editor: Editor,
}

impl KeloApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let editor = Editor::new(cc).expect("Error while creating editor");

        Self {
            file_dialog: None,
            view: View::Prepare,
            monitor: MonitorView::default(),
            prepare: PrepareView::default(),
            editor,
        }
    }
}

impl eframe::App for KeloApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(file_dialog) = &mut self.file_dialog {
            if let Poll::Ready(handle) = async { futures::poll!(file_dialog.as_mut()) }.block_on() {
                self.file_dialog = None;

                if let Some(handle) = handle {
                    async {
                        if let Ok(object) = Object::from_stl(
                            handle.file_name(),
                            handle.read().await,
                            &mut self.editor.id_counter,
                        ) {
                            self.editor.objects.push(object);
                            self.editor.object_changed = true;
                        }
                    }
                    .block_on();
                }
            };
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.file_dialog = Some(Box::pin(
                            AsyncFileDialog::new()
                                .add_filter("STL Files", &["stl"])
                                .set_directory("/")
                                .pick_file(),
                        ));

                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
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

        // The central panel the region left after adding TopPanel's and SidePanel's

        match self.view {
            View::Monitor => self.monitor.show(ctx),
            View::Prepare => self.prepare.show(ctx, &mut self.editor),
            _ => {}
        };
    }
}
