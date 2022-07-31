use std::sync::{Arc, mpsc};

use glam::{EulerRot, Vec3, Mat4, Vec4Swizzles};
use threadpool::ThreadPool;

use crate::model::{Model, vertex::Vertex};

struct ModelData {
    pub verts: Vec<Vertex>,
    pub inds: Vec<usize>,
}

fn get_model_data(model: &Model) -> ModelData {
    let mut verts = Vec::new();
    let mut tmat: Mat4 = Mat4::from_translation(model.pos);
        tmat *= Mat4::from_scale(Vec3::splat(model.scale));
        tmat *= Mat4::from_euler(EulerRot::XYZ, model.rot.x, model.rot.y, model.rot.z);

    for v in &model.verts {
        let pos = tmat * Vec3::from_slice(&v.pos).extend(1.0);
        verts.push(Vertex::new(pos.xyz().into(), v.norm));
    }
    
    ModelData {
        verts,
        inds: model.inds.clone(),
    }
}

pub fn generate_voxels(model: &Model) {
    let model_data = Arc::new(get_model_data(model));
    let (tx, rx) = mpsc::channel::<u8>();

    let pool = ThreadPool::new(8);

    for i in 0..(model_data.inds.len()/3) {
        pool.execute(move || {
            todo!();
        });
    }
}