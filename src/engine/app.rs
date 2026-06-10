//! # EngineApp — Loop Principal (winit 0.30)

use crate::assets::{AssetLibrary, GpuAssetCache};
use crate::audio::AudioEngine;
use crate::game::{
    InputState, ParticleSystem, Player, ProjectileSystem, SceneBuilder, Score, ViewModelAnimator,
};
use crate::game::GameWorld;
use crate::graphics::{BackendKind, Color, GfxRenderer, HudState, ParticleDraw};
use crate::math::Vec3;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

pub struct EngineApp {
    title: String,
    width: u32,
    height: u32,
    backend: BackendKind,
    scene: SceneBuilder,
}

impl EngineApp {
    pub fn new() -> Self {
        Self {
            title: "Desert Shooter Engine".into(),
            width: 1280,
            height: 720,
            backend: BackendKind::from_env(),
            scene: SceneBuilder::new().with_desert_map(),
        }
    }

    pub fn with_window_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_scene(mut self, scene: SceneBuilder) -> Self {
        self.scene = scene;
        self
    }

    pub fn with_backend(mut self, backend: BackendKind) -> Self {
        self.backend = backend;
        self
    }

    pub fn run(self) {
        env_logger::init();
        log::info!("Backend gráfico: {}", self.backend.name());

        let event_loop = EventLoop::new().expect("Falha ao criar event loop");
        let mut app = GameApp::new(self);
        event_loop.run_app(&mut app).expect("Event loop falhou");
    }
}

struct GameApp {
    config: EngineApp,
    engine_window: Option<crate::graphics::renderer::EngineWindow>,
    world: Option<GameWorld>,
    player: Option<Player>,
    score: Score,
    input: InputState,
    assets: Option<GpuAssetCache>,
    viewmodel: ViewModelAnimator,
    projectiles: ProjectileSystem,
    particles: ParticleSystem,
    audio: Option<AudioEngine>,
    hud: HudState,
    last_frame: Instant,
    shoot_cooldown: f32,
    victory: bool,
}

