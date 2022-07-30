use glam::Vec3;
use glium::implement_vertex;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub norm: [f32; 3],
}

implement_vertex!(Vertex, pos, norm);

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3]) -> Vertex {
        Vertex {
            pos: position,
            norm: normal,
        }
    }

    pub fn from_gltf(vert: &easy_gltf::model::Vertex) -> Vertex {
        Vertex {
            pos: [vert.position.x, vert.position.y, vert.position.z],
            norm: [vert.normal.x, vert.normal.y, vert.normal.z],
        }
    }

    pub fn pos_vec(&self) -> Vec3 {
        Vec3::from(self.pos)
    }

    pub fn norm_vec(&self) -> Vec3 {
        Vec3::from(self.norm)
    }
}
