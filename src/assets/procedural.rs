//! Geometria procedural de alta qualidade — substitui modelos low-poly.

use crate::graphics::{Color, Mesh, Vertex};
use crate::math::Vec3;
use noise::{NoiseFn, Perlin};

/// Rochas realistas via icosaedro subdividido + displacement de ruído.
pub fn generate_boulder(seed: u32, radius: f32, subdivisions: u32) -> Mesh {
    let perlin = Perlin::new(seed);
    let (verts, indices) = icosphere(subdivisions);

    let mut vertices = Vec::with_capacity(verts.len());
    for (i, pos) in verts.iter().enumerate() {
        let dir = pos.normalize();
        let n = perlin.get([
            (dir.x * 3.0 + seed as f32) as f64,
            (dir.y * 3.0) as f64,
            (dir.z * 3.0 + seed as f32 * 0.1) as f64,
        ]) as f32;
        let n2 = perlin.get([
            (dir.x * 7.0) as f64,
            (dir.y * 7.0) as f64,
            (dir.z * 7.0) as f64,
        ]) as f32;
        let disp = radius * (1.0 + n * 0.22 + n2 * 0.08);
        let p = dir * disp;

        let gray = 0.38 + n * 0.12;
        let color = Color::rgb(gray, gray * 0.92, gray * 0.78);

        vertices.push(Vertex::new(
            p.to_array(),
            dir.to_array(),
            [i as f32 * 0.01, i as f32 * 0.013],
            color,
        ));
    }

    Mesh {
        vertices,
        indices: indices.iter().map(|&i| i as u32).collect(),
    }
}

/// Tronco retorcido para deserto árido.
pub fn generate_dead_tree(seed: u32, height: f32) -> Mesh {
    let perlin = Perlin::new(seed + 100);
    let segments = 24;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let y = t * height;
        let twist = t * 1.2 + seed as f32 * 0.01;
        let bend = perlin.get([t as f64 * 2.0, 0.0, seed as f64]) as f32 * 0.4;
        let cx = bend * t;
        let cz = perlin.get([0.0, t as f64 * 2.0, seed as f64]) as f32 * 0.3 * t;
        let r = 0.12 * (1.0 - t * 0.65) + 0.03;

        for j in 0..8 {
            let a = j as f32 / 8.0 * std::f32::consts::TAU + twist;
            let px = cx + a.cos() * r;
            let pz = cz + a.sin() * r;
            let color = Color::rgb(0.32, 0.26, 0.2);
            vertices.push(Vertex::new(
                [px, y, pz],
                [a.cos(), 0.2, a.sin()],
                [j as f32 / 8.0, t],
                color,
            ));
        }
    }

    for i in 0..segments {
        for j in 0..8 {
            let a = (i * 8 + j) as u32;
            let b = a + 8;
            let c = a + 1;
            let d = b + 1;
            if j == 7 {
                indices.extend_from_slice(&[a, b, a + 8 - 7, b, b + 8 - 7, a + 8 - 7]);
            } else {
                indices.extend_from_slice(&[a, b, c, c, b, d]);
            }
        }
    }

    Mesh { vertices, indices }
}

/// Alvo de tiro — placa metálica em poste de madeira.
pub fn generate_shooting_target() -> Mesh {
    use crate::graphics::mesh_factory::shape_mesh;

    let post = cylinder_mesh(0.06, 1.6, Color::rgb(0.35, 0.22, 0.12), 20);
    let board = scale_mesh(
        shape_mesh("cube", Color::rgb(0.58, 0.58, 0.62)),
        Vec3::new(0.9, 0.6, 0.05),
    );
    let center = scale_mesh(
        shape_mesh("cube", Color::rgb(0.85, 0.12, 0.08)),
        Vec3::new(0.22, 0.22, 0.04),
    );

    merge_meshes(&[
        (post, Vec3::new(0.0, 0.8, 0.0)),
        (board, Vec3::new(0.0, 1.5, 0.05)),
        (center, Vec3::new(0.0, 1.5, 0.1)),
    ])
}

fn scale_mesh(mut mesh: Mesh, scale: Vec3) -> Mesh {
    for v in &mut mesh.vertices {
        v.position[0] *= scale.x;
        v.position[1] *= scale.y;
        v.position[2] *= scale.z;
    }
    mesh
}

fn merge_meshes(parts: &[(Mesh, Vec3)]) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (mesh, offset) in parts {
        let base = vertices.len() as u32;
        for v in &mesh.vertices {
            let mut nv = *v;
            nv.position[0] += offset.x;
            nv.position[1] += offset.y;
            nv.position[2] += offset.z;
            vertices.push(nv);
        }
        indices.extend(mesh.indices.iter().map(|i| base + i));
    }
    Mesh { vertices, indices }
}

fn cylinder_mesh(r: f32, h: f32, color: Color, segs: u32) -> Mesh {
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

fn icosphere(subdivisions: u32) -> (Vec<Vec3>, Vec<u32>) {
    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let mut verts = vec![
        Vec3::new(-1.0, t, 0.0).normalize(),
        Vec3::new(1.0, t, 0.0).normalize(),
        Vec3::new(-1.0, -t, 0.0).normalize(),
        Vec3::new(1.0, -t, 0.0).normalize(),
        Vec3::new(0.0, -1.0, t).normalize(),
        Vec3::new(0.0, 1.0, t).normalize(),
        Vec3::new(0.0, -1.0, -t).normalize(),
        Vec3::new(0.0, 1.0, -t).normalize(),
        Vec3::new(t, 0.0, -1.0).normalize(),
        Vec3::new(t, 0.0, 1.0).normalize(),
        Vec3::new(-t, 0.0, -1.0).normalize(),
        Vec3::new(-t, 0.0, 1.0).normalize(),
    ];
    let mut faces: Vec<[u32; 3]> = vec![
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
        [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
        [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ];

    for _ in 0..subdivisions {
        let mut new_faces = Vec::new();
        let mut midpoint_cache = std::collections::HashMap::new();

        for tri in &faces {
            let mut mids = [0u32; 3];
            for i in 0..3 {
                let a = tri[i];
                let b = tri[(i + 1) % 3];
                let key = if a < b { (a, b) } else { (b, a) };
                mids[i] = *midpoint_cache.entry(key).or_insert_with(|| {
                    let m = ((verts[a as usize] + verts[b as usize]) * 0.5).normalize();
                    let idx = verts.len() as u32;
                    verts.push(m);
                    idx
                });
            }
            new_faces.extend_from_slice(&[
                [tri[0], mids[0], mids[2]],
                [tri[1], mids[1], mids[0]],
                [tri[2], mids[2], mids[1]],
                [mids[0], mids[1], mids[2]],
            ]);
        }
        faces = new_faces;
    }

    let indices: Vec<u32> = faces.iter().flat_map(|f| f.iter().copied()).collect();
    (verts, indices)
}
