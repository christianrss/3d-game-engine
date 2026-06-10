//! Engine Studio — viewport 3D, egui, gizmo, play mode e Lua hot-reload.

use crate::assets::sample_desert_height;
use crate::assets::{load_pack, GpuAssetCache, STUDIO_PACK};
use crate::editor::gizmo::TransformGizmo;
use crate::editor::scene_doc::{SceneDocument, SceneEntityKind};
use crate::editor::ui::{EditorUi, StudioActions};
use crate::game::build_desert;
use crate::game::world::GameWorld;
use crate::game::DayNightCycle;
use crate::graphics::{BackendKind, Camera, Color, DayNightGpu, DrawMaterial, GfxRenderer, HudState};
use crate::math::{Quat, Vec3};
use crate::scripting::LuaRuntime;
use std::path::PathBuf;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, WindowId};

pub struct EngineStudio {
    width: u32,
    height: u32,
    scene_path: PathBuf,
}

impl EngineStudio {
    pub fn new() -> Self {
        Self {
            width: 1440,
            height: 900,
            scene_path: PathBuf::from("scenes/default.scene.json"),
        }
    }

    pub fn with_scene_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.scene_path = path.into();
        self
    }

    pub fn run(self) {
        env_logger::init();
        log::info!("Engine Studio — editor de cenas");
        let event_loop = EventLoop::new().expect("event loop");
        let mut app = StudioApp::new(self);
        event_loop.run_app(&mut app).expect("studio loop");
    }
}

struct StudioApp {
    config: EngineStudio,
    window: Option<crate::graphics::renderer::EngineWindow>,
    world: Option<GameWorld>,
    assets: Option<GpuAssetCache>,
    scene_doc: SceneDocument,
    camera: Camera,
    orbit_yaw: f32,
    orbit_pitch: f32,
    orbit_dist: f32,
    orbit_target: Vec3,
    selected: usize,
    playing: bool,
    lua: Option<LuaRuntime>,
    day_night: DayNightCycle,
    scene_time: f32,
    hud: HudState,
    last_frame: Instant,
    mouse_look: bool,
    ui: Option<EditorUi>,
    gizmo: TransformGizmo,
    mouse_pos: (f32, f32),
    left_down: bool,
    hot_reload_timer: f32,
}

impl StudioApp {
    fn new(config: EngineStudio) -> Self {
        let scene_doc = SceneDocument::load(&config.scene_path).unwrap_or_default();
        let aspect = config.width as f32 / config.height as f32;
        let mut lua = LuaRuntime::new().ok();
        if let Some(rt) = &mut lua {
            for ent in &scene_doc.entities {
                if let Some(script) = &ent.script {
                    rt.watch_script(script);
                }
            }
        }
        Self {
            config,
            window: None,
            world: None,
            assets: None,
            scene_doc,
            camera: Camera::new(Vec3::new(0.0, 8.0, 18.0), aspect),
            orbit_yaw: 0.0,
            orbit_pitch: 0.35,
            orbit_dist: 22.0,
            orbit_target: Vec3::ZERO,
            selected: 0,
            playing: false,
            lua,
            day_night: DayNightCycle::default(),
            scene_time: 0.0,
            hud: HudState::default(),
            last_frame: Instant::now(),
            mouse_look: false,
            ui: None,
            gizmo: TransformGizmo::default(),
            mouse_pos: (0.0, 0.0),
            left_down: false,
            hot_reload_timer: 0.0,
        }
    }

    fn gizmo_origin(&self) -> Option<Vec3> {
        let ent = self.scene_doc.entities.get(self.selected)?;
        if matches!(
            ent.kind,
            SceneEntityKind::Terrain | SceneEntityKind::Light | SceneEntityKind::Empty
        ) {
            return None;
        }
        let mut p = ent.position_vec();
        if ent.kind == SceneEntityKind::Target || ent.kind == SceneEntityKind::Cube {
            let gy = sample_desert_height(p.x, p.z);
            if ent.position[1] < gy + 0.5 {
                p.y = gy;
            }
        }
        Some(p)
    }

