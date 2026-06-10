//! Estado global da partida Rock 3D.

use crate::core::EcsWorld;
use crate::assets::mesh_foot_offset;
use crate::assets::sample_quarry_height;
use crate::assets::AssetLibrary;
use crate::game::PLAYER_EYE;
use crate::game::{CollisionWorld, ParticleSystem, Player};
use crate::games::rock_3d::ground::snap_to_quarry_ground;
use crate::games::rock_3d::world::build_map_world;
use crate::game::world::{Drawable, GameWorld};
use crate::game::DayNightCycle;
use crate::games::rock_3d::ai::{AiAgent, AiSystem};
use crate::games::rock_3d::camera::{RockCameraController, RockCameraMode};
use crate::games::rock_3d::viewmodel::RockViewModel;
use crate::games::rock_3d::maps::MapConfig;
use crate::games::rock_3d::modes::{ArcadeMode, GameMode};
use crate::games::rock_3d::physics::RockPhysicsWorld;
use crate::games::rock_3d::progression::{PlayerProfile, SkillTree, UnlockRegistry, XpSystem};
use crate::games::rock_3d::replay::{ReplayEvent, ReplayRecording};
use crate::games::rock_3d::save::{load_profile, save_profile, Rock3DSave};
use crate::games::rock_3d::scoring::ScoreSystem;
use crate::games::rock_3d::targets::{TargetKind, TargetRegistry};
use crate::games::rock_3d::throw::{compute_trajectory_preview, ThrowController, ThrowPhase};
use crate::games::rock_3d::ui::RockHud;
use crate::games::rock_3d::weather::WeatherSystem;
use crate::graphics::{DrawMaterial, GfxRenderer};
use crate::math::{Mat4, Quat, Vec3};

pub struct Rock3DState {
    pub world: GameWorld,
    pub player: Player,
    pub collision: CollisionWorld,
    pub targets: TargetRegistry,
    pub throw_ctrl: ThrowController,
    pub physics: RockPhysicsWorld,
    pub weather: WeatherSystem,
    pub scoring: ScoreSystem,
    pub profile: PlayerProfile,
    pub skills: SkillTree,
    pub unlocks: UnlockRegistry,
    pub mode: GameMode,
    pub map: MapConfig,
    pub arcade: ArcadeMode,
    pub hud: RockHud,
    pub particles: ParticleSystem,
    pub day_night: DayNightCycle,
    pub ecs: EcsWorld,
    pub ai_agents: Vec<(u32, AiAgent)>,
    pub replay: ReplayRecording,
    pub scene_time: f32,
    pub time_remaining: f32,
    pub trajectory_preview: Vec<Vec3>,
    pub charging: bool,
    pub round_over: bool,
    pub camera_ctrl: RockCameraController,
    pub rock_vm: RockViewModel,
}

impl Rock3DState {
    pub fn new() -> Self {
        Self::new_with_assets(None)
    }

    pub fn new_with_assets(assets: Option<&AssetLibrary>) -> Self {
        let save = load_profile();
        let map = MapConfig::quarry();
        let mode = GameMode::Arcade;

        let mut state = Self {
            world: GameWorld::default(),
            player: Player::default(),
            collision: CollisionWorld::default(),
            targets: TargetRegistry::default(),
            throw_ctrl: ThrowController::default(),
            physics: RockPhysicsWorld::default(),
            weather: WeatherSystem::default(),
            scoring: ScoreSystem::default(),
            profile: save.profile,
            skills: save.skills,
            unlocks: save.unlocks,
            mode,
            map,
            arcade: ArcadeMode::default(),
            hud: RockHud::default(),
            particles: ParticleSystem::default(),
            day_night: DayNightCycle::default(),
            ecs: EcsWorld::new(),
            ai_agents: Vec::new(),
            replay: ReplayRecording::default(),
            scene_time: 0.0,
            time_remaining: mode.time_limit_secs().unwrap_or(9999.0),
            trajectory_preview: Vec::new(),
            charging: false,
            round_over: false,
            camera_ctrl: RockCameraController::default(),
            rock_vm: RockViewModel::default(),
        };

        state.setup_map();
        if let Some(lib) = assets {
            state.resnap_drawables_to_ground(lib);
        }
        state
    }

