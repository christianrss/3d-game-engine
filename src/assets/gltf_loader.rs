//! Carrega modelos glTF / GLB (CC0) para o pipeline de mesh da engine.

use crate::assets::loader::ModelAsset;
use crate::graphics::{Color, Mesh, Vertex};
use crate::math::{Mat4, Quat, Vec3};
use gltf::mesh::util::ReadIndices;
use std::path::{Path, PathBuf};

/// Resultado do carregamento com ponto do cano para partículas.
#[derive(Debug, Clone)]
pub struct LoadedGltf {
    pub mesh: Mesh,
    pub texture_path: Option<String>,
    pub muzzle_local: Vec3,
}

/// Carrega glTF/GLB e normaliza para viewmodel FPS (eixo -Z = frente).
pub fn load_gltf_viewmodel(path: impl AsRef<Path>, _name: &str) -> Result<LoadedGltf, String> {
    let path = path.as_ref();
    let (document, buffers, images) =
        gltf::import(path).map_err(|e| format!("glTF {}: {e}", path.display()))?;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut texture_path = None;

    for scene in document.scenes() {
        for node in scene.nodes() {
            visit_node(
                &node,
                Mat4::IDENTITY,
                &buffers,
                &images,
                path,
                &mut vertices,
                &mut indices,
                &mut texture_path,
            );
        }
    }

    if vertices.is_empty() {
        return Err(format!("glTF {}: sem geometria", path.display()));
    }

    let mesh = Mesh { vertices, indices };
    let (mesh, muzzle_local) = fit_viewmodel_mesh(mesh, 0.58);
    let place = Vec3::new(0.1, -0.14, -0.32);
    let mesh = place_viewmodel(mesh, place);
    let muzzle_local = offset_muzzle(muzzle_local, place);

    log::info!(
        "glTF viewmodel '{}' — {} verts, cano em ({:.2}, {:.2}, {:.2})",
        path.display(),
        mesh.vertices.len(),
        muzzle_local.x,
        muzzle_local.y,
        muzzle_local.z
    );

    Ok(LoadedGltf {
        mesh,
        texture_path,
        muzzle_local,
    })
}

/// Carrega scan fotogramétrico / prop — assenta no chão (Y=0) e escala uniforme.
pub fn load_gltf_prop(path: impl AsRef<Path>, target_size: f32) -> Result<ModelAsset, String> {
    let path = path.as_ref();
    let (mesh, texture_path) = load_gltf_raw(path)?;
    let (min, max) = mesh_bounds(&mesh);
    let extent = (max - min).max_element().max(0.001);
    let scale = target_size / extent;
    let center = (min + max) * 0.5;
    let mesh = transform_mesh(
        mesh,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::IDENTITY,
            Vec3::new(-center.x * scale, -min.y * scale, -center.z * scale),
        ),
    );
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "prop".into());
    log::info!(
        "glTF prop '{}' — {} vértices (fotogrametria)",
        path.display(),
        mesh.vertices.len()
    );
    Ok(ModelAsset {
        name,
        mesh,
        texture_path,
        tiling: 1.0,
    })
}

fn load_gltf_raw(path: &Path) -> Result<(Mesh, Option<String>), String> {
    let (document, buffers, images) =
        gltf::import(path).map_err(|e| format!("glTF {}: {e}", path.display()))?;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut texture_path = None;
    for scene in document.scenes() {
        for node in scene.nodes() {
            visit_node(
                &node,
                Mat4::IDENTITY,
                &buffers,
                &images,
                path,
                &mut vertices,
                &mut indices,
                &mut texture_path,
            );
        }
    }
    if vertices.is_empty() {
        return Err(format!("glTF {}: sem geometria", path.display()));
    }
    Ok((Mesh { vertices, indices }, texture_path))
}

fn visit_node(
    node: &gltf::Node,
    parent: Mat4,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    base_path: &Path,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    texture_path: &mut Option<String>,
) {
    let local = parent * node_transform(node);

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            read_primitive(
                &primitive,
                local,
                buffers,
                images,
                base_path,
                vertices,
                indices,
                texture_path,
            );
        }
    }

    for child in node.children() {
        visit_node(
            &child,
            local,
            buffers,
            images,
            base_path,
            vertices,
            indices,
            texture_path,
        );
    }
}

fn node_transform(node: &gltf::Node) -> Mat4 {
    let (translation, rotation, scale) = node.transform().decomposed();
    Mat4::from_scale_rotation_translation(
        Vec3::from_array(scale),
        Quat::from_array(rotation),
        Vec3::from_array(translation),
    )
}

fn read_primitive(
    primitive: &gltf::Primitive,
    transform: Mat4,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    base_path: &Path,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    texture_path: &mut Option<String>,
) {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .map(|iter| iter.collect())
        .unwrap_or_default();
    if positions.is_empty() {
        return;
    }

    let normals: Vec<[f32; 3]> = reader
        .read_normals()
        .map(|iter| iter.collect())
        .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

    let uvs: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|iter| iter.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

    let color = material_color(primitive.material(), images, base_path, texture_path);
    let normal_mat = transform.inverse().transpose();

    let base = vertices.len() as u32;
    for i in 0..positions.len() {
        let p = transform.transform_point3(Vec3::from_array(positions[i]));
        let n = normal_mat
            .transform_vector3(Vec3::from_array(normals[i]))
            .normalize_or_zero();
        let n = if n.length_squared() > 0.001 {
            n
        } else {
            Vec3::Y
        };
        vertices.push(Vertex::new(
            p.to_array(),
            n.to_array(),
            uvs[i],
            color,
        ));
    }

    if let Some(idx_reader) = reader.read_indices() {
        match idx_reader {
            ReadIndices::U8(iter) => indices.extend(iter.map(|i| base + i as u32)),
            ReadIndices::U16(iter) => indices.extend(iter.map(|i| base + i as u32)),
            ReadIndices::U32(iter) => indices.extend(iter.map(|i| base + i)),
        }
    } else {
        indices.extend((0..positions.len() as u32).map(|i| base + i));
    }
}