    fn rebuild_world(&mut self) {
        let mut world = GameWorld::default();
        let _ = build_desert(&mut world);
        for ent in &self.scene_doc.entities {
            if !ent.enabled {
                continue;
            }
            match ent.kind {
                SceneEntityKind::Target => {
                    let pos = ent.position_vec();
                    let gy = sample_desert_height(pos.x, pos.z);
                    world.add_target(Vec3::new(pos.x, gy, pos.z), 100, ent.scale);
                }
                SceneEntityKind::Cube => {
                    let pos = ent.position_vec();
                    world.add_drawable(crate::game::world::Drawable {
                        model_id: "cube".into(),
                        position: pos,
                        rotation: Quat::from_rotation_y(ent.rotation_y),
                        scale: Vec3::splat(ent.scale),
                        material: DrawMaterial::wood(),
                        target_id: None,
                    });
                }
                _ => {}
            }
        }
        self.world = Some(world);
        if let Some(ui) = &mut self.ui {
            ui.push_log("Mundo reconstruído.");
        }
    }

    fn update_orbit_camera(&mut self) {
        let yaw = self.orbit_yaw;
        let pitch = self.orbit_pitch.clamp(0.05, 1.45);
        let dist = self.orbit_dist;
        let t = self.orbit_target;
        self.camera.position = Vec3::new(
            t.x + dist * pitch.sin() * yaw.sin(),
            t.y + dist * pitch.cos(),
            t.z + dist * pitch.sin() * yaw.cos(),
        );
        self.camera.yaw = (t.x - self.camera.position.x).atan2(t.z - self.camera.position.z);
        self.camera.pitch = ((t.y - self.camera.position.y) / dist).asin();
    }

    fn start_play(&mut self) {
        self.playing = true;
        let paths: Vec<_> = self
            .scene_doc
            .entities
            .iter()
            .filter_map(|e| e.script.clone())
            .collect();
        if let Some(lua) = &mut self.lua {
            lua.set_playing(true);
            for path in &paths {
                if let Err(e) = lua.load_file(path) {
                    log::error!("Script {path}: {e}");
                    if let Some(ui) = &mut self.ui {
                        ui.push_log(format!("Erro script {path}: {e}"));
                    }
                }
            }
            let _ = lua.call_start();
        }
        if let Some(ui) = &mut self.ui {
            ui.push_log("▶ Play mode");
        }
    }

    fn stop_play(&mut self) {
        self.playing = false;
        if let Some(lua) = &mut self.lua {
            let _ = lua.call_stop();
            lua.set_playing(false);
        }
        if let Some(ui) = &mut self.ui {
            ui.push_log("■ Edit mode");
        }
    }

    fn save_scene(&mut self) {
        if let Err(e) = self.scene_doc.save(&self.config.scene_path) {
            log::error!("Falha ao salvar cena: {e}");
            if let Some(ui) = &mut self.ui {
                ui.push_log(format!("Erro ao salvar: {e}"));
            }
        } else {
            log::info!("Cena salva em {:?}", self.config.scene_path);
            if let Some(ui) = &mut self.ui {
                ui.push_log("Cena salva.");
            }
        }
    }

    fn apply_actions(&mut self, actions: StudioActions) {
        if actions.play {
            self.start_play();
        }
        if actions.stop {
            self.stop_play();
        }
        if actions.save {
            self.save_scene();
        }
        if actions.add_cube {
            self.scene_doc.add_entity(SceneEntityKind::Cube);
            self.rebuild_world();
        }
        if actions.add_target {
            self.scene_doc.add_entity(SceneEntityKind::Target);
            self.rebuild_world();
        }
        if actions.remove_selected {
            if self.scene_doc.entities.len() > 1 {
                self.scene_doc.entities.remove(self.selected);
                self.selected = self.selected.min(self.scene_doc.entities.len().saturating_sub(1));
                self.rebuild_world();
            }
        }
        if actions.rebuild_world {
            self.rebuild_world();
        }
        if actions.script_changed {
            if let Some(ent) = self.scene_doc.entities.get(self.selected) {
                if let Some(path) = &ent.script {
                    if let Some(lua) = &mut self.lua {
                        lua.watch_script(path);
                    }
                }
            }
            if let Some(ui) = &mut self.ui {
                ui.push_log("Script aplicado à entidade.");
            }
        }
        if actions.reload_script {
            if let Some(ent) = self.scene_doc.entities.get(self.selected) {
                if let Some(path) = ent.script.clone() {
                    if let Some(lua) = &mut self.lua {
                        let _ = lua.reload_file(&path);
                        if let Some(ui) = &mut self.ui {
                            ui.push_log(format!("Recarregado: {path}"));
                        }
                    }
                }
            }
        }
    }

    fn poll_hot_reload(&mut self, dt: f32) {
        self.hot_reload_timer += dt;
        if self.hot_reload_timer < 0.4 {
            return;
        }
        self.hot_reload_timer = 0.0;
        if let Some(lua) = &mut self.lua {
            match lua.hot_reload_poll() {
                Ok(changed) => {
                    for path in changed {
                        if let Some(ui) = &mut self.ui {
                            ui.push_log(format!("♻ Hot-reload: {}", path.display()));
                        }
                    }
                }
                Err(e) => {
                    if let Some(ui) = &mut self.ui {
                        ui.push_log(format!("Hot-reload erro: {e}"));
                    }
                }
            }
        }
    }