    pub fn setup_map(&mut self) {
        self.world = GameWorld::default();
        self.collision = build_map_world(&mut self.world, &self.map);
        self.map.kind.apply_weather(&mut self.weather);

        let spawn = self.map.spawn;
        let ground = crate::assets::sample_quarry_height(spawn.x, spawn.z);
        self.player.position = Vec3::new(spawn.x, ground + PLAYER_EYE, spawn.z);
        self.player.yaw = std::f32::consts::PI;
        self.player.pitch = 0.0;

        self.targets = TargetRegistry::default();
        ArcadeMode::setup_stage(self.arcade.stage, &mut self.targets);
        self.snap_targets_to_ground();
        self.spawn_target_drawables();
        self.setup_ai();

        self.throw_ctrl.max_throws = self.mode.max_throws();
        self.throw_ctrl.throws_remaining = self.mode.max_throws();
        self.time_remaining = self.mode.time_limit_secs().unwrap_or(9999.0);
        self.scoring = ScoreSystem::default();
        self.round_over = false;
        self.throw_ctrl.phase = ThrowPhase::Aiming;
    }

    fn setup_ai(&mut self) {
        self.ai_agents.clear();
        for t in &self.targets.targets {
            if t.kind.is_mobile() {
                self.ai_agents.push((t.id, AiAgent::new(t.position)));
            }
        }
    }

    fn snap_targets_to_ground(&mut self) {
        use crate::games::rock_3d::ground::snap_target_position;
        for t in &mut self.targets.targets {
            let lift = match t.kind {
                TargetKind::Plate => 1.1,
                TargetKind::Can | TargetKind::Bottle => 0.35,
                TargetKind::Bell => 1.6,
                TargetKind::Drone => 2.2,
                _ => 1.0,
            };
            t.position = snap_target_position(t.position.x, t.position.z, lift);
        }
    }

    fn resnap_drawables_to_ground(&mut self, lib: &AssetLibrary) {
        for d in &mut self.world.drawables {
            if d.model_id == "terrain" || d.target_id.is_some() {
                continue;
            }
            let foot = lib
                .models
                .get(&d.model_id)
                .map(|m| mesh_foot_offset(&m.mesh))
                .unwrap_or(0.9);
            let scale = d.scale.x;
            d.position = snap_to_quarry_ground(d.position.x, d.position.z, foot, scale);
        }
    }

    fn spawn_target_drawables(&mut self) {
        for t in &self.targets.targets {
            if !t.alive {
                continue;
            }
            let model = match t.kind {
                TargetKind::Plate | TargetKind::Bell => "target",
                TargetKind::Can | TargetKind::Bottle => "boulder_small",
                TargetKind::Drone => "boulder_medium",
                _ => "target",
            };
            self.world.drawables.push(Drawable {
                model_id: model.into(),
                position: t.position,
                rotation: Quat::from_rotation_y(t.position.x * 0.1),
                scale: Vec3::splat(t.kind.radius() * 2.0),
                material: if model.starts_with("boulder") {
                    DrawMaterial::rock()
                } else {
                    DrawMaterial::metal()
                },
                target_id: Some(t.id),
            });
        }
    }

