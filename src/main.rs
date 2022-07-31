use egui::{Align2, Vec2};
use egui_winit::winit::{
    dpi::PhysicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    window::WindowBuilder,
};
use glam::Vec3;
use glium::{Display, Surface};
use glium_app::{context::Context, Application};
use model::{loader, Model};
use renderer::Renderer;
use rfd::FileDialog;

pub mod model;
pub mod renderer;
pub mod voxeliser;

const SENSITIVITY: f32 = 0.05;
const VELOCITY: f32 = 5.0;

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
    voxel_model: Option<Model>,

    mouse_grabbed: bool,
}

impl Application for App {
    fn init(&mut self, ctx: &mut glium_app::context::Context) {
        self.renderer = Some(Renderer::new(&ctx.dis));
    }

    fn update(&mut self, t: &glium_app::Timer, ctx: &mut glium_app::context::Context) {
        if self.renderer.is_none() {
            panic!("No renderer!! AHHH")
        }

        // Mouse movements
        if self.mouse_grabbed {
            self.move_camera(ctx, t.delta());
        }

        // Render gui
        let mut target = ctx.dis.draw();
        target.clear_color_and_depth((0.5, 0.7, 0.8, 1.0), 1.0);

        // Scene
        if let Some(model) = &self.model {
            self.renderer
                .as_mut()
                .unwrap()
                .render_model(&mut target, model);
        }

        // Gui
        let _ = ctx.gui.run(&ctx.dis, |gui_ctx| {
            // Import model prompt
            if self.model.is_none() {
                egui::Window::new("Load a model")
                    .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
                    .resizable(false)
                    .collapsible(false)
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
            if !self.mouse_grabbed {
                egui::Window::new("Controls").show(gui_ctx, |ui| {
                    let resp = ui.button("Import new model");
                    if resp.clicked() {
                        self.import_model(&ctx.dis);
                    }
                    resp.on_hover_text("This will remove the current model");

                    let resp = ui.button("Move Camera");
                    if resp.clicked() {
                        self.mouse_grabbed = true;
                    }
                    resp.on_hover_text("Press escape at any time to stop");

                    if ui.button("Reset camera").clicked() {
                        self.renderer
                            .as_mut()
                            .unwrap()
                            .cam
                            .set_pos(Vec3::new(0.0, 1.0, 3.0));
                        self.renderer
                            .as_mut()
                            .unwrap()
                            .cam
                            .set_rot(Vec3::new(180.0, 0.0, 0.0));
                    }

                    // Model settings
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
            }
        });
        ctx.gui.paint(&ctx.dis, &mut target);
        target.finish().unwrap();

        // Cancel camera control and keep mouse contained
        if self.mouse_grabbed && ctx.keyboard.pressed_this_frame(&VirtualKeyCode::Escape) {
            self.mouse_grabbed = false;
        }
        ctx.set_mouse_grabbed(self.mouse_grabbed).ok();
        ctx.set_mouse_visible(!self.mouse_grabbed);
    }

    fn close(&mut self, _ctx: &glium_app::context::Context) {}

    fn handle_event(
        &mut self,
        _ctx: &mut glium_app::context::Context,
        event: &egui_winit::winit::event::Event<()>,
    ) {
        if let Event::WindowEvent {
            window_id: _,
            event: WindowEvent::Resized(size),
        } = event
        {
            if let Some(rend) = &mut self.renderer {
                rend.cam.set_window_size((size.width, size.height));
            }
        }
    }
}

impl App {
    pub fn new() -> App {
        App {
            renderer: None,
            model: None,
            voxel_model: None,
            mouse_grabbed: false,
        }
    }

    fn import_model(&mut self, dis: &Display) {
        if let Some(pb) = FileDialog::new()
            .add_filter("gltf/obj", &["gltf", "glb", "obj"])
            .pick_file()
        {
            match loader::load_model(pb) {
                Ok((verts, inds)) => self.model = Some(Model::new(dis, verts, inds)),
                Err(e) => todo!("{}", e),
            }
        }
    }

    fn move_camera(&mut self, ctx: &Context, delta: f32) {
        if self.renderer.is_none() {
            panic!("Dum")
        }
        let cam = &mut self.renderer.as_mut().unwrap().cam;
        let vel = VELOCITY * delta;

        if ctx.keyboard.is_pressed(&VirtualKeyCode::W) {
            let mut dir = cam.get_look_vector();
            dir.y = 0.0;
            dir = dir.normalize();
            dir *= vel;
            cam.translate(dir);
        }

        if ctx.keyboard.is_pressed(&VirtualKeyCode::S) {
            let mut dir = cam.get_look_vector();
            dir.y = 0.0;
            dir = dir.normalize();
            dir *= -vel;
            cam.translate(dir);
        }

        if ctx.keyboard.is_pressed(&VirtualKeyCode::A) {
            let mut dir = cam.get_look_vector();
            dir.y = 0.0;
            dir = dir.normalize();
            dir *= -vel;
            dir.y = dir.x;
            dir.x = -dir.z;
            dir.z = dir.y;
            dir.y = 0.0;
            cam.translate(dir);
        }

        if ctx.keyboard.is_pressed(&VirtualKeyCode::D) {
            let mut dir = cam.get_look_vector();
            dir.y = 0.0;
            dir = dir.normalize();
            dir *= vel;
            dir.y = dir.x;
            dir.x = -dir.z;
            dir.z = dir.y;
            dir.y = 0.0;
            cam.translate(dir);
        }

        if ctx.keyboard.is_pressed(&VirtualKeyCode::Space) {
            cam.translate(Vec3::new(0.0, vel, 0.0));
        }

        if ctx.keyboard.is_pressed(&VirtualKeyCode::LShift) {
            cam.translate(Vec3::new(0.0, -vel, 0.0));
        }

        let off = ctx.mouse.get_delta();
        cam.rotate(Vec3::new(
            off.0 as f32 * -SENSITIVITY,
            off.1 as f32 * -SENSITIVITY,
            0.0,
        ));
    }
}
