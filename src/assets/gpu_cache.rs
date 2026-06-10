//! Upload de assets para a GPU no startup.

use crate::assets::loader::load_texture;
use crate::assets::AssetLibrary;
use crate::graphics::renderer::{GfxRenderer, RendererError};
use crate::graphics::{GpuMesh, GpuTexture};
use std::collections::HashMap;
use std::path::Path;

pub struct GpuAssetCache {
    pub meshes: HashMap<String, GpuMesh>,
    pub viewmodel: GpuMesh,
    pub fps_arm: GpuMesh,
    pub terrain: GpuMesh,
    pub sand_albedo: GpuTexture,
    pub sand_normal: GpuTexture,
    pub sand_rough: GpuTexture,
    pub sand_ao: GpuTexture,
    pub rock_albedo: GpuTexture,
    pub rock_normal: GpuTexture,
    pub rock_rough: GpuTexture,
}

impl GpuAssetCache {
    pub fn from_library(
        lib: &AssetLibrary,
        renderer: &mut GfxRenderer,
    ) -> Result<Self, RendererError> {
        let mut meshes = HashMap::new();
        for (id, model) in &lib.models {
            let mut gpu = renderer.upload_mesh(&model.mesh)?;
            if let Some(ref tex_path) = model.texture_path {
                if let Ok(tex_data) = load_texture(Path::new(tex_path)) {
                    if let Ok(tex) = renderer.upload_texture(&tex_data) {
                        gpu.albedo_tex = Some(tex.gpu_id);
                    }
                }
            }
            meshes.insert(id.clone(), gpu);
        }
        let viewmodel = renderer.upload_mesh(&lib.viewmodel.mesh)?;
        let fps_arm = meshes
            .get("fps_arm")
            .cloned()
            .ok_or_else(|| RendererError::Assets("fps_arm mesh missing".into()))?;
        let terrain = renderer.upload_mesh(&lib.terrain.mesh)?;
        let sand_albedo = renderer.upload_texture(&lib.sand_albedo)?;
        let sand_normal = renderer.upload_texture(&lib.sand_normal)?;
        let sand_rough = renderer.upload_texture(&lib.sand_rough)?;
        let sand_ao = renderer.upload_texture(&lib.sand_ao)?;
        let rock_albedo = renderer.upload_texture(&lib.rock_albedo)?;
        let rock_normal = renderer.upload_texture(&lib.rock_normal)?;
        let rock_rough = renderer.upload_texture(&lib.rock_rough)?;

        renderer.set_terrain_textures(&sand_albedo, &sand_normal, &sand_rough, &sand_ao);
        renderer.set_rock_textures(&rock_albedo, &rock_normal, &rock_rough);

        Ok(Self {
            meshes,
            viewmodel,
            fps_arm,
            terrain,
            sand_albedo,
            sand_normal,
            sand_rough,
            sand_ao,
            rock_albedo,
            rock_normal,
            rock_rough,
        })
    }

    pub fn mesh(&self, id: &str) -> Option<&GpuMesh> {
        self.meshes.get(id)
    }
}