    pub fn update(&mut self, dt: f32, mouse_delta: (f32, f32), input: &RockInput) {
        if self.round_over {
            return;
        }

        self.scene_time += dt;
        self.day_night.update(dt);
        self.particles.update(dt);
        self.weather.update(dt);

        if let Some(limit) = self.mode.time_limit_secs() {
            self.time_remaining = (self.time_remaining - dt).max(0.0);
            if self.time_remaining <= 0.0 {
                self.end_round();
            }
        }

        let cinematic = self.camera_ctrl.is_cinematic();

        if cinematic {
            self.camera_ctrl.apply_mouse(mouse_delta);
            if input.scroll.abs() > 0.001 {
                self.camera_ctrl.apply_scroll(input.scroll);
            }
        } else {
            self.player.yaw -= mouse_delta.0 * self.player.mouse_sensitivity;
            self.player.pitch -= mouse_delta.1 * self.player.mouse_sensitivity;
            self.player.pitch = self.player.pitch.clamp(-1.4, 1.4);
        }

        let moving = input.forward || input.backward || input.left || input.right;
        let charging_now = self.charging || self.throw_ctrl.phase == ThrowPhase::Charging;
        self.rock_vm.update(
            dt,
            moving && !cinematic,
            if cinematic { (0.0, 0.0) } else { mouse_delta },
            charging_now,
            self.throw_ctrl.charge,
        );

        if !cinematic {
            let mut fake_input = crate::game::InputState::default();
            fake_input.forward = input.forward;
            fake_input.backward = input.backward;
            fake_input.left = input.left;
            fake_input.right = input.right;
            fake_input.run = input.run;
            fake_input.mouse_delta = (0.0, 0.0);
            self.player
                .update(&fake_input, dt, &crate::game::BlockGrid::default(), &self.collision);
            let ground = sample_quarry_height(self.player.position.x, self.player.position.z);
            self.player.position.y = ground + PLAYER_EYE;
        }

        // Ajustes de arremesso
        if input.spin_left {
            self.throw_ctrl.adjust_spin_lateral(-dt * 4.0);
        }
        if input.spin_right {
            self.throw_ctrl.adjust_spin_lateral(dt * 4.0);
        }
        if input.spin_top {
            self.throw_ctrl.adjust_spin_top(dt * 4.0);
        }
        if input.spin_bottom {
            self.throw_ctrl.adjust_spin_top(-dt * 4.0);
        }
        if input.pitch_up {
            self.throw_ctrl.adjust_pitch(dt * 30.0);
        }
        if input.pitch_down {
            self.throw_ctrl.adjust_pitch(-dt * 30.0);
        }
        if input.yaw_left {
            self.throw_ctrl.adjust_yaw(-dt * 30.0);
        }
        if input.yaw_right {
            self.throw_ctrl.adjust_yaw(dt * 30.0);
        }

        if let Some(idx) = input.stone_select {
            self.throw_ctrl.select_stone(idx);
        }

        // Charge / release
        if input.charging && !self.physics.is_flying() {
            if !self.charging {
                self.throw_ctrl.begin_charge();
                self.charging = true;
            }
            self.throw_ctrl.update_charge(dt);
        } else if self.charging {
            self.charging = false;
            let cam = self.player.to_camera(1.0);
            let dispersion = 1.0 - self.skills.dispersion_reduction();
            let stone_r = self.throw_ctrl.selected_stone.stats().radius_m;
            let release_pos = self.rock_vm.release_world(&cam, stone_r);
            if let Some(params) = self.throw_ctrl.release(&cam, dispersion, release_pos) {
                let origin = params.origin;
                let throw_vel = params.direction * params.speed;
                self.physics.spawn(params);
                self.rock_vm.on_throw();
                self.camera_ctrl.begin_cinematic();
                self.replay.start();
                self.replay.record_frame(0.0, origin, throw_vel, Some(ReplayEvent::Throw));
            }
        } else {
            self.throw_ctrl.update_charge(dt);
        }

        // Física da pedra
        let wind = self.weather.wind * (1.0 - self.skills.wind_reduction());
        let threat = self.physics.position();
        let hits = self.physics.update(
            dt,
            wind,
            self.weather.air_density(),
            self.map.kind.gravity_scale() * self.weather.gravity_scale(),
            self.map.kind.ground_friction(),
            &mut self.targets,
        );

        for (id, damage, pos) in &hits {
            if let Some(target) = self.targets.targets.iter().find(|t| t.id == *id) {
                let dist = self.player.position.distance(target.position);
                let impact_speed = damage / self.throw_ctrl.selected_stone.stats().damage_mult;
                let bounces = self.physics.active.as_ref().map(|r| r.bounces).unwrap_or(0);
                let points = self.scoring.register_hit(target.points, dist, impact_speed, bounces);
                self.particles.emit_hit_dust(*pos);
                let xp = XpSystem::hit_xp(points, self.scoring.combo);
                if XpSystem::award(&mut self.profile, xp) {
                    self.unlocks.check_level(self.profile.level);
                }
                self.profile.total_hits += 1;
                self.profile.best_combo = self.profile.best_combo.max(self.scoring.combo);
                self.remove_target_drawable(*id);
                self.replay.record_frame(
                    self.scene_time,
                    *pos,
                    Vec3::ZERO,
                    Some(ReplayEvent::Hit {
                        target_id: *id,
                        damage: *damage,
                    }),
                );
            }
        }

        if !self.physics.is_flying() && self.throw_ctrl.phase == ThrowPhase::Flying {
            self.throw_ctrl.on_rock_landed();
            if self.camera_ctrl.mode == RockCameraMode::Cinematic {
                self.camera_ctrl.begin_return();
            }
            if self.throw_ctrl.throws_remaining == 0 || self.targets.alive_count() == 0 {
                self.end_round();
            }
        }

        // Replay frame
        if let Some(pos) = self.physics.position() {
            if let Some(rock) = &self.physics.active {
                self.replay.record_frame(self.scene_time, pos, rock.body.velocity, None);
            }
        }

        // IA
        let mut moved_targets = Vec::new();
        for (id, agent) in &mut self.ai_agents {
            if let Some(target) = self.targets.targets.iter_mut().find(|t| t.id == *id && t.alive) {
                AiSystem::update(agent, target, dt, threat);
                moved_targets.push((*id, target.position));
            }
        }
        for (id, pos) in moved_targets {
            self.sync_target_position(id, pos);
        }

        // Trajectory preview
        if self.skills.trajectory_preview() && !self.physics.is_flying() {
            let cam = self.player.to_camera(1.0);
            let stone_r = self.throw_ctrl.selected_stone.stats().radius_m;
            let origin = self.rock_vm.release_world(&cam, stone_r);
            let dir = crate::games::rock_3d::throw::aim_direction(
                &cam,
                self.throw_ctrl.aim_yaw_deg,
                self.throw_ctrl.aim_pitch_deg,
            );
            let speed = 5.0 + self.throw_ctrl.charge * 40.0;
            self.trajectory_preview = compute_trajectory_preview(origin, dir, speed, wind, 48, 0.04);
        } else {
            self.trajectory_preview.clear();
        }

        // HUD
        let nearest_dist = self
            .targets
            .targets
            .iter()
            .filter(|t| t.alive)
            .map(|t| self.player.position.distance(t.position))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        self.hud.update(
            self.throw_ctrl.selected_stone,
            self.throw_ctrl.charge_percent(),
            self.player.yaw.to_degrees(),
            self.player.pitch.to_degrees(),
            self.throw_ctrl.spin_lateral,
            self.throw_ctrl.spin_top,
            self.weather.wind_strength(),
            self.weather.wind_direction_label(),
            nearest_dist,
            self.scoring.session,
            self.scoring.combo,
            self.profile.xp,
            self.profile.level,
            self.throw_ctrl.throws_remaining,
            self.time_remaining,
            self.throw_ctrl.phase,
            self.mode,
            self.skills.trajectory_preview(),
        );
    }

