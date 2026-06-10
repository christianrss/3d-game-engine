//! [`GfxRenderer`] — fachada unificada sobre os 3 backends.

use crate::graphics::backend::{BackendKind, GfxBackend};
use crate::graphics::{Camera, Color, DrawMaterial, GpuMesh, GpuTexture, Mesh, TextureData};
use crate::math::Mat4;
use std::collections::HashMap;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

#[derive(Debug)]
pub enum RendererError {
    #[cfg(feature = "opengl")]
    OpenGL(crate::graphics::opengl::OpenGLError),
    #[cfg(feature = "vulkan")]
    Vulkan(String),
    #[cfg(all(feature = "directx", target_os = "windows"))]
    DirectX(String),
    Unsupported(BackendKind),
    Mesh(String),
    Assets(String),
}

impl std::fmt::Display for RendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub enum GfxRenderer {
    #[cfg(feature = "opengl")]
    OpenGL(crate::graphics::opengl::OpenGLRenderer),
    #[cfg(feature = "vulkan")]
    Vulkan(crate::graphics::vulkan::VulkanRenderer),
    #[cfg(all(feature = "directx", target_os = "windows"))]
    DirectX11(crate::graphics::directx::DirectX11Renderer),
}

pub struct EngineWindow {
    pub window: Window,
    pub renderer: GfxRenderer,
    pub backend: BackendKind,
}

impl GfxRenderer {
    pub fn create(
        event_loop: &ActiveEventLoop,
        title: &str,
        width: u32,
        height: u32,
        backend: BackendKind,
    ) -> Result<EngineWindow, RendererError> {
        match backend {
            #[cfg(feature = "opengl")]
            BackendKind::OpenGL => {
                let (window, gl_ctx) = crate::graphics::opengl::GlContext::new(
                    event_loop, title, width, height,
                )
                .map_err(|e| {
                    RendererError::OpenGL(crate::graphics::opengl::OpenGLError::Context(e))
                })?;

                let size = window.inner_size();
                let renderer = crate::graphics::opengl::OpenGLRenderer::from_context(
                    gl_ctx,
                    size.width,
                    size.height,
                )
                .map_err(RendererError::OpenGL)?;

                Ok(EngineWindow {
                    window,
                    renderer: GfxRenderer::OpenGL(renderer),
                    backend,
                })
            }

            #[cfg(feature = "vulkan")]
            BackendKind::Vulkan => {
                let window = event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title(title)
                            .with_inner_size(winit::dpi::LogicalSize::new(
                                width as f64,
                                height as f64,
                            )),
                    )
                    .map_err(|e| RendererError::Vulkan(e.to_string()))?;

                let renderer = crate::graphics::vulkan::VulkanRenderer::new(&window)
                    .map_err(RendererError::Vulkan)?;

                Ok(EngineWindow {
                    window,
                    renderer: GfxRenderer::Vulkan(renderer),
                    backend,
                })
            }

            #[cfg(all(feature = "directx", target_os = "windows"))]
            BackendKind::DirectX11 => {
                let window = event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title(title)
                            .with_inner_size(winit::dpi::LogicalSize::new(
                                width as f64,
                                height as f64,
                            )),
                    )
                    .map_err(|e| RendererError::DirectX(e.to_string()))?;

                let renderer = crate::graphics::directx::DirectX11Renderer::new(&window)
                    .map_err(RendererError::DirectX)?;

                Ok(EngineWindow {
                    window,
                    renderer: GfxRenderer::DirectX11(renderer),
                    backend,
                })
            }

