//! Props do deserto — piramides, UFOs, ermitoes, animais.

use crate::assets::creatures::{box_mesh, cylinder, ellipsoid, merge_parts, sphere_mesh};
use crate::graphics::{Color, Mesh};
use crate::math::Vec3;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

pub fn generate_camel() -> Mesh {
    let sand = Color::rgb(0.72, 0.55, 0.38);
    let dark = Color::rgb(0.45, 0.32, 0.2);
    let mut parts = Vec::new();
    parts.push((ellipsoid(0.55, 0.45, 0.9, sand, 10, 8), v(0.0, 0.65, 0.0)));
    parts.push((ellipsoid(0.22, 0.35, 0.18, sand, 6, 5), v(0.0, 1.05, 0.55)));
    parts.push((box_mesh(0.12, 0.5, 0.12, dark), v(0.0, 1.35, 0.62)));
    for (dx, dz) in [(-0.35, -0.5), (0.35, -0.5), (-0.35, 0.5), (0.35, 0.5)] {
        parts.push((cylinder(0.08, 0.55, dark, 6), v(dx, 0.28, dz)));
    }
    merge_parts(&parts)
}

pub fn generate_goat() -> Mesh {
    let white = Color::rgb(0.88, 0.86, 0.82);
    let horn = Color::rgb(0.35, 0.28, 0.2);
    let mut parts = Vec::new();
    parts.push((ellipsoid(0.3, 0.28, 0.45, white, 8, 6), v(0.0, 0.45, 0.0)));
    parts.push((sphere_mesh(0.14, white, 6, 5), v(0.0, 0.62, 0.32)));
    parts.push((cylinder(0.03, 0.18, horn, 4), v(-0.08, 0.78, 0.35)));
    parts.push((cylinder(0.03, 0.18, horn, 4), v(0.08, 0.78, 0.35)));
    for (dx, dz) in [(-0.14, -0.18), (0.14, -0.18), (-0.14, 0.18), (0.14, 0.18)] {
        parts.push((cylinder(0.04, 0.28, white, 5), v(dx, 0.14, dz)));
    }
    merge_parts(&parts)
}

pub fn generate_snake() -> Mesh {
    let scale_c = Color::rgb(0.55, 0.42, 0.18);
    let belly = Color::rgb(0.78, 0.68, 0.42);
    let mut parts = Vec::new();
    for i in 0..8 {
        let t = i as f32 / 7.0;
        let x = (t * 3.0).sin() * 0.15;
        let z = t * 0.9 - 0.45;
        let r = 0.06 + (1.0 - t) * 0.04;
        let c = if i % 2 == 0 { scale_c } else { belly };
        parts.push((sphere_mesh(r, c, 5, 4), v(x, 0.06 + r, z)));
    }
    parts.push((sphere_mesh(0.1, scale_c, 6, 5), v(0.0, 0.1, 0.48)));
    merge_parts(&parts)
}

pub fn generate_scorpion() -> Mesh {
    let chitin = Color::rgb(0.35, 0.22, 0.12);
    let mut parts = Vec::new();
    parts.push((ellipsoid(0.18, 0.08, 0.28, chitin, 6, 4), v(0.0, 0.1, 0.0)));
    parts.push((box_mesh(0.04, 0.35, 0.04, chitin), v(0.0, 0.32, -0.12)));
    parts.push((sphere_mesh(0.07, chitin, 5, 4), v(0.0, 0.08, 0.22)));
    for i in 0..4 {
        let a = i as f32 * 0.5 - 0.75;
        parts.push((cylinder(0.015, 0.22, chitin, 3), v(a * 0.3, 0.04, 0.1 + i as f32 * 0.06)));
    }
    merge_parts(&parts)
}