    fn remove_target_drawable(&mut self, target_id: u32) {
        self.world.drawables.retain(|d| d.target_id != Some(target_id));
    }

    fn sync_target_position(&mut self, target_id: u32, pos: Vec3) {
        for d in &mut self.world.drawables {
            if d.target_id == Some(target_id) {
                d.position = pos;
            }
        }
    }

    fn end_round(&mut self) {
        self.round_over = true;
        self.scoring.apply_time_bonus(self.time_remaining);
        self.arcade.evaluate_stars(
            self.scoring.accuracy(),
            self.mode.time_limit_secs().unwrap_or(120.0) - self.time_remaining,
            self.mode.max_throws() - self.throw_ctrl.throws_remaining,
        );
        let save = Rock3DSave {
            profile: self.profile.clone(),
            skills: self.skills.clone(),
            unlocks: self.unlocks.clone(),
            best_scores: self.scoring.clone(),
            daily: Default::default(),
            version: 1,
        };
        let _ = save_profile(&save);
    }

    pub fn update_camera(&mut self, dt: f32, aspect: f32) -> crate::graphics::Camera {
        let rock_pos = self.physics.position();
        let rock_vel = self
            .physics
            .active
            .as_ref()
            .map(|r| r.body.velocity);
        self.camera_ctrl.update(
            dt,
            &self.player,
            aspect,
            rock_pos,
            rock_vel,
            self.physics.is_flying(),
        )
    }

    pub fn show_rock_in_hand(&self) -> bool {
        !self.physics.is_flying()
            && !self.camera_ctrl.is_cinematic()
            && matches!(
                self.throw_ctrl.phase,
                ThrowPhase::Idle | ThrowPhase::Aiming | ThrowPhase::Charging | ThrowPhase::Cooldown
            )
    }

    pub fn rock_speed(&self) -> f32 {
        self.physics
            .active
            .as_ref()
            .map(|r| r.body.velocity.length())
            .unwrap_or(0.0)
    }

    pub fn draw_rock(&self, renderer: &mut GfxRenderer, camera: &crate::graphics::Camera) {
        if let Some(rock) = &self.physics.active {
            let scale = rock.stone.radius_m * 2.0;
            let model = Mat4::from_scale_rotation_translation(
                Vec3::splat(scale),
                Quat::IDENTITY,
                rock.body.position,
            );
            // Desenha como esfera procedural se disponível
            let _ = (renderer, camera, model);
        }
    }

    pub fn persist(&self) {
        let save = Rock3DSave {
            profile: self.profile.clone(),
            skills: self.skills.clone(),
            unlocks: self.unlocks.clone(),
            best_scores: self.scoring.clone(),
            daily: Default::default(),
            version: 1,
        };
        let _ = save_profile(&save);
    }
}

/// Input simplificado para Rock 3D.
#[derive(Debug, Default)]
pub struct RockInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub run: bool,
    pub charging: bool,
    pub spin_left: bool,
    pub spin_right: bool,
    pub spin_top: bool,
    pub spin_bottom: bool,
    pub pitch_up: bool,
    pub pitch_down: bool,
    pub yaw_left: bool,
    pub yaw_right: bool,
    pub stone_select: Option<usize>,
    pub restart: bool,
    pub scroll: f32,
}
