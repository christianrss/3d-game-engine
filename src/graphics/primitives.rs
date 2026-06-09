//! Geração de meshes 3D na CPU (cubo, esfera, plano, cilindro).
//!
//! Todos os backends recebem os mesmos vértices — a diferença está
//! apenas em *como* enviamos esses dados à GPU.

use crate::graphics::types::{Color, Mesh, Vertex};
use crate::math::Vec3;

/// Plano horizontal (chão do deserto).
pub fn plane(size: f32, color: Color) -> Mesh {
    let h = size / 2.0;
    let y = 0.0;
    let normal = [0.0, 1.0, 0.0];

    let vertices = vec![
        Vertex::new([-h, y, -h], color, normal),
        Vertex::new([h, y, -h], color, normal),
        Vertex::new([h, y, h], color, normal),
        Vertex::new([-h, y, h], color, normal),
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];

    Mesh { vertices, indices }
}

/// Cubo unitário centrado na origem.
pub fn cube(color: Color) -> Mesh {
    let p = [
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ];

    // 6 faces × 2 triângulos × 3 vértices
    let faces: [([usize; 4], [f32; 3]); 6] = [
        ([0, 1, 2, 3], [0.0, 0.0, -1.0]), // frente
        ([5, 4, 7, 6], [0.0, 0.0, 1.0]),  // trás
        ([4, 0, 3, 7], [-1.0, 0.0, 0.0]), // esquerda
        ([1, 5, 6, 2], [1.0, 0.0, 0.0]),  // direita
        ([3, 2, 6, 7], [0.0, 1.0, 0.0]),  // topo
        ([4, 5, 1, 0], [0.0, -1.0, 0.0]), // base
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (face_verts, normal) in &faces {
        let base = vertices.len() as u32;
        for &vi in face_verts {
            vertices.push(Vertex::new(p[vi], color, *normal));
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    Mesh { vertices, indices }
}

/// Esfera UV (aproximação por triângulos).
pub fn sphere(radius: f32, color: Color, segments: u32, rings: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let phi = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = phi.cos() * radius;
        let ring_radius = phi.sin() * radius;

        for seg in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * seg as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();
            let pos = [x, y, z];
            let normal = Vec3::from_array(pos).normalize().to_array();
            vertices.push(Vertex::new(pos, color, normal));
        }
    }

    for ring in 0..rings {
        for seg in 0..segments {
            let a = ring * (segments + 1) + seg;
            let b = a + segments + 1;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }

    Mesh { vertices, indices }
}

/// Cilindro vertical (cactos, pedestais).
pub fn cylinder(radius: f32, height: f32, color: Color, segments: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let half_h = height / 2.0;

    // Lateral
    for i in 0..=segments {
        let theta = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let x = radius * theta.cos();
        let z = radius * theta.sin();
        let normal = [x / radius, 0.0, z / radius];

        vertices.push(Vertex::new([x, -half_h, z], color, normal));
        vertices.push(Vertex::new([x, half_h, z], color, normal));
    }

    for i in 0..segments {
        let a = i * 2;
        indices.extend_from_slice(&[a, a + 1, a + 2, a + 2, a + 1, a + 3]);
    }

    // Tampa superior
    let top_center = vertices.len() as u32;
    vertices.push(Vertex::new([0.0, half_h, 0.0], color, [0.0, 1.0, 0.0]));
    let top_ring = vertices.len() as u32;
    for i in 0..=segments {
        let theta = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        vertices.push(Vertex::new(
            [radius * theta.cos(), half_h, radius * theta.sin()],
            color,
            [0.0, 1.0, 0.0],
        ));
    }
    for i in 0..segments {
        indices.extend_from_slice(&[top_center, top_ring + i, top_ring + i + 1]);
    }

    Mesh { vertices, indices }
}
