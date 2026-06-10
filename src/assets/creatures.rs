//! Malhas procedurais — ovelhas e cercas estilo fazenda.

use crate::graphics::{Color, Mesh, Vertex};
use crate::math::Vec3;

pub fn generate_sheep() -> Mesh {
    let mut parts: Vec<(Mesh, Vec3)> = Vec::new();
    let wool = Color::rgb(0.92, 0.9, 0.86);
    let skin = Color::rgb(0.55, 0.42, 0.35);
    let dark = Color::rgb(0.15, 0.12, 0.1);

    parts.push((ellipsoid(0.42, 0.38, 0.55, wool, 10, 8), Vec3::new(0.0, 0.55, 0.0)));
    parts.push((ellipsoid(0.22, 0.2, 0.22, wool, 8, 6), Vec3::new(0.0, 0.72, 0.35)));
    parts.push((sphere_mesh(0.14, skin, 8, 6), Vec3::new(0.0, 0.68, 0.52)));
    parts.push((sphere_mesh(0.04, dark, 4, 3), Vec3::new(-0.05, 0.72, 0.62)));
    parts.push((sphere_mesh(0.04, dark, 4, 3), Vec3::new(0.05, 0.72, 0.62)));

    for (dx, dz) in [(-0.18, -0.22), (0.18, -0.22), (-0.18, 0.22), (0.18, 0.22)] {
        parts.push((cylinder(0.05, 0.32, skin, 6), Vec3::new(dx, 0.16, dz)));
    }

    merge_parts(&parts)
}

pub fn generate_dirt_block() -> Mesh {
    tinted_cube(Color::rgb(0.42, 0.28, 0.14))
}

pub fn generate_stone_block() -> Mesh {
    tinted_cube(Color::rgb(0.52, 0.5, 0.48))
}

fn tinted_cube(color: Color) -> Mesh {
    crate::graphics::cube(color)
}

pub fn generate_stone_wall() -> Mesh {
    tinted_cube(Color::rgb(0.48, 0.46, 0.44))
}

pub fn generate_wood_wall() -> Mesh {
    tinted_cube(Color::rgb(0.42, 0.3, 0.16))
}

pub fn generate_fence_post() -> Mesh {
    let wood = Color::rgb(0.45, 0.32, 0.18);
    let wood_dark = Color::rgb(0.35, 0.24, 0.12);
    let mut parts: Vec<(Mesh, Vec3)> = Vec::new();
    parts.push((cylinder(0.07, 1.1, wood, 8), Vec3::new(0.0, 0.55, 0.0)));
    parts.push((cylinder(0.09, 0.08, wood_dark, 8), Vec3::new(0.0, 0.08, 0.0)));
    parts.push((box_mesh(1.6, 0.06, 0.06, wood), Vec3::new(0.0, 0.45, 0.0)));
    parts.push((box_mesh(1.6, 0.06, 0.06, wood), Vec3::new(0.0, 0.75, 0.0)));
    parts.push((box_mesh(0.06, 0.06, 1.6, wood), Vec3::new(0.0, 0.45, 0.0)));
    parts.push((box_mesh(0.06, 0.06, 1.6, wood), Vec3::new(0.0, 0.75, 0.0)));
    merge_parts(&parts)
}

pub(crate) fn merge_parts(parts: &[(Mesh, Vec3)]) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (mesh, off) in parts {
        let base = vertices.len() as u32;
        for v in &mesh.vertices {
            let mut nv = *v;
            nv.position[0] += off.x;
            nv.position[1] += off.y;
            nv.position[2] += off.z;
            vertices.push(nv);
        }
        indices.extend(mesh.indices.iter().map(|i| base + i));
    }
    Mesh { vertices, indices }
}

pub(crate) fn ellipsoid(rx: f32, ry: f32, rz: f32, color: Color, segs: u32, rings: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for ring in 0..=rings {
        let v = ring as f32 / rings as f32;
        let phi = v * std::f32::consts::PI;
        for seg in 0..=segs {
            let u = seg as f32 / segs as f32;
            let theta = u * std::f32::consts::TAU;
            let x = rx * phi.sin() * theta.cos();
            let y = ry * phi.cos();
            let z = rz * phi.sin() * theta.sin();
            let n = Vec3::new(x / rx, y / ry, z / rz).normalize();
            vertices.push(Vertex::new([x, y, z], n.to_array(), [u, v], color));
        }
    }
    for ring in 0..rings {
        for seg in 0..segs {
            let a = ring * (segs + 1) + seg;
            let b = a + segs + 1;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    Mesh { vertices, indices }
}

pub(crate) fn sphere_mesh(r: f32, color: Color, segs: u32, rings: u32) -> Mesh {
    ellipsoid(r, r, r, color, segs, rings)
}

pub(crate) fn cylinder(r: f32, h: f32, color: Color, segs: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let hh = h / 2.0;
    for i in 0..=segs {
        let a = i as f32 / segs as f32 * std::f32::consts::TAU;
        let x = a.cos() * r;
        let z = a.sin() * r;
        let n = [a.cos(), 0.0, a.sin()];
        vertices.push(Vertex::new([x, -hh, z], n, [0.0, 0.0], color));
        vertices.push(Vertex::new([x, hh, z], n, [1.0, 1.0], color));
    }
    for i in 0..segs {
        let a = i * 2;
        indices.extend_from_slice(&[a, a + 1, a + 2, a + 2, a + 1, a + 3]);
    }
    Mesh { vertices, indices }
}

pub(crate) fn box_mesh(w: f32, h: f32, d: f32, color: Color) -> Mesh {
    let hw = w / 2.0;
    let hh = h / 2.0;
    let hd = d / 2.0;
    let corners = [
        ([-hw, -hh, -hd], [0.0, -1.0, 0.0]),
        ([hw, -hh, -hd], [0.0, -1.0, 0.0]),
        ([hw, -hh, hd], [0.0, -1.0, 0.0]),
        ([-hw, -hh, hd], [0.0, -1.0, 0.0]),
        ([-hw, hh, -hd], [0.0, 1.0, 0.0]),
        ([hw, hh, -hd], [0.0, 1.0, 0.0]),
        ([hw, hh, hd], [0.0, 1.0, 0.0]),
        ([-hw, hh, hd], [0.0, 1.0, 0.0]),
    ];
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let faces: [[usize; 4]; 6] = [
        [0, 1, 2, 3],
        [4, 7, 6, 5],
        [0, 4, 5, 1],
        [2, 6, 7, 3],
        [0, 3, 7, 4],
        [1, 5, 6, 2],
    ];
    let normals = [
        [0.0, -1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0],
        [-1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
    ];
    for (fi, face) in faces.iter().enumerate() {
        let base = vertices.len() as u32;
        let n = normals[fi];
        for &ci in face {
            let p = corners[ci].0;
            vertices.push(Vertex::new(p, n, [0.0, 0.0], color));
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    Mesh { vertices, indices }
}
