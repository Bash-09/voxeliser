use std::{sync::{Arc, mpsc::{self, Receiver, Sender}}, thread, time::Duration, slice::SliceIndex};

use glam::{EulerRot, Vec3, Mat4, Vec4Swizzles, IVec3};
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

pub fn generate_voxels(model: &Model, scale: f32) -> Receiver<(Vec<Vertex>, Vec<usize>)> {
    let (send_model, receive_model) = mpsc::channel::<(Vec<Vertex>, Vec<usize>)>();

    let model_data = Arc::new(get_model_data(model));
    thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<(usize, usize, usize, bool)>();

        let pool = ThreadPool::new(8);

        let mut min = model_data.verts[0].pos;
        for v in &model_data.verts {
            min[0] = min[0].min(v.pos[0]);
            min[1] = min[1].min(v.pos[1]);
            min[2] = min[2].min(v.pos[2]);
        }
        let min = min;

        let mut max = model_data.verts[0].pos;
        for v in &model_data.verts {
            max[0] = max[0].max(v.pos[0]);
            max[1] = max[1].max(v.pos[1]);
            max[2] = max[2].max(v.pos[2]);
        }
        let max = max;

        let res_x = ((max[0] - min[0]) / scale).ceil() as usize;
        let res_y = ((max[1] - min[1]) / scale).ceil() as usize;
        let res_z = ((max[2] - min[2]) / scale).ceil() as usize;

        for x in 0..res_x {
            for y in 0..res_y {
                for z in 0..res_z {
                    let tx = tx.clone();
                    let model_data = model_data.clone();

                    pool.execute(move || {
                        generate_voxel(tx, model_data, min.into(), x, y, z, scale);
                    });
                }
            }
        }

        // Consolidate voxels into a grid
        let mut voxels = vec![vec![vec![false; res_z]; res_y]; res_x];

        for _ in 0..(res_x * res_y * res_z) {
            match rx.recv() {
                Ok((x, y, z, voxel)) => {
                    voxels[x][y][z] = voxel;
                },
                Err(e) => {
                    println!("Error receiving: {}", e);
                    break;
                },
            }
        }

        // Create verts from voxels
        let mut verts: Vec<Vertex> = Vec::new();
        for x in 0..res_x {
            for y in 0..res_y {
                for z in 0..res_z {
                    if !voxels[x][y][z] {continue}

                    let px = if x == res_x-1 {
                        false
                    } else {
                        voxels[x+1][y][z]
                    };
                    let py = if y == res_y-1 {
                        false
                    } else {
                        voxels[x][y+1][z]
                    };
                    let pz = if z == res_z-1 {
                        false
                    } else {
                        voxels[x][y][z+1]
                    };

                    let nx = if x == 0 {
                        false
                    } else {
                        voxels[x-1][y][z]
                    };
                    let ny = if y == 0 {
                        false
                    } else {
                        voxels[x][y-1][z]
                    };
                    let nz = if z == 0 {
                        false
                    } else {
                        voxels[x][y][z-1]
                    };

                    verts.append(&mut generate_block_mesh(Vec3::new(x as f32, y as f32, z as f32), scale, min.into(), px, py, pz, nx, ny, nz));
                }
            }
        }
        let inds: Vec<usize> = (0..verts.len()).collect();

        std::thread::sleep(Duration::from_millis(2000));

        send_model.send((verts, inds)).unwrap();
    });

    receive_model
}

