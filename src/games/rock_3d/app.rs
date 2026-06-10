//! Loop principal do Rock 3D.

use crate::assets::GpuAssetCache;
use crate::games::rock_3d::load_rock_assets;

use crate::audio::AudioEngine;

use crate::game::should_draw;

use crate::games::rock_3d::audio::RockAudio;

use crate::games::rock_3d::state::{Rock3DState, RockInput};

use crate::games::rock_3d::throw::ThrowPhase;

use crate::assets::AssetLibrary;
use crate::graphics::{BackendKind, DayNightGpu, DrawMaterial, GfxRenderer, HudState, HudText, ParticleDraw};

use crate::math::{Mat4, Quat, Vec3};

use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;

use winit::event::{DeviceEvent, ElementState, WindowEvent};

use winit::event_loop::{ActiveEventLoop, EventLoop};

use winit::keyboard::{KeyCode, PhysicalKey};

use winit::window::{CursorGrabMode, WindowId};



pub struct Rock3DApp {

    title: String,

    width: u32,

    height: u32,

    backend: BackendKind,

}



impl Rock3DApp {

    pub fn new() -> Self {

        Self {

            title: "Rock 3D — Competitive Stone Throwing".into(),

            width: 1280,

            height: 720,

            backend: BackendKind::from_env(),

        }

    }



    pub fn with_title(mut self, title: impl Into<String>) -> Self {

        self.title = title.into();

        self

    }



    pub fn with_size(mut self, w: u32, h: u32) -> Self {

        self.width = w;

        self.height = h;

        self

    }



    pub fn run(self) {

        env_logger::init();

        log::info!("Rock 3D — Backend: {}", self.backend.name());

        let event_loop = EventLoop::new().expect("event loop");

        let mut app = Rock3DAppInner::new(self);

        event_loop.run_app(&mut app).expect("event loop failed");

    }

}



struct Rock3DAppInner {

    config: Rock3DApp,

    engine_window: Option<crate::graphics::renderer::EngineWindow>,

    state: Option<Rock3DState>,

    assets: Option<GpuAssetCache>,

    library: Option<AssetLibrary>,

    audio: Option<AudioEngine>,

    input: RockInput,

    mouse_delta: (f32, f32),

    cursor_grabbed: bool,

    hud: HudState,

    last_frame: Instant,

    next_frame: Instant,

    render_dt: f32,

}



impl Rock3DAppInner {

    fn new(config: Rock3DApp) -> Self {

        Self {

            config,

            engine_window: None,

            state: None,

            assets: None,

            library: None,

            audio: None,

            input: RockInput::default(),

            mouse_delta: (0.0, 0.0),

            cursor_grabbed: false,

            hud: HudState::default(),

            last_frame: Instant::now(),

            next_frame: Instant::now(),

            render_dt: 0.016,

        }

    }



    fn init(&mut self, event_loop: &ActiveEventLoop) {

        let mut ew = GfxRenderer::create(

            event_loop,

            &self.config.title,

            self.config.width,

            self.config.height,

            self.config.backend,

        )

        .expect("renderer");



        let library = load_rock_assets().expect("assets Rock 3D");

        let gpu_assets = GpuAssetCache::from_library(&library, &mut ew.renderer).expect("gpu");

        self.audio = AudioEngine::new().ok();



        self.state = Some(Rock3DState::new_with_assets(Some(&library)));

        self.library = Some(library);

        self.assets = Some(gpu_assets);

        self.engine_window = Some(ew);

    }



    fn update(&mut self, dt: f32) {

        let Some(state) = &mut self.state else { return };



        if self.input.restart {
            let lib = self.library.as_ref();
            *state = Rock3DState::new_with_assets(lib);
            self.input.restart = false;
        }



        state.update(dt, self.mouse_delta, &self.input);

        self.mouse_delta = (0.0, 0.0);

        self.input.scroll = 0.0;

        self.input.charging = false;

        self.input.stone_select = None;

    }