impl GameApp {
    fn new(config: EngineApp) -> Self {
        Self {
            config,
            engine_window: None,
            world: None,
            player: None,
            score: Score::default(),
            input: InputState::default(),
            assets: None,
            viewmodel: ViewModelAnimator::default(),
            projectiles: ProjectileSystem::default(),
            particles: ParticleSystem::default(),
            audio: None,
            hud: HudState::default(),
            last_frame: Instant::now(),
            shoot_cooldown: 0.0,
            victory: false,
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
        .expect("Falha ao criar renderer");

        let library = AssetLibrary::load().expect("Falha ao carregar assets CC0");
        self.viewmodel = ViewModelAnimator::with_muzzle(library.viewmodel_muzzle);
        let gpu_assets = GpuAssetCache::from_library(&library, &mut ew.renderer)
            .expect("Falha ao enviar assets para GPU");

        self.audio = AudioEngine::new().ok();
        if let Some(audio) = &self.audio {
            audio.play_wind_ambient();
        }

        let (world, player) = self.config.scene.clone().build();
        self.assets = Some(gpu_assets);
        self.world = Some(world);
        self.player = Some(player);
        self.engine_window = Some(ew);
    }

    fn update(&mut self, dt: f32) {
        self.hud.muzzle_flash = (self.hud.muzzle_flash - dt * 6.0).max(0.0);
        self.hud.hit_flash = (self.hud.hit_flash - dt * 3.0).max(0.0);
        self.particles.update(dt);

        if self.victory {
            return;
        }

        let player = self.player.as_mut().unwrap();
        let mouse_delta = self.input.mouse_delta;
        player.update(&self.input, dt);
        self.viewmodel
            .update(dt, player.is_moving, player.is_sprinting, mouse_delta);

        let ew = self.engine_window.as_ref().unwrap();
        let aspect =
            ew.window.inner_size().width as f32 / ew.window.inner_size().height.max(1) as f32;
        let cam = player.to_camera(aspect);
        let muzzle_world = self.viewmodel.muzzle_world(&cam);
        self.projectiles
            .update_trajectory_preview(muzzle_world, cam.forward());

        let hit_positions = self
            .projectiles
            .update(dt, self.world.as_mut().unwrap(), &mut self.score);
        for hit_pos in hit_positions {
            self.hud.hit_flash = 1.0;
            if let Some(audio) = &self.audio {
                audio.play_hit();
            }
            self.particles.emit_hit_dust(hit_pos);
        }

        self.shoot_cooldown = (self.shoot_cooldown - dt).max(0.0);
        if self.input.shoot && self.shoot_cooldown <= 0.0 && self.input.cursor_grabbed {
            self.shoot_cooldown = 0.25;
            self.hud.muzzle_flash = 1.0;
            self.viewmodel.on_shoot();

            self.particles
                .emit_muzzle_smoke(self.viewmodel.muzzle_vm_space(), Vec3::NEG_Z);

            if let Some(audio) = &self.audio {
                audio.play_gunshot();
            }

            self.projectiles.spawn(muzzle_world, cam.forward());
        }

        if self.world.as_ref().unwrap().all_targets_destroyed() {
            self.victory = true;
            log::info!("VITÓRIA!");
        }

        self.hud.show_crosshair = self.input.cursor_grabbed;
        self.hud.crosshair_spread = if player.is_sprinting {
            1.0
        } else if player.is_moving {
            0.45
        } else {
            0.1
        };
        self.input.reset_frame();
    }

    fn render(&mut self) {
        let ew = self.engine_window.as_mut().unwrap();
        let player = self.player.as_ref().unwrap();
        let world = self.world.as_ref().unwrap();
        let assets = self.assets.as_ref().unwrap();

        let size = ew.window.inner_size();
        let camera = player.to_camera(size.width as f32 / size.height.max(1) as f32);
        let vm = self.viewmodel.transform();

        ew.renderer.begin_frame(Color::SKY);
        let _ = ew.renderer.begin_shadow_pass(&camera);
        for drawable in &world.drawables {
            if let Some(gpu) = resolve_mesh(drawable, assets) {
                let _ = ew.renderer.draw_shadow(gpu, drawable.model_matrix());
            }
        }
        let _ = ew.renderer.end_shadow_pass();
        let _ = ew.renderer.begin_scene_pass(Color::SKY);
        let _ = ew.renderer.draw_sky(&camera);

        for drawable in &world.drawables {
            if let Some(gpu) = resolve_mesh(drawable, assets) {
                let _ = ew.renderer.draw(
                    gpu,
                    drawable.model_matrix(),
                    &camera,
                    drawable.material,
                );
            }
        }

        // Trajetória balística prevista
        if !self.projectiles.trajectory.is_empty() {
            let traj: Vec<[f32; 3]> = self
                .projectiles
                .trajectory
                .iter()
                .map(|p| p.to_array())
                .collect();
            ew.renderer
                .draw_line_strip(&camera, &traj, [1.0, 0.75, 0.2, 0.55]);
        }

        // Projéteis em voo — trilha + cabeça
        for bullet in &self.projectiles.active {
            if bullet.trail.len() >= 2 {
                let trail: Vec<[f32; 3]> = bullet.trail.iter().map(|p| p.to_array()).collect();
                ew.renderer
                    .draw_line_strip(&camera, &trail, [1.0, 0.9, 0.35, 0.85]);
            }
        }

        ew.renderer.draw_viewmodel(&camera, &assets.viewmodel, vm);

        let mut vm_parts = Vec::new();
        let mut world_parts = Vec::new();
        for p in &self.particles.particles {
            let d = ParticleDraw {
                pos: p.pos.to_array(),
                size: p.size,
                alpha: p.life,
            };
            if p.kind == 0 {
                vm_parts.push(d);
            } else {
                world_parts.push(d);
            }
        }
        for bullet in &self.projectiles.active {
            world_parts.push(ParticleDraw {
                pos: bullet.pos.to_array(),
                size: 0.08,
                alpha: 1.0,
            });
        }
        ew.renderer.draw_particles(&camera, &vm_parts, vm);
        ew.renderer.draw_world_particles(&camera, &world_parts);

        let _ = ew.renderer.end_scene_pass();
        ew.renderer.draw_hud(&self.hud);
        let _ = ew.renderer.end_frame();

        let remaining = world.alive_targets();
        let title = if self.victory {
            format!(
                "VITORIA! Pontos: {} | Precisao: {:.0}%",
                self.score.total,
                self.score.accuracy()
            )
        } else {
            format!(
                "Pontos: {} | Alvos: {} | {:.0}% precisao",
                self.score.total,
                remaining,
                self.score.accuracy()
            )
        };
        ew.window.set_title(&title);
    }
}

impl ApplicationHandler for GameApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine_window.is_none() {
            self.init(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(ew) = self.engine_window.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => ew.renderer.resize(size.width, size.height),
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                if let PhysicalKey::Code(code) = event.physical_key {
                    self.input.on_key(code, pressed);
                    if code == KeyCode::Escape && pressed {
                        self.input.cursor_grabbed = !self.input.cursor_grabbed;
                        set_cursor_grab(&ew.window, self.input.cursor_grabbed);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.on_mouse_button(button, state);
                if self.input.cursor_grabbed {
                    set_cursor_grab(&ew.window, true);
                }
            }
            WindowEvent::RedrawRequested => {
                let dt = self.last_frame.elapsed().as_secs_f32().min(0.05);
                self.last_frame = Instant::now();
                self.update(dt);
                self.render();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.input.on_mouse_delta(delta.0 as f32, delta.1 as f32);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ew) = &self.engine_window {
            ew.window.request_redraw();
        }
    }
}

fn resolve_mesh<'a>(
    drawable: &crate::game::world::Drawable,
    assets: &'a GpuAssetCache,
) -> Option<&'a crate::graphics::GpuMesh> {
    if drawable.model_id == "terrain" {
        Some(&assets.terrain)
    } else {
        assets.mesh(&drawable.model_id)
    }
}

fn set_cursor_grab(window: &Window, grab: bool) {
    if grab {
        let _ = window.set_cursor_grab(CursorGrabMode::Locked);
        window.set_cursor_visible(false);
    } else {
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        window.set_cursor_visible(true);
    }
}