    fn handle_gizmo_input(&mut self, width: f32, height: f32) {
        if self.playing {
            return;
        }
        if self.gizmo_origin().is_none() {
            return;
        }
        if self.ui.as_ref().is_some_and(|ui| ui.wants_pointer) {
            return;
        }

        if self.left_down {
            if let Some((pos, rot)) = self.gizmo.drag_update(
                self.mouse_pos,
                &self.camera,
                width,
                height,
            ) {
                if let Some(ent) = self.scene_doc.entities.get_mut(self.selected) {
                    ent.position = pos;
                    ent.rotation_y = rot;
                }
            }
        }
    }

    fn render_frame(&mut self, dt: f32) {
        self.scene_time += dt;
        let speed = if self.playing { 1.0 } else { 0.15 };
        self.day_night.update(dt * speed);
        self.poll_hot_reload(dt);

        if self.playing {
            if let Some(lua) = &mut self.lua {
                lua.set_time(self.scene_time);
                let _ = lua.call_update(dt);
                let msgs = lua.drain_messages();
                if let Some(ui) = &mut self.ui {
                    for msg in msgs {
                        ui.push_log(format!("[Lua] {msg}"));
                    }
                }
            }
        }

        self.update_orbit_camera();

        let gizmo_origin = self.gizmo_origin();
        let win_size = self
            .window
            .as_ref()
            .map(|ew| ew.window.inner_size())
            .unwrap_or(winit::dpi::PhysicalSize::new(1, 1));
        let win_w = win_size.width as f32;
        let win_h = win_size.height as f32;
        self.handle_gizmo_input(win_w, win_h);

        let lighting = self.day_night.lighting();
        let night_factor = (1.0 - lighting.sun_dir[1].clamp(0.0, 1.0)).powf(0.65);
        let dn = DayNightGpu {
            sun_dir: lighting.sun_dir,
            horizon: lighting.horizon,
            zenith: lighting.zenith,
            fog_color: lighting.fog_color,
            night_factor,
            fog_intensity: 0.4,
        };

        {
            let Some(ew) = &mut self.window else { return };
            let Some(world) = &self.world else { return };
            let Some(assets) = &self.assets else { return };

            ew.renderer.set_day_night(dn);
            ew.renderer.set_scene_time(self.scene_time);
            ew.renderer.sand_update(dt, Vec3::new(2.5, 0.0, 0.8));

            let _ = ew.renderer.begin_shadow_pass(&self.camera);
            for d in &world.drawables {
                if d.model_id == "terrain" {
                    continue;
                }
                let mesh = assets.mesh(&d.model_id).unwrap_or(&assets.terrain);
                let _ = ew.renderer.draw_shadow(mesh, d.model_matrix());
            }
            let _ = ew.renderer.end_shadow_pass();

            let clear = Color::rgb(0.52, 0.68, 0.92);
            let _ = ew.renderer.begin_scene_pass(clear);
            let _ = ew.renderer.draw_sky(&self.camera);

            for d in &world.drawables {
                let mesh = if d.model_id == "terrain" {
                    &assets.terrain
                } else {
                    assets.mesh(&d.model_id).unwrap_or(&assets.terrain)
                };
                let _ = ew.renderer.draw(mesh, d.model_matrix(), &self.camera, d.material);
            }

            if !self.playing {
                if let Some(origin) = gizmo_origin {
                    TransformGizmo::draw_axes(
                        &mut ew.renderer,
                        &self.camera,
                        origin,
                        self.gizmo.active_axis,
                    );
                }
            }

            ew.renderer.sand_draw(&self.camera);
            let _ = ew.renderer.end_scene_pass();
            ew.renderer.draw_hud(&self.hud);
        }

        let (actions, egui_output) = {
            let Some(ew) = self.window.as_ref() else { return };
            let Some(ui) = &mut self.ui else { return };
            ui.draw_panels(
                &ew.window,
                &mut self.scene_doc,
                &mut self.selected,
                self.playing,
                &self.gizmo,
                &self.config.scene_path,
                dt,
            )
        };
        self.apply_actions(actions);

        if let (Some(ew), Some(ui), Some(output)) =
            (&mut self.window, &mut self.ui, egui_output)
        {
            let size = ew.window.inner_size();
            ui.paint(output, size.width, size.height);
            let _ = ew.renderer.end_frame();
            ew.window.request_redraw();
        }
    }
}