    fn render(&mut self) {

        let Some(ew) = &mut self.engine_window else { return };

        let Some(state) = &mut self.state else { return };

        let Some(assets) = &self.assets else { return };



        let size = ew.window.inner_size();

        let aspect = size.width as f32 / size.height.max(1) as f32;

        let camera = state.update_camera(self.render_dt, aspect);



        let lighting = state.day_night.lighting();

        let weather_fog = 0.35 + state.weather.fog_density * 0.4;
        let dn_gpu = DayNightGpu {
            sun_dir: lighting.sun_dir,
            horizon: lighting.horizon,
            zenith: lighting.zenith,
            fog_color: lighting.fog_color,
            night_factor: if lighting.is_night { 0.85 } else { 0.0 },
            fog_intensity: weather_fog,
        };



        let cam_pos = camera.position;

        let cam_fwd = camera.forward();



        ew.renderer.set_scene_time(state.scene_time);

        ew.renderer.set_day_night(dn_gpu);

        ew.renderer.begin_frame(lighting.clear);



        let _ = ew.renderer.begin_shadow_pass(&camera);

        for drawable in &state.world.drawables {

            if !should_draw(drawable.position, cam_pos, cam_fwd, &drawable.model_id) {

                continue;

            }

            if let Some(mesh) = assets.meshes.get(&drawable.model_id) {

                let _ = ew.renderer.draw_shadow(mesh, drawable.model_matrix());

            }

        }

        let _ = ew.renderer.end_shadow_pass();



        let _ = ew.renderer.begin_scene_pass(lighting.clear);

        let _ = ew.renderer.draw_sky(&camera);



        let _ = ew.renderer.draw(

            &assets.terrain,

            Mat4::IDENTITY,

            &camera,

            DrawMaterial::Terrain { tiling: 140.0 },

        );



        for drawable in &state.world.drawables {

            if !should_draw(drawable.position, cam_pos, cam_fwd, &drawable.model_id) {

                continue;

            }

            if let Some(mesh) = assets.meshes.get(&drawable.model_id) {

                let _ = ew.renderer.draw(mesh, drawable.model_matrix(), &camera, drawable.material);

            }

        }



        // Pedra em voo

        if let Some(rock) = &state.physics.active {

            if let Some(mesh) = assets.meshes.get("boulder_small") {

                let scale = (rock.stone.radius_m / 0.17).clamp(0.3, 1.8);

                let spin = Quat::from_rotation_y(state.scene_time * 4.0)

                    * Quat::from_rotation_x(state.scene_time * 3.0);

                let model = Mat4::from_scale_rotation_translation(

                    Vec3::splat(scale),

                    spin,

                    rock.body.position,

                );

                let _ = ew.renderer.draw(mesh, model, &camera, DrawMaterial::rock());

            }

            if rock.trail.len() >= 2 {

                let trail: Vec<[f32; 3]> = rock.trail.iter().map(|p| p.to_array()).collect();

                ew.renderer

                    .draw_line_strip(&camera, &trail, [0.6, 0.55, 0.5, 0.7]);

            }

        }



        // Pedra + mão em primeira pessoa

        if state.show_rock_in_hand() {

            let stone = state.throw_ctrl.selected_stone.stats();

            let hand_rock = assets
                .meshes
                .get("rock_hand")
                .or_else(|| assets.meshes.get("boulder_small"));
            if let Some(rock_mesh) = hand_rock {
                ew.renderer.draw_viewmodel_mat(
                    &camera,
                    rock_mesh,
                    state.rock_vm.rock_transform(stone.radius_m),
                    DrawMaterial::rock(),
                );
            }
            ew.renderer.draw_viewmodel_mat(
                &camera,
                &assets.fps_arm,
                state.rock_vm.hand_transform(),
                DrawMaterial::wood(),
            );

        }



        if state.trajectory_preview.len() >= 2 && !state.camera_ctrl.is_cinematic() {

            let pts: Vec<[f32; 3]> = state.trajectory_preview.iter().map(|p| p.to_array()).collect();

            ew.renderer

                .draw_line_strip(&camera, &pts, [0.2, 0.9, 0.4, 0.5]);

        }



        let world_parts: Vec<ParticleDraw> = state

            .particles

            .particles

            .iter()

            .map(|p| ParticleDraw {

                pos: p.pos.to_array(),

                size: p.size,

                alpha: p.life,

                kind: p.kind as f32,

            })

            .collect();

        ew.renderer.draw_world_particles(&camera, &world_parts);



        let _ = ew.renderer.end_scene_pass();



        // HUD animado

        let charge = state.throw_ctrl.charge;

        let pulse = (state.scene_time * 8.0).sin() * 0.5 + 0.5;

        self.hud.show_crosshair = !state.camera_ctrl.is_cinematic();

        self.hud.crosshair_spread = charge * 0.55;

        self.hud.muzzle_flash = if state.throw_ctrl.phase == ThrowPhase::Charging {

            charge * 0.35 + pulse * 0.15

        } else {

            0.0

        };

        self.hud.hit_flash = if state.scoring.combo > 1 {

            (state.scoring.combo as f32 * 0.08).min(0.9)

        } else {

            0.0

        };

        self.hud.day_hour = state.day_night.hour;

        self.hud.is_night = lighting.is_night;

        self.hud.hud_time = state.scene_time;

        self.hud.rock_hud = true;

        self.hud.force_bar = charge;

        self.hud.wind_strength = state.weather.wind_strength();

        self.hud.rock_speed = state.rock_speed();

        self.hud.combo_pulse = if state.scoring.combo > 1 {

            pulse * 0.8

        } else {

            0.0

        };

        self.hud.charge_pulse = if state.throw_ctrl.phase == ThrowPhase::Charging {

            pulse

        } else {

            0.0

        };

        self.hud.cinematic_active = state.camera_ctrl.is_cinematic();



        let cine_hint = if state.camera_ctrl.is_cinematic() {

            " | CINEMA: mouse orbita | scroll zoom"

        } else {

            ""

        };

        self.hud.net_label = format!(
            "Segure BOTAO DIREITO para forca{}",
            cine_hint
        );

        let rh = &state.hud;
        self.hud.hud_texts = vec![
            HudText {
                text: format!("FORCA {:.0}%", rh.force_pct),
                x: -0.95,
                y: -0.31,
                size: 0.022,
                color: [0.75, 0.9, 1.0, 1.0],
            },
            HudText {
                text: rh.stone_name.clone(),
                x: -0.95,
                y: -0.38,
                size: 0.02,
                color: [0.9, 0.85, 0.7, 0.95],
            },
            HudText {
                text: format!("VENTO {:.1} m/s {}", rh.wind_speed, rh.wind_dir),
                x: -0.11,
                y: 0.865,
                size: 0.02,
                color: [0.6, 1.0, 0.75, 1.0],
            },
            HudText {
                text: format!("SCORE {}  COMBO x{}", rh.score, rh.combo.max(1)),
                x: -0.95,
                y: 0.88,
                size: 0.022,
                color: [1.0, 0.92, 0.55, 1.0],
            },
            HudText {
                text: format!(
                    "ARREMESSOS {}  DIST {:.0}m",
                    rh.throws_left, rh.distance_m
                ),
                x: -0.95,
                y: 0.82,
                size: 0.019,
                color: [0.85, 0.88, 0.92, 0.92],
            },
            HudText {
                text: format!("{} | {}", rh.mode, rh.phase),
                x: -0.95,
                y: 0.76,
                size: 0.018,
                color: [0.65, 0.75, 0.9, 0.88],
            },
        ];
        if self.hud.cinematic_active && self.hud.rock_speed > 0.5 {
            self.hud.hud_texts.push(HudText {
                text: format!("{:.1} m/s", self.hud.rock_speed),
                x: 0.64,
                y: 0.84,
                size: 0.022,
                color: [1.0, 0.8, 0.35, 1.0],
            });
        }
        if rh.combo > 1 {
            self.hud.hud_texts.push(HudText {
                text: format!("COMBO x{}", rh.combo),
                x: 0.78,
                y: 0.72,
                size: 0.028,
                color: [0.3, 1.0, 0.5, 0.95],
            });
        }

        ew.renderer.draw_hud(&self.hud);



        let _ = ew.renderer.end_frame();

    }



