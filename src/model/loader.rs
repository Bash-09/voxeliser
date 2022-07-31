use std::{fs::File, io::BufReader, path::PathBuf};

use obj::{load_obj, TexturedVertex};

use super::vertex::Vertex;

pub fn load_model(file: PathBuf) -> Result<(Vec<Vertex>, Vec<usize>), String> {
    match file.extension().unwrap().to_str().unwrap() {
        "gltf" | "glb" => match easy_gltf::load(&file) {
            Ok(scenes) => {
                let model = &scenes[0].models[0];

                if let Some(inds) = model.indices() {
                    let verts = model.vertices().clone();
                    let inds = inds.clone();

                    let mut new_verts = Vec::new();
                    for v in verts {
                        new_verts.push(Vertex::from_gltf(&v));
                    }

                    Ok((new_verts, inds))
                } else {
                    Err(String::from(
                        "Only models with vertex indexing are supported rn",
                    ))
                }
            }
            Err(e) => Err(format!("Failed to read file - {}", e)),
        },
        "obj" => {
            let input = BufReader::new(File::open(&file).unwrap());
            match load_obj::<TexturedVertex, BufReader<File>, usize>(input) {
                Ok(obj) => {
                    let inds = obj.indices;
                    let verts: Vec<Vertex> = obj
                        .vertices
                        .iter()
                        .map(|v| Vertex::new(v.position, v.normal))
                        .collect();

                    Ok((verts, inds))
                }
                Err(e) => Err(format!("Failed to load file - {}", e)),
            }
        }
        _ => Err(String::from("Unexpected file format received")),
    }
}
