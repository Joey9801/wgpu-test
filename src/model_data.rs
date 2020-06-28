use std::path::Path;
use tokio::fs::File;
use tokio::prelude::*;

use super::Vertex;

/// Represents the data for a single model on the CPU
pub struct ModelData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub texture: image::RgbaImage,
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

        let (doc, buffers, images) =
            gltf::import_slice(&file_content).map_err(|_| "Failed to parse GLTF file")?;

        if doc.meshes().len() < 1 {
            return Err("Expected a GLTF file with at least one mesh");
        } else if doc.meshes().len() > 1 {
            println!("WARN: GLTF file has multiple meshes, only loading the first")
        }
        let mesh = doc.meshes().next().unwrap();

        if mesh.primitives().len() < 1 {
            return Err("Expected a GLTF mesh with at least one primitive");
        } else if mesh.primitives().len() > 1 {
            println!("WARN: mesh has multiple primitives, only loading the first")
        }
        let primitive = mesh.primitives().next().unwrap();

        let reader = primitive.reader(|buff| Some(&buffers[buff.index()]));
        let position_iter = reader
            .read_positions()
            .ok_or("Mesh vertices have no position data")?;
        let normal_iter = reader
            .read_normals()
            .ok_or("Mesh vertices have no normal data")?;
        let texcoord_iter = reader
            .read_tex_coords(0)
            .ok_or("Mesh vertices have no texcoord data")?
            .into_f32();

        let mut vertices = Vec::new();
        for ((position, normal), texcoord) in position_iter.zip(normal_iter).zip(texcoord_iter) {
            vertices.push(Vertex {
                position,
                normal,
                texcoord,
                color: [0.5, 0.5, 0.5, 1.0],
            })
        }

        let indices = reader
            .read_indices()
            .ok_or("Mesh doesn't have vertex index data")?
            .into_u32()
            .collect();

        let pbr_material = primitive.material().pbr_metallic_roughness();
        let base_color_texture = match pbr_material.base_color_texture() {
            Some(texture_info) => &images[texture_info.texture().index()],
            None => return Err("Primitive material doesn't have a pbr base color"),
        };
        let base_color_texture = match base_color_texture.format {
            gltf::image::Format::R8G8B8 => {
                let rgb = image::RgbImage::from_raw(
                    base_color_texture.width,
                    base_color_texture.height,
                    base_color_texture.pixels.clone(),
                )
                .ok_or("GLTF texture didn't have sufficient pixel data to fill its width*height")?;

                image::DynamicImage::ImageRgb8(rgb).into_rgba()
            }
            gltf::image::Format::R8G8B8A8 => image::RgbaImage::from_raw(
                base_color_texture.width,
                base_color_texture.height,
                base_color_texture.pixels.clone(),
            )
            .ok_or("GLTF texture didn't have sufficient pixel data to fill its width*height")?,
            _ => return Err("Primitive base color texture has an unsupported pixel format"),
        };

        Ok(Self {
            vertices,
            indices,
            texture: base_color_texture,
        })
    }
}