    fn grab_cursor(&self, grab: bool) {

        if let Some(ew) = &self.engine_window {

            ew.window.set_cursor_grab(if grab {

                CursorGrabMode::Locked

            } else {

                CursorGrabMode::None

            }).ok();

            ew.window.set_cursor_visible(!grab);

        }

    }

}



impl ApplicationHandler for Rock3DAppInner {

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        if self.engine_window.is_none() {

            self.init(event_loop);

            self.grab_cursor(true);

            self.cursor_grabbed = true;

        }

    }



    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {

        match event {

            WindowEvent::CloseRequested => {

                if let Some(state) = &self.state {

                    state.persist();

                }

                event_loop.exit();

            }

            WindowEvent::Resized(size) => {

                if let Some(ew) = &mut self.engine_window {

                    ew.renderer.resize(size.width, size.height);

                }

            }

            WindowEvent::RedrawRequested => {

                let now = Instant::now();

                let dt = (now - self.last_frame).as_secs_f32().min(0.05);

                self.last_frame = now;

                self.render_dt = dt;

                self.update(dt);

                self.render();

                if let Some(ew) = &self.engine_window {

                    ew.window.request_redraw();

                }

            }

            WindowEvent::KeyboardInput { event, .. } => {

                let pressed = event.state == ElementState::Pressed;

                if let PhysicalKey::Code(key) = event.physical_key {

                    match key {

                        KeyCode::Escape => {

                            if self.cursor_grabbed {

                                self.grab_cursor(false);

                                self.cursor_grabbed = false;

                            } else {

                                event_loop.exit();

                            }

                        }

                        KeyCode::KeyW => self.input.forward = pressed,

                        KeyCode::KeyS => self.input.backward = pressed,

                        KeyCode::KeyA => self.input.left = pressed,

                        KeyCode::KeyD => self.input.right = pressed,

                        KeyCode::ShiftLeft => self.input.run = pressed,

                        KeyCode::KeyQ => self.input.spin_left = pressed,

                        KeyCode::KeyE => self.input.spin_right = pressed,

                        KeyCode::KeyR => self.input.spin_top = pressed,

                        KeyCode::KeyF => self.input.spin_bottom = pressed,

                        KeyCode::ArrowUp => self.input.pitch_up = pressed,

                        KeyCode::ArrowDown => self.input.pitch_down = pressed,

                        KeyCode::ArrowLeft => self.input.yaw_left = pressed,

                        KeyCode::ArrowRight => self.input.yaw_right = pressed,

                        KeyCode::Digit1 if pressed => self.input.stone_select = Some(0),

                        KeyCode::Digit2 if pressed => self.input.stone_select = Some(1),

                        KeyCode::Digit3 if pressed => self.input.stone_select = Some(2),

                        KeyCode::Digit4 if pressed => self.input.stone_select = Some(3),

                        KeyCode::Digit5 if pressed => self.input.stone_select = Some(4),

                        KeyCode::Digit6 if pressed => self.input.stone_select = Some(5),

                        KeyCode::Digit7 if pressed => self.input.stone_select = Some(6),

                        KeyCode::KeyN if pressed => self.input.restart = true,

                        _ => {}

                    }

                }

            }

            WindowEvent::MouseInput { state, button, .. } => {

                if button == winit::event::MouseButton::Left {

                    let pressed = state == ElementState::Pressed;

                    if pressed && !self.cursor_grabbed {

                        self.grab_cursor(true);

                        self.cursor_grabbed = true;

                    }

                }

                if button == winit::event::MouseButton::Right {

                    let pressed = state == ElementState::Pressed;

                    self.input.charging = pressed;

                    if !pressed {

                        if let Some(audio) = &self.audio {

                            let speed = self

                                .state

                                .as_ref()

                                .map(|s| 5.0 + s.throw_ctrl.charge * 40.0)

                                .unwrap_or(20.0);

                            RockAudio::on_throw(audio, speed);

                        }

                    }

                }

            }

            WindowEvent::MouseWheel { delta, .. } => {

                let scroll = match delta {

                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,

                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.02,

                };

                self.input.scroll = scroll;

            }

            _ => {}

        }

    }



    fn device_event(&mut self, _el: &ActiveEventLoop, _id: winit::event::DeviceId, event: DeviceEvent) {

        if let DeviceEvent::MouseMotion { delta } = event {

            if self.cursor_grabbed {

                self.mouse_delta.0 += delta.0 as f32;

                self.mouse_delta.1 += delta.1 as f32;

            }

        }

    }



    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {

        let now = Instant::now();

        if now < self.next_frame {

            return;

        }

        self.next_frame = now + Duration::from_secs_f64(1.0 / 60.0);

        if let Some(ew) = &self.engine_window {

            ew.window.request_redraw();

        }

    }

}


