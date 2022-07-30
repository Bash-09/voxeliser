use egui::{Align2, Vec2};
use egui_winit::winit::{window::WindowBuilder, dpi::PhysicalSize, event::{WindowEvent, Event}};
use glium::{Surface, Display};
use glium_app::Application;
use model::{Model, loader};
use renderer::Renderer;
use rfd::FileDialog;

pub mod model;
pub mod renderer;

fn main() {

    let app = App::new();
    let wb = WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(PhysicalSize::new(1000, 600))
        .with_title("Voxeliser!");

    glium_app::run(app, wb);
}

struct App {
    renderer: Option<Renderer>,
    model: Option<Model>,
}

impl Application for App {
    fn init(&mut self, ctx: &mut glium_app::context::Context) {
        self.renderer = Some(Renderer::new(&ctx.dis));
    }

    fn update(&mut self, _t: &glium_app::Timer, ctx: &mut glium_app::context::Context) {
        if self.renderer.is_none() {panic!("No renderer!! AHHH")}

        // Render gui
        let mut target = ctx.dis.draw();
        target.clear_color_and_depth((0.5, 0.7, 0.8, 1.0), 1.0);

        // Scene
        if let Some(model) = &self.model {
            self.renderer.as_mut().unwrap().render_model(&mut target, model);
        }

        // Gui
        let _ = ctx.gui.run(&ctx.dis, |gui_ctx| {

            // Top menu bar
            egui::TopBottomPanel::top("top panel").show(gui_ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Import model").clicked() {
                            self.import_model(&ctx.dis);
                        }
                    });
                });
            });

            // Import model prompt
            if self.model.is_none() {
                egui::Window::new("Load a model")
                .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
                .resizable(false)
                .show(gui_ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("You must ");
                        if ui.selectable_label(false, "import a model").clicked() {
                            self.import_model(&ctx.dis);
                        }
                    });
                });

                return;
            }

            // Main control window
            egui::Window::new("Hello World!").show(gui_ctx, |ui| {
                ui.label("Ded");

                let model = self.model.as_mut().unwrap();
                ui.collapsing("Model Settings", |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut model.scale).speed(0.01));
                        ui.label("Scale");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut model.pos.x).speed(0.01));
                        ui.label("X");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut model.pos.y).speed(0.01));
                        ui.label("Y");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut model.pos.z).speed(0.01));
                        ui.label("Z");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut model.rot.x).speed(0.01));
                        ui.label("Rot X");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut model.rot.y).speed(0.01));
                        ui.label("Rot Y");
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut model.rot.z).speed(0.01));
                        ui.label("Rot Z");
                    });
                });
            });

        });
        ctx.gui.paint(&ctx.dis, &mut target);
        target.finish().unwrap();
    }

    fn close(&mut self, _ctx: &glium_app::context::Context) {
        
    }

    fn handle_event(&mut self, _ctx: &mut glium_app::context::Context, event: &egui_winit::winit::event::Event<()>) {
        if let Event::WindowEvent{ window_id: _, event: WindowEvent::Resized(size)} = event {
            if let Some(rend) = &mut self.renderer {
                rend.cam.set_window_size((size.width, size.height));
            }
        }
    }
}

impl App {
    pub fn new() -> App {
        App { renderer: None, model: None }
    }

    fn import_model(&mut self, dis: &Display) {
        if let Some(pb) = FileDialog::new()
        .add_filter("gltf/obj", &["gltf", "glb", "obj"])
        .pick_file() {
            match loader::load_model(pb) {
                Ok((verts, inds)) =>  self.model = Some(Model::new(dis, verts, inds)),
                Err(e) => todo!("{}", e),
            }
        }
    }
}