fn material_color(
    material: MaterialRef<'_>,
    images: &[gltf::image::Data],
    base_path: &Path,
    texture_path: &mut Option<String>,
) -> Color {
    let pbr = material.pbr_metallic_roughness();
    let factor = pbr.base_color_factor();
    if texture_path.is_none() {
        if let Some(tex) = pbr.base_color_texture() {
            if let Some(path) = resolve_texture_path(tex.texture(), images, base_path) {
                *texture_path = Some(path);
            }
        }
    }
    Color::rgb(factor[0], factor[1], factor[2])
}

type MaterialRef<'a> = gltf::Material<'a>;

fn resolve_texture_path(
    texture: gltf::Texture,
    images: &[gltf::image::Data],
    base_path: &Path,
) -> Option<String> {
    let image = texture.source();
    match image.source() {
        gltf::image::Source::Uri { uri, .. } => {
            let p = base_path.parent()?.join(uri);
            if p.exists() {
                Some(p.to_string_lossy().to_string())
            } else {
                None
            }
        }
        gltf::image::Source::View { view, .. } => {
            let idx = view.index();
            let data = images.get(idx)?;
            let out = base_path
                .parent()
                .unwrap_or(base_path)
                .join(format!("gltf_embedded_{idx}.png"));
            if !out.exists() {
                let rgba = match data.format {
                    gltf::image::Format::R8G8B8A8 => data.pixels.clone(),
                    gltf::image::Format::R8G8B8 => {
                        let mut rgba = Vec::with_capacity(data.pixels.len() / 3 * 4);
                        for px in data.pixels.chunks_exact(3) {
                            rgba.extend_from_slice(px);
                            rgba.push(255);
                        }
                        rgba
                    }
                    _ => Vec::new(),
                };
                if !rgba.is_empty() {
                    if let Some(img) = image::RgbaImage::from_raw(data.width, data.height, rgba) {
                        let _ = img.save(&out);
                    }
                }
            }
            if out.exists() {
                Some(out.to_string_lossy().to_string())
            } else {
                None
            }
        }
    }
}

fn mesh_bounds(mesh: &Mesh) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for v in &mesh.vertices {
        let p = Vec3::from_array(v.position);
        min = min.min(p);
        max = max.max(p);
    }
    (min, max)
}

fn transform_mesh(mesh: Mesh, transform: Mat4) -> Mesh {
    let normal_mat = transform.inverse().transpose();
    let mut vertices = Vec::with_capacity(mesh.vertices.len());
    for v in mesh.vertices {
        let p = transform.transform_point3(Vec3::from_array(v.position));
        let n = normal_mat
            .transform_vector3(Vec3::from_array(v.normal))
            .normalize_or_zero();
        let n = if n.length_squared() > 0.001 {
            n
        } else {
            Vec3::Y
        };
        vertices.push(Vertex::new(p.to_array(), n.to_array(), v.uv, Color::rgb(v.color[0], v.color[1], v.color[2])));
    }
    Mesh {
        vertices,
        indices: mesh.indices,
    }
}

/// Converte Y-up (glTF) para viewmodel FPS e escala pelo comprimento.
fn fit_viewmodel_mesh(mesh: Mesh, target_length: f32) -> (Mesh, Vec3) {
    let orient = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2) * Quat::from_rotation_y(std::f32::consts::PI);
    let mesh = transform_mesh(mesh, Mat4::from_quat(orient));

    let (min, max) = mesh_bounds(&mesh);
    let size = max - min;
    let length = size.x.max(size.y).max(size.z).max(0.001);
    let scale = target_length / length;
    let center = (min + max) * 0.5;

    let fit = Mat4::from_scale_rotation_translation(
        Vec3::splat(scale),
        Quat::IDENTITY,
        -center * scale,
    );
    let mesh = transform_mesh(mesh, fit);

    let (min, max) = mesh_bounds(&mesh);
    let muzzle = Vec3::new(
        (min.x + max.x) * 0.5,
        (min.y + max.y) * 0.5,
        min.z,
    );
    (mesh, muzzle)
}

fn place_viewmodel(mesh: Mesh, offset: Vec3) -> Mesh {
    transform_mesh(mesh, Mat4::from_translation(offset))
}

fn scale_mesh(mesh: Mesh, scale: f32) -> Mesh {
    transform_mesh(mesh, Mat4::from_scale(Vec3::splat(scale)))
}

pub fn merge_meshes(parts: &[Mesh]) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for mesh in parts {
        let base = vertices.len() as u32;
        vertices.extend_from_slice(&mesh.vertices);
        indices.extend(mesh.indices.iter().map(|i| base + i));
    }
    Mesh { vertices, indices }
}

pub fn offset_muzzle(muzzle: Vec3, offset: Vec3) -> Vec3 {
    muzzle + offset
}

pub fn viewmodel_gun_candidates(root: &Path) -> Vec<PathBuf> {
    [
        root.join("gun/rifle.glb"),
        root.join("gun/gun/gun.glb"),
        root.join("gun/rifle.gltf"),
    ]
    .into_iter()
    .filter(|p| p.exists())
    .collect()
}
