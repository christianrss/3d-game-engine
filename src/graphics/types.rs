//! Tipos compartilhados entre todos os backends gráficos.



use bytemuck::{Pod, Zeroable};



/// Cor RGBA normalizada (0.0 – 1.0).

#[derive(Debug, Clone, Copy, PartialEq)]

pub struct Color {

    pub r: f32,

    pub g: f32,

    pub b: f32,

    pub a: f32,

}



impl Color {

    pub const SAND: Color = Color::rgb(0.88, 0.76, 0.48);

    pub const SAND_DARK: Color = Color::rgb(0.72, 0.58, 0.35);

    pub const SKY: Color = Color::rgb(0.55, 0.78, 0.95);

    pub const SKY_HORIZON: Color = Color::rgb(0.92, 0.72, 0.48);

    pub const TARGET_RED: Color = Color::rgb(0.95, 0.12, 0.08);

    pub const TARGET_WHITE: Color = Color::rgb(0.98, 0.98, 0.95);

    pub const TARGET_GOLD: Color = Color::rgb(1.0, 0.82, 0.15);

    pub const CACTUS: Color = Color::rgb(0.15, 0.52, 0.22);

    pub const CACTUS_DARK: Color = Color::rgb(0.1, 0.38, 0.15);

    pub const ROCK: Color = Color::rgb(0.48, 0.42, 0.36);

    pub const DUNE: Color = Color::rgb(0.82, 0.65, 0.38);

    pub const PEDESTAL: Color = Color::rgb(0.45, 0.28, 0.15);

    pub const GUN_METAL: Color = Color::rgb(0.25, 0.25, 0.28);

    pub const GUN_WOOD: Color = Color::rgb(0.4, 0.25, 0.12);

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

/// location 1 → normal   (vec3)

/// location 2 → UV       (vec2)

/// location 3 → cor      (vec3)

/// ```

#[repr(C)]

#[derive(Debug, Clone, Copy, Pod, Zeroable)]

pub struct Vertex {

    pub position: [f32; 3],

    pub normal: [f32; 3],

    pub uv: [f32; 2],

    pub color: [f32; 3],

}



impl Vertex {

    pub fn new(pos: [f32; 3], normal: [f32; 3], uv: [f32; 2], color: Color) -> Self {

        Self {

            position: pos,

            normal,

            uv,

            color: [color.r, color.g, color.b],

        }

    }



    pub fn colored(pos: [f32; 3], color: Color, normal: [f32; 3]) -> Self {

        Self::new(pos, normal, [0.0, 0.0], color)

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

    pub gpu_id: u64,

    /// Albedo embutido do glTF (fotogrametria), quando disponível.
    pub albedo_tex: Option<u64>,

}



/// Textura na GPU.

#[derive(Debug, Clone)]

pub struct GpuTexture {

    pub gpu_id: u64,

    pub width: u32,

    pub height: u32,

}



/// Material de desenho — cor de vértice ou terreno texturizado.

#[derive(Debug, Clone, Copy)]

pub enum DrawMaterial {

    /// PBR padrão — roughness/metallic por objeto
    Standard { roughness: f32, metallic: f32 },

    /// Terreno com mapas PBR completos + ripples de areia
    Terrain { tiling: f32 },

    /// Rochas com textura triplanar PBR (Poly Haven)
    Rock { tiling: f32 },

    /// Lago/oásis — renderizado com shader de água
    Water,

}

impl DrawMaterial {

    pub fn rock() -> Self {
        Self::Rock { tiling: 1.8 }
    }

    pub fn wood() -> Self {
        Self::Standard { roughness: 0.75, metallic: 0.0 }
    }

    pub fn metal() -> Self {
        Self::Standard { roughness: 0.35, metallic: 0.85 }
    }

    pub fn foliage() -> Self {
        Self::Standard { roughness: 0.9, metallic: 0.0 }
    }

}



/// Dados de textura na CPU para upload à GPU.

#[derive(Debug, Clone)]

pub struct TextureData {

    pub width: u32,

    pub height: u32,

    pub pixels: Vec<u8>,

}



/// Partícula para renderização (viewmodel space).

#[derive(Clone, Copy)]

pub struct ParticleDraw {

    pub pos: [f32; 3],

    pub size: f32,

    pub alpha: f32,

    pub kind: f32,

}


