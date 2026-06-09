//! Trait [`GfxBackend`] — interface comum aos 3 backends de renderização.
//!
//! Cada backend (OpenGL, Vulkan, DirectX) implementa este trait.
//! O jogo nunca chama APIs gráficas diretamente — só usa o trait.

use crate::graphics::{Camera, Color, GpuMesh, Mesh};
use crate::math::Mat4;
use std::fmt::Debug;

/// Qual backend gráfico usar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    OpenGL,
    Vulkan,
    DirectX11,
}

impl BackendKind {
    /// Lê a variável de ambiente `GFX_BACKEND` ou usa OpenGL como padrão.
    pub fn from_env() -> Self {
        match std::env::var("GFX_BACKEND")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "vulkan" | "vk" => BackendKind::Vulkan,
            "directx" | "dx11" | "d3d11" => BackendKind::DirectX11,
            _ => BackendKind::OpenGL,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BackendKind::OpenGL => "OpenGL 3.3",
            BackendKind::Vulkan => "Vulkan 1.2",
            BackendKind::DirectX11 => "Direct3D 11",
        }
    }
}

/// Interface que todo backend de renderização deve implementar.
///
/// ## Ciclo de um frame
///
/// ```text
/// begin_frame(cor) → draw(mesh, mvp) × N → end_frame()
/// ```
pub trait GfxBackend {
    type Error: Debug;

    /// Chamado quando a janela muda de tamanho.
    fn resize(&mut self, width: u32, height: u32);

    /// Envia uma mesh da CPU para a memória da GPU.
    fn upload_mesh(&mut self, mesh: &Mesh) -> Result<GpuMesh, Self::Error>;

    /// Inicia um novo frame (limpa o framebuffer).
    fn begin_frame(&mut self, clear: Color);

    /// Desenha uma mesh com a matriz MVP (Model-View-Projection).
    fn draw(&mut self, gpu_mesh: &GpuMesh, model: Mat4, camera: &Camera) -> Result<(), Self::Error>;

    /// Finaliza o frame e apresenta na tela (swap buffers / present).
    fn end_frame(&mut self) -> Result<(), Self::Error>;
}
