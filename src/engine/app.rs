//! # EngineApp — Loop Principal (winit 0.30)

use crate::game::{try_shoot, InputState, Player, SceneBuilder, Score};
use crate::game::GameWorld;
use crate::graphics::{cylinder, plane, sphere, BackendKind, Color, GfxRenderer, Mesh, MeshCache};
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
    mesh_cache: MeshCache,
    cpu_meshes: Option<CpuMeshes>,
    last_frame: Instant,
    shoot_cooldown: f32,
    victory: bool,
    base_title: String,
}

struct CpuMeshes {
    plane: Mesh,
    sphere: Mesh,
    cylinder: Mesh,
}

impl GameApp {
    fn new(config: EngineApp) -> Self {
        Self {
            base_title: config.title.clone(),
            config,
            engine_window: None,
            world: None,
            player: None,
            score: Score::default(),
            input: InputState::default(),
            mesh_cache: MeshCache::default(),
            cpu_meshes: None,
            last_frame: Instant::now(),
            shoot_cooldown: 0.0,
            victory: false,
        }
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let ew = GfxRenderer::create(
            event_loop,
            &self.config.title,
            self.config.width,
            self.config.height,
            self.config.backend,
        )
        .expect("Falha ao criar renderer");

        let (world, player) = self.config.scene.clone().build();
        self.world = Some(world);
        self.player = Some(player);
        self.cpu_meshes = Some(CpuMeshes {
            plane: plane(1.0, Color::SAND),
            sphere: sphere(1.0, Color::WHITE, 16, 12),
            cylinder: cylinder(1.0, 1.0, Color::WHITE, 16),
        });
        self.engine_window = Some(ew);
    }

    fn update(&mut self, dt: f32) {
        if self.victory {
            return;
        }

        let player = self.player.as_mut().unwrap();
        player.update(&self.input, dt);

        self.shoot_cooldown = (self.shoot_cooldown - dt).max(0.0);
        if self.input.shoot && self.shoot_cooldown <= 0.0 && self.input.cursor_grabbed {
            self.shoot_cooldown = 0.25;
            let ew = self.engine_window.as_ref().unwrap();
            let aspect = ew.window.inner_size().width as f32
                / ew.window.inner_size().height.max(1) as f32;
            let cam = player.to_camera(aspect);
            let world = self.world.as_mut().unwrap();
            if let Some(pts) = try_shoot(world, &mut self.score, cam.position, cam.forward(), 100.0) {
                log::info!("Acerto! +{pts}");
            }
        }

        if self.world.as_ref().unwrap().all_targets_destroyed() {
            self.victory = true;
            log::info!("VITÓRIA!");
        }

        self.input.reset_frame();
    }

    fn render(&mut self) {
        let ew = self.engine_window.as_mut().unwrap();
        let player = self.player.as_ref().unwrap();
        let world = self.world.as_ref().unwrap();
        let cpu = self.cpu_meshes.as_ref().unwrap();

        let size = ew.window.inner_size();
        let camera = player.to_camera(size.width as f32 / size.height.max(1) as f32);

        ew.renderer.begin_frame(Color::SKY);

        for drawable in &world.drawables {
            let mesh = match drawable.mesh_name.as_str() {
                "plane" => &cpu.plane,
                "sphere" => &cpu.sphere,
                "cylinder" => &cpu.cylinder,
                _ => &cpu.sphere,
            };
            if let Ok(gpu) = self
                .mesh_cache
                .get_or_upload(&mut ew.renderer, &drawable.mesh_name, mesh)
            {
                let _ = ew.renderer.draw(gpu, drawable.model_matrix(), &camera);
            }
        }

        let _ = ew.renderer.end_frame();

        let title = if self.victory {
            format!("VITÓRIA! | Pontos: {} | {:.0}%", self.score.total, self.score.accuracy())
        } else {
            format!(
                "Pontos: {} | {:.0}% | {} | WASD+mover, Mouse+olhar, Clique+atirar, ESC+soltar",
                self.score.total, self.score.accuracy(), self.base_title
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

            WindowEvent::Resized(size) => {
                ew.renderer.resize(size.width, size.height);
            }

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

fn set_cursor_grab(window: &Window, grab: bool) {
    if grab {
        let _ = window.set_cursor_grab(CursorGrabMode::Locked);
        window.set_cursor_visible(false);
    } else {
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        window.set_cursor_visible(true);
    }
}