pub fn generate_hermit() -> Mesh {
    let robe = Color::rgb(0.38, 0.32, 0.28);
    let skin = Color::rgb(0.72, 0.55, 0.42);
    let beard = Color::rgb(0.55, 0.45, 0.35);
    let staff = Color::rgb(0.42, 0.28, 0.14);
    let mut parts = Vec::new();
    parts.push((cylinder(0.22, 1.1, robe, 8), v(0.0, 0.55, 0.0)));
    parts.push((sphere_mesh(0.16, skin, 8, 6), v(0.0, 1.15, 0.0)));
    parts.push((ellipsoid(0.12, 0.22, 0.1, beard, 5, 4), v(0.0, 1.02, 0.1)));
    parts.push((cylinder(0.03, 1.4, staff, 5), v(0.35, 0.7, 0.0)));
    merge_parts(&parts)
}

pub fn generate_et() -> Mesh {
    let green = Color::rgb(0.45, 0.72, 0.38);
    let dark = Color::rgb(0.2, 0.35, 0.18);
    let mut parts = Vec::new();
    parts.push((ellipsoid(0.25, 0.55, 0.18, green, 8, 6), v(0.0, 0.55, 0.0)));
    parts.push((ellipsoid(0.32, 0.38, 0.28, green, 8, 6), v(0.0, 1.05, 0.0)));
    parts.push((sphere_mesh(0.09, dark, 6, 4), v(-0.1, 1.12, 0.18)));
    parts.push((sphere_mesh(0.09, dark, 6, 4), v(0.1, 1.12, 0.18)));
    for dx in [-0.08, 0.08] {
        parts.push((cylinder(0.025, 0.45, green, 4), v(dx, 0.22, 0.0)));
    }
    merge_parts(&parts)
}

pub fn generate_pyramid() -> Mesh {
    let stone = Color::rgb(0.78, 0.68, 0.42);
    let mut parts = Vec::new();
    parts.push((box_mesh(14.0, 0.8, 14.0, stone), v(0.0, 0.4, 0.0)));
    for (w, h, y) in [(12.0, 3.5, 2.2), (8.5, 3.0, 5.0), (5.0, 2.5, 7.2), (2.5, 2.0, 9.0)] {
        parts.push((box_mesh(w, h, w, stone), v(0.0, y, 0.0)));
    }
    merge_parts(&parts)
}

pub fn generate_ufo() -> Mesh {
    let metal = Color::rgb(0.55, 0.58, 0.62);
    let glow = Color::rgb(0.2, 0.85, 1.0);
    let mut parts = Vec::new();
    parts.push((ellipsoid(1.2, 0.25, 1.2, metal, 12, 6), v(0.0, 0.0, 0.0)));
    parts.push((sphere_mesh(0.45, glow, 8, 6), v(0.0, 0.35, 0.0)));
    parts.push((cylinder(1.4, 0.06, glow, 16), v(0.0, -0.08, 0.0)));
    merge_parts(&parts)
}

pub fn generate_mirage() -> Mesh {
    let haze = Color::rgba(0.75, 0.82, 0.95, 0.35);
    ellipsoid(3.0, 1.5, 3.0, haze, 8, 6)
}

pub fn generate_grass_clump() -> Mesh {
    let green = Color::rgb(0.28, 0.55, 0.22);
    let dark = Color::rgb(0.18, 0.42, 0.14);
    let mut parts = Vec::new();
    for i in 0..7 {
        let a = i as f32 * 0.9;
        let r = 0.15 + (i % 3) as f32 * 0.08;
        parts.push((
            cylinder(0.02, 0.35 + (i % 2) as f32 * 0.12, if i % 2 == 0 { green } else { dark }, 4),
            v(a.cos() * r, 0.18, a.sin() * r),
        ));
    }
    merge_parts(&parts)
}

