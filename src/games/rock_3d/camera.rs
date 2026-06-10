//! Câmera cinematográfica — segue a pedra com mouse look e zoom.

use crate::game::Player;
use crate::graphics::Camera;
use crate::math::{Quat, Vec3};

const CINE_MAX_TIME: f32 = 12.0;
const RETURN_DURATION: f32 = 1.4;
const BASE_FOV: f32 = 70.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RockCameraMode {
    Player,
    Cinematic,
    Returning,
}

#[derive(Debug, Clone)]
pub struct RockCameraController {
    pub mode: RockCameraMode,
    pub cine_yaw: f32,
    pub cine_pitch: f32,
    /// < 1 = zoom in (FOV menor), > 1 = zoom out
    pub cine_zoom: f32,
    pub cine_elapsed: f32,
    pub return_t: f32,
    last_cine_pos: Vec3,
    last_cine_yaw: f32,
    last_cine_pitch: f32,
    last_cine_fov: f32,
}

impl Default for RockCameraController {
    fn default() -> Self {
        Self {
            mode: RockCameraMode::Player,
            cine_yaw: 0.0,
            cine_pitch: 0.2,
            cine_zoom: 1.0,
            cine_elapsed: 0.0,
            return_t: 0.0,
            last_cine_pos: Vec3::ZERO,
            last_cine_yaw: 0.0,
            last_cine_pitch: 0.0,
            last_cine_fov: BASE_FOV,
        }
    }
}

impl RockCameraController {
    pub fn is_cinematic(&self) -> bool {
        matches!(
            self.mode,
            RockCameraMode::Cinematic | RockCameraMode::Returning
        )
    }

    pub fn begin_cinematic(&mut self) {
        self.mode = RockCameraMode::Cinematic;
        self.cine_yaw = 0.0;
        self.cine_pitch = 0.18;
        self.cine_zoom = 1.0;
        self.cine_elapsed = 0.0;
        self.return_t = 0.0;
    }

    pub fn begin_return(&mut self) {
        self.mode = RockCameraMode::Returning;
        self.return_t = 0.0;
    }

    pub fn apply_mouse(&mut self, mouse_delta: (f32, f32)) {
        if !self.is_cinematic() {
            return;
        }
        self.cine_yaw += mouse_delta.0 * 0.0045;
        self.cine_pitch = (self.cine_pitch - mouse_delta.1 * 0.0045).clamp(-1.35, 1.35);
    }

    pub fn apply_scroll(&mut self, scroll: f32) {
        if !self.is_cinematic() {
            return;
        }
        self.cine_zoom = (self.cine_zoom - scroll * 0.1).clamp(0.4, 2.0);
    }

    pub fn update(
        &mut self,
        dt: f32,
        player: &Player,
        aspect: f32,
        rock_pos: Option<Vec3>,
        rock_vel: Option<Vec3>,
        rock_flying: bool,
    ) -> Camera {
        match self.mode {
            RockCameraMode::Player => player.to_camera(aspect),
            RockCameraMode::Cinematic => {
                self.cine_elapsed += dt;
                if !rock_flying || self.cine_elapsed > CINE_MAX_TIME {
                    self.begin_return();
                    return self.update(dt, player, aspect, rock_pos, rock_vel, false);
                }
                let cam = self.cinematic_camera(rock_pos, rock_vel, aspect);
                self.last_cine_pos = cam.position;
                self.last_cine_yaw = cam.yaw;
                self.last_cine_pitch = cam.pitch;
                self.last_cine_fov = cam.fov_y.to_degrees();
                cam
            }
            RockCameraMode::Returning => {
                self.return_t = (self.return_t + dt / RETURN_DURATION).min(1.0);
                let player_cam = player.to_camera(aspect);
                if self.return_t >= 1.0 {
                    self.mode = RockCameraMode::Player;
                    return player_cam;
                }
                let t = smoothstep(self.return_t);
                let cine = Camera {
                    position: self.last_cine_pos,
                    yaw: self.last_cine_yaw,
                    pitch: self.last_cine_pitch,
                    fov_y: self.last_cine_fov.to_radians(),
                    aspect,
                    near: 0.1,
                    far: 500.0,
                };
                blend_cameras(&cine, &player_cam, t)
            }
        }
    }

    fn cinematic_camera(
        &self,
        rock_pos: Option<Vec3>,
        rock_vel: Option<Vec3>,
        aspect: f32,
    ) -> Camera {
        let rock_pos = rock_pos.unwrap_or(Vec3::new(0.0, 2.0, -8.0));
        let vel = rock_vel.unwrap_or(Vec3::new(0.0, 0.0, -12.0));
        let speed = vel.length().max(2.0);

        let dist = (3.5 + speed * 0.08).clamp(2.5, 9.0);
        let height = 1.2 + speed * 0.02;

        let orbit = Quat::from_euler(glam::EulerRot::YXZ, self.cine_yaw, self.cine_pitch, 0.0);
        let offset = orbit * Vec3::new(0.0, height, dist);
        let cam_pos = rock_pos + offset;

        let to_rock = (rock_pos - cam_pos).normalize();
        let yaw = to_rock.x.atan2(to_rock.z);
        let pitch = to_rock.y.asin().clamp(-1.4, 1.4);

        let fov = (BASE_FOV / self.cine_zoom).clamp(28.0, 95.0);

        Camera {
            position: cam_pos,
            yaw,
            pitch,
            fov_y: fov.to_radians(),
            aspect,
            near: 0.1,
            far: 500.0,
        }
    }
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn blend_cameras(a: &Camera, b: &Camera, t: f32) -> Camera {
    Camera {
        position: a.position.lerp(b.position, t),
        yaw: lerp_angle(a.yaw, b.yaw, t),
        pitch: a.pitch + (b.pitch - a.pitch) * t,
        fov_y: a.fov_y + (b.fov_y - a.fov_y) * t,
        aspect: b.aspect,
        near: b.near,
        far: b.far,
    }
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut diff = b - a;
    while diff > std::f32::consts::PI {
        diff -= std::f32::consts::TAU;
    }
    while diff < -std::f32::consts::PI {
        diff += std::f32::consts::TAU;
    }
    a + diff * t
}
