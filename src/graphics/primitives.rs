//! Geração de meshes 3D na CPU (cubo, esfera, plano, cilindro).

use crate::graphics::types::{Color, Mesh, Vertex};
use crate::math::Vec3;

/// Plano horizontal (chão do deserto).
pub fn plane(size: f32, color: Color) -> Mesh {
    let h = size / 2.0;
    let y = 0.0;
    let normal = [0.0, 1.0, 0.0];

    let vertices = vec![
        Vertex::new([-h, y, -h], normal, [0.0, 0.0], color),
        Vertex::new([h, y, -h], normal, [1.0, 0.0], color),
        Vertex::new([h, y, h], normal, [1.0, 1.0], color),
        Vertex::new([-h, y, h], normal, [0.0, 1.0], color),
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

    let faces: [([usize; 4], [f32; 3], [[f32; 2]; 4]); 6] = [
        ([0, 1, 2, 3], [0.0, 0.0, -1.0], [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
        ([5, 4, 7, 6], [0.0, 0.0, 1.0], [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
        ([4, 0, 3, 7], [-1.0, 0.0, 0.0], [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
        ([1, 5, 6, 2], [1.0, 0.0, 0.0], [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
        ([3, 2, 6, 7], [0.0, 1.0, 0.0], [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
        ([4, 5, 1, 0], [0.0, -1.0, 0.0], [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (face_verts, normal, uvs) in &faces {
        let base = vertices.len() as u32;
        for (i, &vi) in face_verts.iter().enumerate() {
            vertices.push(Vertex::new(p[vi], *normal, uvs[i], color));
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
            let u = seg as f32 / segments as f32;
            let v = ring as f32 / rings as f32;
            vertices.push(Vertex::new(pos, normal, [u, v], color));
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

/// Cilindro vertical (viewmodel, pedestais).
pub fn cylinder(radius: f32, height: f32, color: Color, segments: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let half_h = height / 2.0;

    for i in 0..=segments {
        let theta = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let x = radius * theta.cos();
        let z = radius * theta.sin();
        let normal = [x / radius, 0.0, z / radius];
        let u = i as f32 / segments as f32;

        vertices.push(Vertex::new([x, -half_h, z], normal, [u, 0.0], color));
        vertices.push(Vertex::new([x, half_h, z], normal, [u, 1.0], color));
    }

    for i in 0..segments {
        let a = i * 2;
        indices.extend_from_slice(&[a, a + 1, a + 2, a + 2, a + 1, a + 3]);
    }

    let top_center = vertices.len() as u32;
    vertices.push(Vertex::colored([0.0, half_h, 0.0], color, [0.0, 1.0, 0.0]));
    let top_ring = vertices.len() as u32;
    for i in 0..=segments {
        let theta = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        vertices.push(Vertex::new(
            [radius * theta.cos(), half_h, radius * theta.sin()],
            [0.0, 1.0, 0.0],
            [theta.cos() * 0.5 + 0.5, theta.sin() * 0.5 + 0.5],
            color,
        ));
    }
    for i in 0..segments {
        indices.extend_from_slice(&[top_center, top_ring + i, top_ring + i + 1]);
    }

    Mesh { vertices, indices }
}

/// Hemisfério invertido para céu procedural.
pub fn sky_dome(radius: f32, segments: u32, rings: u32) -> Mesh {
    let color = Color::SKY;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let phi = std::f32::consts::PI * 0.5 * ring as f32 / rings as f32;
        let y = phi.sin() * radius;
        let ring_radius = phi.cos() * radius;

        for seg in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * seg as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();
            let pos = [x, y, z];
            let normal = [-x, -y, -z];
            vertices.push(Vertex::colored(pos, color, normal));
        }
    }

    for ring in 0..rings {
        for seg in 0..segments {
            let a = ring * (segments + 1) + seg;
            let b = a + segments + 1;
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }

    Mesh { vertices, indices }
}
