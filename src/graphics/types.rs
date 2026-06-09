//! Tipos compartilhados entre todos os backends gráficos.

use bytemuck::{Pod, Zeroable};

/// Cor RGBA normalizada (0.0 – 1.0).
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const SAND: Color = Color::rgb(0.85, 0.72, 0.45);
    pub const SKY: Color = Color::rgb(0.95, 0.75, 0.5);
    pub const TARGET_RED: Color = Color::rgb(0.9, 0.15, 0.1);
    pub const CACTUS: Color = Color::rgb(0.2, 0.55, 0.25);
    pub const ROCK: Color = Color::rgb(0.5, 0.45, 0.4);
    pub const DUNE: Color = Color::rgb(0.78, 0.62, 0.38);
    pub const PEDESTAL: Color = Color::rgb(0.55, 0.35, 0.2);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Vértice enviado à GPU — layout idêntico nos 3 backends.
///
/// ```text
/// location 0 → posição (vec3)
/// location 1 → cor     (vec3)
/// location 2 → normal  (vec3)  — usada para iluminação difusa simples
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn new(pos: [f32; 3], color: Color, normal: [f32; 3]) -> Self {
        Self {
            position: pos,
            color: [color.r, color.g, color.b],
            normal,
        }
    }
}

/// Mesh na CPU — lista de vértices e índices.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

/// Mesh já carregada na GPU (handle opaco por backend).
#[derive(Debug, Clone)]
pub struct GpuMesh {
    pub vertex_count: u32,
    pub index_count: u32,
    /// ID interno usado pelo backend ativo
    pub gpu_id: u64,
}