pub fn generate_palm_tree() -> Mesh {
    let trunk = Color::rgb(0.45, 0.32, 0.18);
    let leaf = Color::rgb(0.22, 0.58, 0.2);
    let nut = Color::rgb(0.35, 0.28, 0.12);
    let mut parts = Vec::new();
    parts.push((cylinder(0.18, 4.5, trunk, 8), v(0.0, 2.25, 0.0)));
    for i in 0..6 {
        let a = i as f32 * 1.05;
        parts.push((
            ellipsoid(0.08, 1.8, 0.35, leaf, 4, 3),
            v(a.cos() * 0.3, 4.6, a.sin() * 0.3),
        ));
    }
    parts.push((sphere_mesh(0.12, nut, 5, 4), v(0.0, 3.8, 0.2)));
    merge_parts(&parts)
}

pub fn generate_well() -> Mesh {
    let stone = Color::rgb(0.5, 0.48, 0.45);
    let wood = Color::rgb(0.38, 0.26, 0.14);
    let mut parts = Vec::new();
    parts.push((cylinder(0.9, 0.6, stone, 10), v(0.0, 0.3, 0.0)));
    parts.push((cylinder(0.75, 0.5, Color::rgb(0.12, 0.28, 0.42), 8), v(0.0, 0.15, 0.0)));
    parts.push((box_mesh(0.08, 1.6, 0.08, wood), v(-0.7, 0.8, 0.0)));
    parts.push((box_mesh(0.08, 1.6, 0.08, wood), v(0.7, 0.8, 0.0)));
    parts.push((cylinder(0.05, 1.5, wood, 6), v(0.0, 1.55, 0.0)));
    merge_parts(&parts)
}

pub fn generate_bird() -> Mesh {
    let body = Color::rgb(0.55, 0.35, 0.18);
    let wing = Color::rgb(0.42, 0.28, 0.14);
    let beak = Color::rgb(0.9, 0.65, 0.1);
    let mut parts = Vec::new();
    parts.push((ellipsoid(0.08, 0.07, 0.12, body, 6, 5), v(0.0, 0.12, 0.0)));
    parts.push((ellipsoid(0.14, 0.03, 0.08, wing, 4, 3), v(0.0, 0.14, 0.0)));
    parts.push((box_mesh(0.04, 0.03, 0.08, beak), v(0.0, 0.13, 0.14)));
    merge_parts(&parts)
}

pub fn generate_lion() -> Mesh {
    let fur = Color::rgb(0.78, 0.55, 0.22);
    let mane = Color::rgb(0.45, 0.28, 0.12);
    let mut parts = Vec::new();
    parts.push((ellipsoid(0.5, 0.42, 0.85, fur, 10, 8), v(0.0, 0.55, 0.0)));
    parts.push((sphere_mesh(0.28, fur, 8, 6), v(0.0, 0.72, 0.55)));
    parts.push((ellipsoid(0.32, 0.28, 0.12, mane, 8, 6), v(0.0, 0.82, 0.48)));
    for (dx, dz) in [(-0.32, -0.45), (0.32, -0.45), (-0.32, 0.45), (0.32, 0.45)] {
        parts.push((cylinder(0.09, 0.42, fur, 6), v(dx, 0.21, dz)));
    }
    merge_parts(&parts)
}

pub fn generate_dog() -> Mesh {
    let brown = Color::rgb(0.52, 0.35, 0.22);
    let dark = Color::rgb(0.28, 0.18, 0.1);
    let mut parts = Vec::new();
    parts.push((ellipsoid(0.28, 0.26, 0.45, brown, 8, 6), v(0.0, 0.38, 0.0)));
    parts.push((sphere_mesh(0.16, brown, 7, 5), v(0.0, 0.48, 0.32)));
    parts.push((box_mesh(0.06, 0.12, 0.06, dark), v(0.0, 0.62, 0.42)));
    for (dx, dz) in [(-0.14, -0.2), (0.14, -0.2), (-0.14, 0.2), (0.14, 0.2)] {
        parts.push((cylinder(0.04, 0.22, brown, 5), v(dx, 0.11, dz)));
    }
    merge_parts(&parts)
}

