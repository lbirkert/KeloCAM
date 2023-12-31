use nalgebra::Vector3;
use pollster::FutureExt;

use rfd::{AsyncFileDialog, FileHandle};
use std::pin::Pin;
use std::task::Poll;
use std::{future::Future, io::Cursor};

use crate::view::{PrepareView, View};
use kelocam_core::primitives::{BoundingBox, Mesh};
use kelocam_editor::{object::Object, Editor};

pub struct KeloApp {
    file_dialog: Option<Pin<Box<dyn Future<Output = Option<FileHandle>>>>>,

    view: View,

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
                        if let Ok(mut mesh) = Mesh::from_stl(&mut Cursor::new(&handle.read().await))
                        {
                            let (min, max) = mesh.bb_min_max();
                            let size = max - min;
                            // Don't center on z axis
                            let transform = -min - Vector3::new(size.x, size.y, 0.0).scale(0.5);
                            mesh.translate(&transform);
                            let mut message = self
                                .editor
                                .state
                                .insert_object(Object::new(mesh, handle.file_name()));
                            self.editor.state.apply(&mut message);
                            self.editor.log.push(message);
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
                    if ui.button("Prepare").clicked() {
                        self.view = View::Prepare;
                    }
                });
            });
        });

        // The central panel the region left after adding TopPanel's and SidePanel's

        //match self.view {
        //    View::Prepare => self.prepare.show(ctx, &mut self.editor),
        //    _ => {}
        //};

        self.prepare.show(ctx, &mut self.editor)
    }
}
