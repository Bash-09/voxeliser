use glam::Vec3;
use glium::{index::PrimitiveType, Display, IndexBuffer, VertexBuffer};

use self::vertex::Vertex;

pub mod loader;
pub mod vertex;

pub struct Model {
    pub vbo: VertexBuffer<Vertex>,
    pub ind_buf: IndexBuffer<u32>,

    pub verts: Vec<Vertex>,
    pub inds: Vec<usize>,

    pub render: bool,

    pub pos: Vec3,
    pub rot: Vec3,
    pub scale: f32,
}

impl Model {
    pub fn new(dis: &Display, verts: Vec<Vertex>, inds: Vec<usize>) -> Model {
        let ind_buf: Vec<u32> = inds.iter().map(|v| *v as u32).collect();
        let ind_buf = IndexBuffer::new(dis, PrimitiveType::TrianglesList, &ind_buf).unwrap();

        Model {
            vbo: glium::VertexBuffer::new(dis, &verts).unwrap(),
            ind_buf,
            verts,
            inds,

            render: true,

            pos: Vec3::splat(0.0),
            rot: Vec3::splat(0.0),
            scale: 1.0,
        }
    }

    pub fn triangles(&self) -> Triangles {
        Triangles {
            verts: &self.verts,
            inds: &self.inds,
            cur: 0,
        }
    }
}

pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    pub fn inside(&self, pos: &Vec3) -> bool {
        pos.x >= self.min.x
            && pos.x <= self.max.x
            && pos.y >= self.min.y
            && pos.x <= self.max.y
            && pos.z >= self.min.z
            && pos.x <= self.max.z
    }
}

pub struct Triangle<'a> {
    v1: &'a Vertex,
    v2: &'a Vertex,
    v3: &'a Vertex,
}

impl<'a> Triangle<'a> {
    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            min: Vec3::new(
                self.v1.pos[0].min(self.v2.pos[0].min(self.v3.pos[0])),
                self.v1.pos[1].min(self.v2.pos[1].min(self.v3.pos[1])),
                self.v1.pos[2].min(self.v2.pos[2].min(self.v3.pos[2])),
            ),
            max: Vec3::new(
                self.v1.pos[0].max(self.v2.pos[0].max(self.v3.pos[0])),
                self.v1.pos[1].max(self.v2.pos[1].max(self.v3.pos[1])),
                self.v1.pos[2].max(self.v2.pos[2].max(self.v3.pos[2])),
            ),
        }
    }
}

/// Iterator over each of the triangles making up a model
pub struct Triangles<'a> {
    verts: &'a [Vertex],
    inds: &'a [usize],
    cur: usize,
}

impl<'a> Iterator for Triangles<'a> {
    type Item = Triangle<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ind1 = self.inds.get(self.cur)?;
        let ind2 = self.inds.get(self.cur + 1)?;
        let ind3 = self.inds.get(self.cur + 2)?;
        self.cur += 3;

        let v1 = self.verts.get(*ind1)?;
        let v2 = self.verts.get(*ind2)?;
        let v3 = self.verts.get(*ind3)?;

        Some(Triangle { v1, v2, v3 })
    }
}