impl ApplicationHandler for StudioApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let mut ew = GfxRenderer::create(
            event_loop,
            "Engine Studio",
            self.config.width,
            self.config.height,
            BackendKind::from_env(),
        )
        .expect("renderer");
        let size = ew.window.inner_size();
        self.camera.resize(size.width, size.height);

        let lib = load_pack(STUDIO_PACK).expect("assets studio");
        let assets = GpuAssetCache::from_library(&lib, &mut ew.renderer).expect("gpu assets");
        self.rebuild_world();

        let mut ui = EditorUi::new(&ew.window);
        #[cfg(feature = "opengl")]
        {
            let gl = ew.renderer.create_glow_context();
            ui.init_glow(gl);
        }
        #[cfg(not(feature = "opengl"))]
        {
            ui.init_glow(());
            ui.push_log("Aviso: UI do Studio requer backend OpenGL (egui_glow).");
        }
        ui.push_log("Engine Studio iniciado.");
        self.ui = Some(ui);
        self.window = Some(ew);
        self.assets = Some(assets);
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(ew), Some(ui)) = (&self.window, &mut self.ui) {
            let response = ui.on_event(&ew.window, &event);
            if response.consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                self.stop_play();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(ew) = &mut self.window {
                    ew.renderer.resize(size.width, size.height);
                    self.camera.resize(size.width, size.height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = (position.x as f32, position.y as f32);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::F5) => self.start_play(),
                    PhysicalKey::Code(KeyCode::F6) => self.stop_play(),
                    PhysicalKey::Code(KeyCode::KeyS) => self.save_scene(),
                    PhysicalKey::Code(KeyCode::KeyW) => self.gizmo.set_mode_translate(),
                    PhysicalKey::Code(KeyCode::KeyE) => self.gizmo.set_mode_rotate(),
                    PhysicalKey::Code(KeyCode::KeyN) => {
                        self.scene_doc.add_entity(SceneEntityKind::Cube);
                        self.rebuild_world();
                    }
                    PhysicalKey::Code(KeyCode::Escape) => {
                        self.mouse_look = false;
                        self.gizmo.end_drag();
                        if let Some(ew) = &self.window {
                            let _ = ew.window.set_cursor_grab(CursorGrabMode::None);
                            ew.window.set_cursor_visible(true);
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(ui) = &self.ui {
                    if ui.wants_pointer {
                        return;
                    }
                }
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.02,
                };
                self.orbit_dist = (self.orbit_dist - scroll * 1.5).clamp(4.0, 80.0);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Right && state == ElementState::Pressed {
                    if let Some(ui) = &self.ui {
                        if !ui.wants_pointer {
                            self.mouse_look = true;
                            if let Some(ew) = &self.window {
                                let _ = ew.window.set_cursor_grab(CursorGrabMode::Locked);
                                ew.window.set_cursor_visible(false);
                            }
                        }
                    }
                }
                if button == MouseButton::Left {
                    self.left_down = state == ElementState::Pressed;
                    if state == ElementState::Pressed && !self.playing {
                        if let Some(ui) = &self.ui {
                            if !ui.wants_pointer {
                                if let Some(origin) = self.gizmo_origin() {
                                    let s = self.window.as_ref().unwrap().window.inner_size();
                                    let axis = self.gizmo.pick_axis(
                                        self.mouse_pos,
                                        origin,
                                        &self.camera,
                                        s.width as f32,
                                        s.height as f32,
                                    );
                                    if let Some(ent) = self.scene_doc.entities.get(self.selected) {
                                        self.gizmo.begin_drag(
                                            axis,
                                            self.mouse_pos,
                                            ent.position,
                                            ent.rotation_y,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    if state == ElementState::Released {
                        if self.gizmo.is_dragging() {
                            self.gizmo.end_drag();
                            self.rebuild_world();
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let dt = self.last_frame.elapsed().as_secs_f32().min(0.05);
                self.last_frame = Instant::now();
                self.render_frame(dt);
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
        if !self.mouse_look {
            return;
        }
        if let Some(ui) = &self.ui {
            if ui.wants_pointer {
                return;
            }
        }
        if let DeviceEvent::MouseMotion { delta } = event {
            self.orbit_yaw += delta.0 as f32 * 0.005;
            self.orbit_pitch = (self.orbit_pitch - delta.1 as f32 * 0.005).clamp(0.05, 1.45);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ew) = &self.window {
            ew.window.request_redraw();
        }
    }
}
