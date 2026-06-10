//! Gizmo de transformação — mover e rotacionar entidades no viewport.

use crate::graphics::Camera;
use crate::math::{Mat4, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Translate,
    Rotate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxis {
    None,
    X,
    Y,
    Z,
}

pub struct TransformGizmo {
    pub mode: GizmoMode,
    pub active_axis: GizmoAxis,
    drag_start_mouse: (f32, f32),
    drag_start_pos: [f32; 3],
    drag_start_rot: f32,
    dragging: bool,
}

impl Default for TransformGizmo {
    fn default() -> Self {
        Self {
            mode: GizmoMode::Translate,
            active_axis: GizmoAxis::None,
            drag_start_mouse: (0.0, 0.0),
            drag_start_pos: [0.0; 3],
            drag_start_rot: 0.0,
            dragging: false,
        }
    }
}

impl TransformGizmo {
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            GizmoMode::Translate => GizmoMode::Rotate,
            GizmoMode::Rotate => GizmoMode::Translate,
        };
    }

    pub fn set_mode_translate(&mut self) {
        self.mode = GizmoMode::Translate;
    }

    pub fn set_mode_rotate(&mut self) {
        self.mode = GizmoMode::Rotate;
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// Projeta posição mundo → pixels de tela.
    pub fn project(pos: Vec3, camera: &Camera, width: f32, height: f32) -> Option<(f32, f32)> {
        let clip = camera.view_projection() * pos.extend(1.0);
        if clip.w <= 0.05 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        Some(((ndc.x + 1.0) * 0.5 * width, (1.0 - ndc.y) * 0.5 * height))
    }

    fn axis_world(axis: GizmoAxis) -> Vec3 {
        match axis {
            GizmoAxis::X => Vec3::X,
            GizmoAxis::Y => Vec3::Y,
            GizmoAxis::Z => Vec3::Z,
            GizmoAxis::None => Vec3::ZERO,
        }
    }

    fn dist_point_to_segment(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
        let dx = bx - ax;
        let dy = by - ay;
        let len_sq = dx * dx + dy * dy;
        if len_sq < 1e-6 {
            let ddx = px - ax;
            let ddy = py - ay;
            return (ddx * ddx + ddy * ddy).sqrt();
        }
        let t = ((px - ax) * dx + (py - ay) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);
        let cx = ax + t * dx;
        let cy = ay + t * dy;
        let ddx = px - cx;
        let ddy = py - cy;
        (ddx * ddx + ddy * ddy).sqrt()
    }

    /// Detecta eixo clicado (threshold em pixels).
    pub fn pick_axis(
        &self,
        mouse: (f32, f32),
        origin: Vec3,
        camera: &Camera,
        width: f32,
        height: f32,
    ) -> GizmoAxis {
        let Some((ox, oy)) = Self::project(origin, camera, width, height) else {
            return GizmoAxis::None;
        };
        let axis_len = 1.2;
        let mut best = GizmoAxis::None;
        let mut best_dist = 18.0_f32;
        for axis in [GizmoAxis::X, GizmoAxis::Y, GizmoAxis::Z] {
            let end = origin + Self::axis_world(axis) * axis_len;
            let Some((ex, ey)) = Self::project(end, camera, width, height) else {
                continue;
            };
            let d = Self::dist_point_to_segment(mouse.0, mouse.1, ox, oy, ex, ey);
            if d < best_dist {
                best_dist = d;
                best = axis;
            }
        }
        best
    }

    pub fn begin_drag(
        &mut self,
        axis: GizmoAxis,
        mouse: (f32, f32),
        pos: [f32; 3],
        rot_y: f32,
    ) {
        self.active_axis = axis;
        self.drag_start_mouse = mouse;
        self.drag_start_pos = pos;
        self.drag_start_rot = rot_y;
        self.dragging = axis != GizmoAxis::None;
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
        self.active_axis = GizmoAxis::None;
    }

    /// Atualiza posição/rotação durante arraste.
    pub fn drag_update(
        &self,
        mouse: (f32, f32),
        camera: &Camera,
        width: f32,
        height: f32,
    ) -> Option<([f32; 3], f32)> {
        if !self.dragging || self.active_axis == GizmoAxis::None {
            return None;
        }
        let origin = Vec3::from_array(self.drag_start_pos);
        let axis = Self::axis_world(self.active_axis);
        let axis_end = origin + axis * 1.2;

        let (ox, oy) = Self::project(origin, camera, width, height)?;
        let (ex, ey) = Self::project(axis_end, camera, width, height)?;

        let sdx = ex - ox;
        let sdy = ey - oy;
        let len = (sdx * sdx + sdy * sdy).sqrt().max(1e-4);
        let axis_screen = (sdx / len, sdy / len);

        let mdx = mouse.0 - self.drag_start_mouse.0;
        let mdy = mouse.1 - self.drag_start_mouse.1;
        let along = mdx * axis_screen.0 + mdy * axis_screen.1;

        match self.mode {
            GizmoMode::Translate => {
                let sensitivity = 0.025;
                let mut pos = self.drag_start_pos;
                let idx = match self.active_axis {
                    GizmoAxis::X => 0,
                    GizmoAxis::Y => 1,
                    GizmoAxis::Z => 2,
                    GizmoAxis::None => return None,
                };
                pos[idx] += along * sensitivity;
                Some((pos, self.drag_start_rot))
            }
            GizmoMode::Rotate => {
                let sensitivity = 0.02;
                let rot = self.drag_start_rot + along * sensitivity;
                Some((self.drag_start_pos, rot))
            }
        }
    }

    /// Desenha eixos X/Y/Z como line strips.
    pub fn draw_axes(
        renderer: &mut crate::graphics::GfxRenderer,
        camera: &Camera,
        origin: Vec3,
        active: GizmoAxis,
    ) {
        let len = 1.2;
        let axes = [
            (Vec3::X * len, [1.0, 0.25, 0.25, 0.95], GizmoAxis::X),
            (Vec3::Y * len, [0.25, 1.0, 0.35, 0.95], GizmoAxis::Y),
            (Vec3::Z * len, [0.3, 0.5, 1.0, 0.95], GizmoAxis::Z),
        ];
        for (offset, mut col, axis) in axes {
            if active == axis {
                col[3] = 1.0;
                col[0] = (col[0] + 0.3_f32).min(1.0_f32);
                col[1] = (col[1] + 0.3_f32).min(1.0_f32);
                col[2] = (col[2] + 0.3_f32).min(1.0_f32);
            }
            let line = [origin.to_array(), (origin + offset).to_array()];
            renderer.draw_line_strip(camera, &line, col);
        }
    }
}

/// Utilitário para ray unproject simplificado (não usado no gizmo screen-space).
#[allow(dead_code)]
pub fn unproject_screen(
    x: f32,
    y: f32,
    depth: f32,
    inv_vp: Mat4,
    width: f32,
    height: f32,
) -> Vec3 {
    let ndc_x = x / width * 2.0 - 1.0;
    let ndc_y = 1.0 - y / height * 2.0;
    let clip = Vec3::new(ndc_x, ndc_y, depth * 2.0 - 1.0).extend(1.0);
    let world = inv_vp * clip;
    world.truncate() / world.w
}