pub fn generate_desert_cabin() -> Mesh {
    let mud = Color::rgb(0.62, 0.48, 0.32);
    let wood = Color::rgb(0.42, 0.3, 0.18);
    let mut parts = Vec::new();
    parts.push((box_mesh(3.2, 2.2, 3.0, mud), v(0.0, 1.1, 0.0)));
    parts.push((box_mesh(3.6, 0.25, 3.4, wood), v(0.0, 2.25, 0.0)));
    parts.push((box_mesh(0.9, 1.4, 0.08, wood), v(0.0, 1.0, 1.52)));
    merge_parts(&parts)
}

pub fn generate_desert_house() -> Mesh {
    let wall = Color::rgb(0.68, 0.52, 0.36);
    let roof = Color::rgb(0.38, 0.26, 0.14);
    let mut parts = Vec::new();
    parts.push((box_mesh(4.5, 2.8, 4.0, wall), v(0.0, 1.4, 0.0)));
    parts.push((box_mesh(5.0, 0.3, 4.5, roof), v(0.0, 2.85, 0.0)));
    parts.push((box_mesh(1.0, 1.8, 0.1, Color::rgb(0.25, 0.18, 0.1)), v(0.0, 0.9, 2.05)));
    merge_parts(&parts)
}

pub fn generate_desert_market() -> Mesh {
    let cloth = Color::rgb(0.78, 0.35, 0.22);
    let pole = Color::rgb(0.4, 0.28, 0.16);
    let mut parts = Vec::new();
    for dx in [-2.0_f32, 2.0] {
        parts.push((cylinder(0.08, 3.0, pole, 6), v(dx, 1.5, 0.0)));
    }
    parts.push((box_mesh(5.0, 0.08, 3.5, cloth), v(0.0, 2.8, 0.0)));
    parts.push((box_mesh(2.0, 0.8, 1.2, pole), v(0.0, 0.4, 0.0)));
    merge_parts(&parts)
}

pub fn generate_desert_castle() -> Mesh {
    let stone = Color::rgb(0.58, 0.52, 0.44);
    let mut parts = Vec::new();
    parts.push((box_mesh(10.0, 4.0, 10.0, stone), v(0.0, 2.0, 0.0)));
    for (dx, dz) in [(-4.5, -4.5), (4.5, -4.5), (-4.5, 4.5), (4.5, 4.5)] {
        parts.push((box_mesh(2.0, 7.0, 2.0, stone), v(dx, 3.5, dz)));
    }
    parts.push((box_mesh(4.0, 1.5, 3.0, Color::rgb(0.3, 0.22, 0.15)), v(0.0, 1.0, 5.2)));
    merge_parts(&parts)
}

pub fn generate_desert_tower() -> Mesh {
    let stone = Color::rgb(0.55, 0.5, 0.42);
    let mut parts = Vec::new();
    parts.push((cylinder(1.8, 8.0, stone, 8), v(0.0, 4.0, 0.0)));
    parts.push((cylinder(2.2, 0.6, stone, 8), v(0.0, 8.3, 0.0)));
    merge_parts(&parts)
}

pub fn generate_desert_caravan() -> Mesh {
    let cloth = Color::rgb(0.72, 0.55, 0.28);
    let wood = Color::rgb(0.38, 0.26, 0.14);
    let mut parts = Vec::new();
    parts.push((box_mesh(2.5, 1.2, 1.8, wood), v(0.0, 0.6, 0.0)));
    for dx in [-1.2_f32, 1.2] {
        parts.push((cylinder(0.35, 0.35, wood, 8), v(dx, 0.35, 0.8)));
        parts.push((cylinder(0.35, 0.35, wood, 8), v(dx, 0.35, -0.8)));
    }
    parts.push((box_mesh(2.8, 0.06, 2.0, cloth), v(0.0, 2.2, 0.0)));
    for dx in [-1.1_f32, 1.1] {
        parts.push((cylinder(0.04, 2.5, wood, 4), v(dx, 1.25, 0.0)));
    }
    merge_parts(&parts)
}