            other => Err(RendererError::Unsupported(other)),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.resize(width, height),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.resize(width, height),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.resize(width, height),
        }
    }

    pub fn upload_mesh(&mut self, mesh: &Mesh) -> Result<GpuMesh, RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.upload_mesh(mesh).map_err(RendererError::OpenGL),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.upload_mesh(mesh).map_err(RendererError::Mesh),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.upload_mesh(mesh).map_err(RendererError::Mesh),
        }
    }

    pub fn upload_texture(&mut self, data: &TextureData) -> Result<GpuTexture, RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.upload_texture(data).map_err(RendererError::OpenGL),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.upload_texture(data).map_err(RendererError::Mesh),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.upload_texture(data).map_err(RendererError::Mesh),
        }
    }

    pub fn set_terrain_textures(
        &mut self,
        albedo: &GpuTexture,
        normal: &GpuTexture,
        rough: &GpuTexture,
        ao: &GpuTexture,
    ) {
        #[cfg(feature = "opengl")]
        if let GfxRenderer::OpenGL(r) = self {
            r.set_terrain_textures(albedo, normal, rough, ao);
        }
        let _ = (albedo, normal, rough, ao);
    }

    pub fn begin_frame(&mut self, clear: Color) {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.begin_frame(clear),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.begin_frame(clear),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.begin_frame(clear),
        }
    }

    pub fn begin_shadow_pass(&mut self, camera: &Camera) -> Result<(), RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.begin_shadow_pass(camera).map_err(RendererError::OpenGL),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.begin_shadow_pass(camera).map_err(RendererError::Mesh),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.begin_shadow_pass(camera).map_err(RendererError::Mesh),
        }
    }

    pub fn draw_shadow(&mut self, gpu_mesh: &GpuMesh, model: Mat4) -> Result<(), RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.draw_shadow(gpu_mesh, model).map_err(RendererError::OpenGL),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.draw_shadow(gpu_mesh, model).map_err(RendererError::Mesh),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.draw_shadow(gpu_mesh, model).map_err(RendererError::Mesh),
        }
    }

    pub fn end_shadow_pass(&mut self) -> Result<(), RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.end_shadow_pass().map_err(RendererError::OpenGL),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.end_shadow_pass().map_err(RendererError::Mesh),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.end_shadow_pass().map_err(RendererError::Mesh),
        }
    }

    pub fn begin_scene_pass(&mut self, clear: Color) -> Result<(), RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.begin_scene_pass(clear).map_err(RendererError::OpenGL),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.begin_scene_pass(clear).map_err(RendererError::Mesh),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.begin_scene_pass(clear).map_err(RendererError::Mesh),
        }
    }

    pub fn end_scene_pass(&mut self) -> Result<(), RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.end_scene_pass().map_err(RendererError::OpenGL),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.end_scene_pass().map_err(RendererError::Mesh),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.end_scene_pass().map_err(RendererError::Mesh),
        }
    }

    pub fn draw_sky(&mut self, camera: &Camera) -> Result<(), RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.draw_sky(camera).map_err(RendererError::OpenGL),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.draw_sky(camera).map_err(RendererError::Mesh),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.draw_sky(camera).map_err(RendererError::Mesh),
        }
    }

    pub fn draw(
        &mut self,
        gpu_mesh: &GpuMesh,
        model: Mat4,
        camera: &Camera,
        material: DrawMaterial,
    ) -> Result<(), RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => {
                r.draw(gpu_mesh, model, camera, material).map_err(RendererError::OpenGL)
            }
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => {
                r.draw(gpu_mesh, model, camera, material).map_err(RendererError::Mesh)
            }
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => {
                r.draw(gpu_mesh, model, camera, material).map_err(RendererError::Mesh)
            }
        }
    }

    pub fn draw_hud(&mut self, hud: &HudState) {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.draw_hud(hud),
            _ => {}
        }
    }

    pub fn draw_viewmodel(&mut self, camera: &Camera, gun_mesh: &GpuMesh, local: Mat4) {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.draw_viewmodel(camera, gun_mesh, local),
            _ => {}
        }
    }

    pub fn draw_particles(
        &mut self,
        camera: &Camera,
        particles: &[crate::graphics::ParticleDraw],
        vm_transform: Mat4,
    ) {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.draw_particles(camera, particles, vm_transform),
            _ => {}
        }
    }

    pub fn draw_world_particles(
        &mut self,
        camera: &Camera,
        particles: &[crate::graphics::ParticleDraw],
    ) {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.draw_world_particles(camera, particles),
            _ => {}
        }
    }

    pub fn draw_line_strip(&mut self, camera: &Camera, points: &[[f32; 3]], color: [f32; 4]) {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.draw_line_strip(camera, points, color),
            _ => {}
        }
    }

    pub fn end_frame(&mut self) -> Result<(), RendererError> {
        match self {
            #[cfg(feature = "opengl")]
            GfxRenderer::OpenGL(r) => r.end_frame().map_err(RendererError::OpenGL),
            #[cfg(feature = "vulkan")]
            GfxRenderer::Vulkan(r) => r.end_frame().map_err(RendererError::Mesh),
            #[cfg(all(feature = "directx", target_os = "windows"))]
            GfxRenderer::DirectX11(r) => r.end_frame().map_err(RendererError::Mesh),
        }
    }
}

#[derive(Default)]
pub struct MeshCache {
    meshes: HashMap<String, GpuMesh>,
}

impl MeshCache {
    pub fn get_or_upload(
        &mut self,
        renderer: &mut GfxRenderer,
        key: &str,
        mesh: &Mesh,
    ) -> Result<&GpuMesh, RendererError> {
        if !self.meshes.contains_key(key) {
            let gpu = renderer.upload_mesh(mesh)?;
            self.meshes.insert(key.to_string(), gpu);
        }
        Ok(self.meshes.get(key).unwrap())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HudState {
    pub show_crosshair: bool,
    pub muzzle_flash: f32,
    pub hit_flash: f32,
    /// Abertura da mira (0 = fechada, 1 = máxima dispersão).
    pub crosshair_spread: f32,
}
