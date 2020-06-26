use std::path::Path;
use tokio::fs::File;
use tokio::prelude::*;

use super::Vertex;

/// Represents the data for a single model on the CPU
pub struct ModelData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl ModelData {
    // TODO: Proper error type
    /// Load a model from a GLTF file.
    ///
    /// The file must contain only a single mesh, made from a single primitive.
    pub async fn load_gltf<P: AsRef<Path>>(path: P) -> Result<Self, &'static str> {
        let path = path.as_ref();

        let mut file_content = Vec::new();
        {
            let mut file = File::open(path)
                .await
                .map_err(|_| "Failed to open model file")?;
                
            file.read_to_end(&mut file_content)
                .await
                .map_err(|_| "Failed to read model data")?;
        }

        let (doc, buffers, _images) = gltf::import_slice(&file_content)
            .map_err(|_| "Failed to parse GLTF file")?;

        if doc.meshes().len() != 1 {
            return Err("Expected a GLTF file with precisely one mesh")
        }
        let mesh = doc.meshes().next().unwrap();
        if mesh.primitives().len() != 1 {
            return Err("Expected a GLTF mesh with exactly one primitive");
        }
        let primitive = mesh.primitives().next().unwrap();

        let reader = primitive.reader(|buff| Some(&buffers[buff.index()]));
        let position_iter = reader.read_positions().ok_or("Mesh vertices have no position data")?;
        let normal_iter = reader.read_normals().ok_or("Mesh vertices have no normal data")?;
        let indices_iter = reader.read_indices().ok_or("Mesh doesn't have vertex index data")?.into_u32();

        let mut vertices = Vec::new();
        for (position, normal) in position_iter.zip(normal_iter) {
            vertices.push(Vertex {
                position,
                normal,
                color: [0.5, 0.5, 0.5, 1.0],
            })
        }
        let indices = indices_iter.collect();

        Ok(Self {
            vertices,
            indices,
        })
    }
}
