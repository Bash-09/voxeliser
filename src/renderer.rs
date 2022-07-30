use std::path::Path;

use glam::{EulerRot, Mat4, Vec3};
use glium::{
    draw_parameters,
    uniform, BackfaceCullingMode, Depth, Display, DrawParameters, Frame,
    Program, Surface,
};

use crate::model::Model;

use self::camera::Camera;

pub mod camera;
pub mod shader;

pub struct Renderer {
    pub cam: Camera,

    normal_prog: Program,
}

impl Renderer {
    pub fn new(dis: &Display) -> Renderer {
        // let vbo = glium::VertexBuffer::new(dis, &shape).unwrap();
        let normal_prog = shader::read_shader(
            dis,
            Path::new("shaders/v.glsl"),
            Path::new("shaders/f.glsl"),
        )
        .expect("Failed to compile shaders");

        Renderer {
            cam: Camera::new_with_values(
                dis.get_framebuffer_dimensions(),
                Vec3::new(0.0, 1.0, 3.0),
                Vec3::new(180.0, 0.0, 0.0),
                90.0,
            ),

            normal_prog,
        }
    }

    pub fn render_model(
        &mut self,
        target: &mut Frame,
        model: &Model,
    ) {
        let params = DrawParameters {
            depth: Depth {
                test: draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let pvmat = self.cam.get_pvmat().to_cols_array_2d();
        let mut tmat: Mat4 = Mat4::from_translation(model.pos);
        tmat *= Mat4::from_scale(Vec3::splat(model.scale));
        tmat *= Mat4::from_euler(EulerRot::XYZ, model.rot.x, model.rot.y, model.rot.z);

        let uniforms = uniform! {
            pvmat: pvmat,
            tmat: tmat.to_cols_array_2d(),
        };
        target
            .draw(
                &model.vbo,
                &model.ind_buf,
                &self.normal_prog,
                &uniforms,
                &params,
            )
            .unwrap();
    }
}