pub fn generate_npc_vendor() -> Mesh {
    let robe = Color::rgb(0.75, 0.45, 0.2);
    let skin = Color::rgb(0.72, 0.55, 0.42);
    let mut parts = Vec::new();
    parts.push((cylinder(0.24, 1.05, robe, 8), v(0.0, 0.52, 0.0)));
    parts.push((sphere_mesh(0.15, skin, 7, 5), v(0.0, 1.12, 0.0)));
    parts.push((box_mesh(0.5, 0.35, 0.35, Color::rgb(0.55, 0.38, 0.22)), v(0.35, 0.55, 0.0)));
    merge_parts(&parts)
}

pub fn generate_npc_soldier() -> Mesh {
    let armor = Color::rgb(0.48, 0.46, 0.42);
    let cloth = Color::rgb(0.55, 0.22, 0.15);
    let mut parts = Vec::new();
    parts.push((cylinder(0.26, 1.1, cloth, 8), v(0.0, 0.55, 0.0)));
    parts.push((box_mesh(0.35, 0.35, 0.08, armor), v(0.0, 0.75, 0.18)));
    parts.push((sphere_mesh(0.15, armor, 7, 5), v(0.0, 1.18, 0.0)));
    parts.push((cylinder(0.04, 1.6, Color::rgb(0.35, 0.3, 0.28), 5), v(0.4, 0.8, 0.0)));
    merge_parts(&parts)
}

pub fn generate_npc_caravan() -> Mesh {
    generate_npc_vendor()
}

pub fn generate_npc_hunter() -> Mesh {
    let leather = Color::rgb(0.42, 0.28, 0.16);
    let skin = Color::rgb(0.68, 0.5, 0.38);
    let mut parts = Vec::new();
    parts.push((cylinder(0.22, 1.0, leather, 7), v(0.0, 0.5, 0.0)));
    parts.push((sphere_mesh(0.14, skin, 6, 5), v(0.0, 1.08, 0.0)));
    parts.push((cylinder(0.03, 1.3, leather, 4), v(0.3, 0.65, 0.0)));
    merge_parts(&parts)
}

pub fn generate_npc_builder() -> Mesh {
    let shirt = Color::rgb(0.52, 0.42, 0.32);
    let skin = Color::rgb(0.7, 0.52, 0.4);
    let mut parts = Vec::new();
    parts.push((cylinder(0.23, 1.0, shirt, 7), v(0.0, 0.5, 0.0)));
    parts.push((sphere_mesh(0.14, skin, 6, 5), v(0.0, 1.08, 0.0)));
    parts.push((box_mesh(0.35, 0.06, 0.2, Color::rgb(0.45, 0.32, 0.18)), v(0.3, 0.7, 0.0)));
    merge_parts(&parts)
}

pub fn generate_npc_citizen() -> Mesh {
    let robe = Color::rgb(0.58, 0.48, 0.38);
    let skin = Color::rgb(0.68, 0.52, 0.4);
    let mut parts = Vec::new();
    parts.push((cylinder(0.22, 1.0, robe, 7), v(0.0, 0.5, 0.0)));
    parts.push((sphere_mesh(0.14, skin, 6, 5), v(0.0, 1.08, 0.0)));
    merge_parts(&parts)
}

pub fn generate_mountain_rock() -> Mesh {
    let rock = Color::rgb(0.42, 0.4, 0.38);
    let mut parts = Vec::new();
    parts.push((ellipsoid(3.5, 2.8, 3.2, rock, 8, 6), v(0.0, 2.5, 0.0)));
    parts.push((ellipsoid(2.2, 1.8, 2.0, rock, 7, 5), v(1.2, 3.8, 0.5)));
    parts.push((ellipsoid(1.8, 1.4, 1.6, rock, 6, 5), v(-1.0, 4.2, -0.8)));
    merge_parts(&parts)
}