fn generate_voxel(tx: Sender<(usize, usize, usize, bool)>, model: Arc<ModelData>, min: Vec3, x: usize, y: usize, z: usize, size: f32) {
    let mut overlap = false;
    // for i in 0..(model.inds.len() / 3) {
    //     let v1 = model.verts[model.inds[i * 3]];
    //     let v2 = model.verts[model.inds[i * 3 + 1]];
    //     let v3 = model.verts[model.inds[i * 3 + 2]];
        
    //     let t_min = Vec3::new(
    //         v1.pos[0].min(v2.pos[0].min(v3.pos[0])),
    //         v1.pos[1].min(v2.pos[1].min(v3.pos[1])),
    //         v1.pos[2].min(v2.pos[2].min(v3.pos[2]))
    //     );
    //     let t_max = Vec3::new(
    //         v1.pos[0].max(v2.pos[0].max(v3.pos[0])),
    //         v1.pos[1].max(v2.pos[1].max(v3.pos[1])),
    //         v1.pos[2].max(v2.pos[2].max(v3.pos[2]))
    //     );

    //     let v_min = min;
    //     let v_max = min + Vec3::splat(size);

    //     // if v1.pos[0] >= min.x && v1.pos[0] <= min.x + size
    //     // && v1.pos[1] >= min.y && v1.pos[1] <= min.y + size
    //     // && v1.pos[2] >= min.z && v1.pos[2] <= min.z + size {
    //     //     overlap = true;
    //     //     break;
    //     // }

    //     // if v_min.x <= t_max.x && v_max.x >= t_min.x
    //     //     && v_min.y <= t_max.y && v_max.y >= t_min.y
    //     //     && v_min.z <= t_max.z && v_max.z >= t_min.z {

    //     //     overlap = true;
    //     //     break;
    //     // }

    //     // if t_max.x > v_min.x && t_min.x < v_max.x &&
    //     // t_max.y > v_min.y && t_min.y < v_max.y &&
    //     // t_max.z > v_min.z && t_min.z < v_max.z {
    //     //     overlap = true;
    //     //     break;
    //     // }

    // }

    if x%2 == 0 && y%2 == 0 && z%2 == 0 {
        overlap = true;
    }

    if let Err(e) = tx.send((x, y, z, overlap)) {
        panic!("Failed to send voxel: {}", e);
    }
}

fn generate_block_mesh(
    pos: Vec3,
    size: f32,
    min: Vec3,
    px: bool,
    py: bool,
    pz: bool,
    nx: bool,
    ny: bool,
    nz: bool,
) -> Vec<Vertex> {
    let mut verts: Vec<Vertex> = Vec::new();

    // Positive y
    if !py {
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [0.0, 1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size],
            norm: [0.0, 1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size + size, min.z + pos.z * size],
            norm: [0.0, 1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [0.0, 1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size + size, min.z + pos.z * size],
            norm: [0.0, 1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [0.0, 1.0, 0.0],
        });
    }
    // Negative y
    if !ny {
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size, min.z + pos.z * size + size],
            norm: [0.0, -1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size + size],
            norm: [0.0, -1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [0.0, -1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size, min.z + pos.z * size + size],
            norm: [0.0, -1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [0.0, -1.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [0.0, -1.0, 0.0],
        });
    }
    // Negative z
    if !nz {
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size],
            norm: [0.0, 0.0, -1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [0.0, 0.0, -1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size + size, min.z + pos.z * size],
            norm: [0.0, 0.0, -1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size],
            norm: [0.0, 0.0, -1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [0.0, 0.0, -1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [0.0, 0.0, -1.0],
        });
    }
    // Positive x
    if !px {
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size],
            norm: [1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size, min.z + pos.z * size + size],
            norm: [1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [1.0, 0.0, 0.0],
        });
    }
    // Positive z
    if !pz {
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [0.0, 0.0, 1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [0.0, 0.0, 1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size + size],
            norm: [0.0, 0.0, 1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [0.0, 0.0, 1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size + size],
            norm: [0.0, 0.0, 1.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size + size, min.y + pos.y * size, min.z + pos.z * size + size],
            norm: [0.0, 0.0, 1.0],
        });
    }
    // Negative X
    if !nx {
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [-1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size + size, min.z + pos.z * size],
            norm: [-1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [-1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size + size, min.z + pos.z * size + size],
            norm: [-1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size],
            norm: [-1.0, 0.0, 0.0],
        });
        verts.push(Vertex {
            pos: [min.x + pos.x * size, min.y + pos.y * size, min.z + pos.z * size + size],
            norm: [-1.0, 0.0, 0.0],
        });
    }

    verts
}