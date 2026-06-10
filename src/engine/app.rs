//! # EngineApp — Loop Principal (winit 0.30)



use crate::assets::GpuAssetCache;

use crate::audio::AudioEngine;

use crate::game::{

    aim_build, aim_remove_key, sync_block_drawables, BlockGrid, BlockKind, DayNightCycle,
    HotbarSlot, InputState, Inventory, ParticleSystem, PlacedBlock,
    Player, ProjectileSystem, SandSimulator, SceneBuilder, CollisionWorld, Ecosystem, Score,
    ViewModelAnimator, WeaponKind, WeaponState, MAX_BUILD_LEVEL,

};

use crate::game::{CreatureKind, ProjectileParams, BULLET_SPEED, PLAYER_ENTITY};

use crate::game::{
    aim_block_pos, should_draw, should_reflect, should_shadow, sync_ecosystem_drawables,
    WorldSimulation, BEACON_RANGE, RADAR_RANGE,
};

use crate::game::GameWorld;

use crate::graphics::{BackendKind, DayNightGpu, DrawMaterial, GfxRenderer, HudState, ParticleDraw};

use crate::math::Vec3;

use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;

use winit::event::{DeviceEvent, ElementState, WindowEvent};

use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};

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

            title: "Mega Deserto — Ecossistema Vivo".into(),

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

    sand: SandSimulator,

    particles: ParticleSystem,

    scene_time: f32,

    audio: Option<AudioEngine>,

    hud: HudState,

    last_frame: Instant,
    next_frame: Instant,

    victory: bool,

    inventory: Inventory,

    ecosystem: Ecosystem,
    collision: CollisionWorld,

    blocks: BlockGrid,

    weapon: WeaponState,

    build_yaw: u8,

    build_level: i32,

    gunshot_notify: Option<Vec3>,

    day_night: DayNightCycle,

    sim: WorldSimulation,

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

            sand: SandSimulator::default(),

            particles: ParticleSystem::default(),

            scene_time: 0.0,

            audio: None,

            hud: HudState::default(),

            last_frame: Instant::now(),
            next_frame: Instant::now(),

            victory: false,

            inventory: Inventory::starter_ranch(),

            ecosystem: Ecosystem::default(),
            collision: CollisionWorld::default(),

            blocks: BlockGrid::default(),

            weapon: WeaponState::default(),

            build_yaw: 0,

            build_level: 0,

            gunshot_notify: None,

            day_night: DayNightCycle::default(),

            sim: WorldSimulation::default(),

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



        let library = crate::games::desert_shooter::load_assets()
            .expect("Falha ao carregar assets do Desert Shooter");

        self.viewmodel = ViewModelAnimator::with_muzzle(library.viewmodel_muzzle);

        let gpu_assets = GpuAssetCache::from_library(&library, &mut ew.renderer)

            .expect("Falha ao enviar assets para GPU");



        self.audio = AudioEngine::new().ok();

        if let Some(audio) = &self.audio {

            audio.play_wind_ambient();

        }



        let (mut world, mut player, collision) = self.config.scene.clone().build();

        self.collision = collision;

        self.ecosystem.populate_desert();
        if self.sim.try_load_into(&mut self.blocks, &mut self.inventory, &mut player) {
            log::info!("Save carregado — construções e cidades restauradas");
        }

        sync_block_drawables(&mut world, &self.blocks);
        sync_ecosystem_drawables(&mut world, &self.ecosystem, player.position);
        self.sim.sync_drawables(&mut world);

        self.assets = Some(gpu_assets);

        self.world = Some(world);

        self.player = Some(player);

        self.engine_window = Some(ew);

    }



    fn update(&mut self, dt: f32) {

        self.scene_time += dt;

        self.day_night.update(dt);

        self.hud.muzzle_flash = (self.hud.muzzle_flash - dt * 6.0).max(0.0);

        self.hud.hit_flash = (self.hud.hit_flash - dt * 3.0).max(0.0);

        self.particles.update(dt);



        if self.victory {

            return;

        }



        let player = self.player.as_mut().unwrap();

        let mouse_delta = self.input.mouse_delta;

        player.update(&self.input, dt, &self.blocks, &self.collision);

        self.viewmodel

            .update(dt, player.is_moving, player.is_sprinting, mouse_delta);



        let ew = self.engine_window.as_ref().unwrap();

        let aspect =

            ew.window.inner_size().width as f32 / ew.window.inner_size().height.max(1) as f32;

        let cam = player.to_camera(aspect);

        let muzzle_world = self.viewmodel.muzzle_world(&cam);

        let preview_speed = self
            .weapon
            .active
            .projectile_speed()
            .unwrap_or(BULLET_SPEED);
        self.projectiles
            .update_trajectory_preview(muzzle_world, cam.forward(), preview_speed);



        self.sand

            .update(dt, player.position, player.velocity);

        if let Some(ew) = self.engine_window.as_mut() {

            for (pos, vel, count) in self.sand.drain_emits() {

                ew.renderer.sand_emit(pos, vel, count);

            }

            ew.renderer

                .sand_update(dt, Vec3::new(2.2, 0.0, -0.9));

        }



        let (hit_positions, kills) = self.projectiles.update(
            dt,
            self.world.as_mut().unwrap(),
            &mut self.score,
            &mut self.ecosystem,
        );

        for (_, kind) in &kills {
            match kind {
                CreatureKind::Bird => self.inventory.add_bird_meat(1),
                CreatureKind::Sheep => self.inventory.add_loot(2, 1),
                CreatureKind::Goat => self.inventory.add_loot(1, 1),
                CreatureKind::Camel => self.inventory.add_loot(1, 2),
                _ => {}
            }
        }

        for hit_pos in hit_positions {

            self.hud.hit_flash = 1.0;

            if let Some(audio) = &self.audio {

                audio.play_hit();

            }

            self.particles.emit_hit_dust(hit_pos);

            self.sand.emit_impact(hit_pos);

        }

        if let Some(ew) = self.engine_window.as_mut() {

            for (pos, vel, count) in self.sand.drain_emits() {

                ew.renderer.sand_emit(pos, vel, count);

            }

        }



        if let Some(slot) = self.input.hotbar_select.and_then(HotbarSlot::from_index) {
            if slot.placeable() || matches!(slot, HotbarSlot::Wool | HotbarSlot::Mutton) {
                self.inventory.hotbar = slot;
            }
        }

        if self.input.cycle_weapon {
            self.weapon.cycle();
        }
        if self.input.rotate_build {
            self.build_yaw = (self.build_yaw + 1) % 4;
        }
        if self.input.level_up {
            self.build_level = (self.build_level + 1).min(MAX_BUILD_LEVEL);
        }
        if self.input.level_down {
            self.build_level = (self.build_level - 1).max(0);
        }

        self.weapon.cooldown = (self.weapon.cooldown - dt).max(0.0);

        self.sim.update_trade_ui(player.position);

        if self.input.interact {
            if self.sim.trade.visible {
                if self.sim.try_trade_buy(&mut self.inventory) {
                    self.particles.emit_hit_dust(player.position + Vec3::Y);
                }
            } else {
                self.ecosystem.toggle_herd_near(player.position, 5.0);
            }
        }

        if self.input.tame {
            if let Some((cid, _)) = self.ecosystem.try_tame_near(player.position, 5.0) {
                self.sim.events.push(crate::game::GameEvent::Tamed {
                    tick: self.sim.tick,
                    creature_id: cid,
                    actor: PLAYER_ENTITY,
                });
            }
        }

        if self.sim.trade.visible {
            if let Some(sel) = self.input.hotbar_select {
                if sel < 4 {
                    self.sim.trade.selection = sel;
                }
            }
        }

        if self.input.ignite && self.input.cursor_grabbed {
            if let Some(hit) = aim_block_pos(&cam) {
                let (_, pos) = hit;
                self.sim.ignite_at(pos.x, pos.z);
                self.particles.emit_hit_dust(pos + Vec3::new(0.0, 0.5, 0.0));
            }
        }

        if self.input.save_game {
            self.sim.save_now(player.position, player.yaw, &self.inventory, &self.blocks);
        }

        if self.input.release_herd {

            self.ecosystem.release_all_herd();

        }

        if self.input.craft_fence {

            self.inventory.craft_fence_from_wool();

        }

        if self.input.shear {

            if let Some((pos, wool)) = self.ecosystem.try_shear_near(player.position) {

                self.inventory.add_wool(wool);

                self.particles.emit_hit_dust(pos);

            }

        }



        let world = self.world.as_mut().unwrap();

        if self.input.place_block && self.input.build_mode && self.input.cursor_grabbed {
            if let Some((key, mut block, _)) =
                aim_build(&cam, &self.blocks, self.build_level, self.build_yaw)
            {
                if let Some(kind) = hotbar_to_block(self.inventory.hotbar) {
                    block.kind = kind;
                    if !self.blocks.has(key)
                        && self.inventory.use_hotbar_item(self.inventory.hotbar)
                    {
                        self.blocks.place(key, block);
                        sync_block_drawables(world, &self.blocks);
                    }
                }
            }
        }

        if self.input.remove_block && self.input.build_mode {
            if let Some(key) = aim_remove_key(&cam, &self.blocks) {
                if let Some(block) = self.blocks.remove(key) {
                    self.inventory.refund(block.kind.hotbar_slot());
                    sync_block_drawables(world, &self.blocks);
                }
            }
        }



        let is_night = self.day_night.lighting().is_night;

        self.ecosystem.update(

            dt,

            player.position,

            player.is_sprinting,

            self.gunshot_notify,

            is_night,

            &self.blocks,

            &self.collision,

        );

        self.gunshot_notify = None;

        sync_ecosystem_drawables(world, &self.ecosystem, player.position);

        self.sim.update(dt, player.position, &mut self.blocks, &mut self.ecosystem);
        self.sim.sync_drawables(world);
        sync_block_drawables(world, &self.blocks);
        self.sim
            .maybe_autosave(player.position, player.yaw, &self.inventory, &self.blocks);

        if self.input.shoot && self.weapon.cooldown <= 0.0 && self.input.cursor_grabbed {
            let wpn = self.weapon.active;
            self.weapon.cooldown = wpn.cooldown();
            self.hud.muzzle_flash = 1.0;
            self.viewmodel.on_shoot();

            if wpn.ignites() {
                let aim = player.position + cam.forward() * 2.5;
                self.sim.ignite_at(aim.x, aim.z);
                self.particles.emit_hit_dust(aim);
            } else if wpn.is_melee() {
                if let Some((_, hit, kind)) = self.ecosystem.melee_damage_at(
                    player.position,
                    cam.forward(),
                    wpn.melee_range(),
                    wpn.damage(),
                ) {
                    self.particles.emit_hit_dust(hit);
                    if let CreatureKind::Bird = kind {
                        self.inventory.add_bird_meat(1);
                    }
                } else if wpn == WeaponKind::Hammer {
                    if let Some(key) = aim_remove_key(&cam, &self.blocks) {
                        if let Some(block) = self.blocks.remove(key) {
                            self.inventory.refund(block.kind.hotbar_slot());
                            sync_block_drawables(world, &self.blocks);
                        }
                    }
                }
            } else if wpn.uses_projectile() {
                self.gunshot_notify = Some(muzzle_world);
                self.particles
                    .emit_muzzle_smoke(self.viewmodel.muzzle_vm_space(), Vec3::NEG_Z);
                if let Some(audio) = &self.audio {
                    audio.play_gunshot();
                }
                let speed = wpn.projectile_speed().unwrap();
                self.projectiles.spawn(
                    muzzle_world,
                    cam.forward(),
                    ProjectileParams {
                        speed,
                        damage: wpn.damage(),
                        radius: wpn.projectile_radius(),
                    },
                );
            }
        }



        if self.world.as_ref().unwrap().all_targets_destroyed() {

            self.victory = true;

            log::info!("VITÓRIA!");

        }



        let lighting = self.day_night.lighting();

        self.hud.show_crosshair = self.input.cursor_grabbed;

        self.hud.build_mode = self.input.build_mode;

        self.hud.day_hour = lighting.hour;

        self.hud.is_night = lighting.is_night;

        self.hud.hotbar_index = self.inventory.hotbar as u8;

        self.hud.fence_posts = self.inventory.fence_posts;

        self.hud.dirt_blocks = self.inventory.dirt_blocks;

        self.hud.stone_blocks = self.inventory.stone_blocks;
        self.hud.wall_blocks = self.inventory.wall_blocks;
        self.hud.wood_walls = self.inventory.wood_walls;

        self.hud.wool = self.inventory.wool;

        self.hud.mutton = self.inventory.mutton;

        self.hud.sheep_alive = self.ecosystem.sheep_alive() as u32;

        self.hud.sheep_herded = self.ecosystem.herded_count as u32;
        self.hud.radar_blips = self
            .ecosystem
            .build_radar(player.position, player.yaw, RADAR_RANGE);
        self.hud.nearest_interact_m = self.ecosystem.nearest_interact_dist(player.position);
        self.hud.hud_time = self.scene_time;
        self.hud.trade_visible = self.sim.trade.visible;
        self.hud.trade_selection = self.sim.trade.selection;
        self.hud.chunks_loaded = self.sim.chunks.loaded_count() as u32;
        self.hud.net_label = self.sim.net.role_label().to_string();

        self.hud.crosshair_spread = if player.is_sprinting {

            1.0

        } else if player.is_moving {

            0.45

        } else {

            0.1

        };



        if let Some(ew) = self.engine_window.as_mut() {

            let night_factor = (1.0 - lighting.sun_dir[1].clamp(0.0, 1.0)).powf(0.65);

            ew.renderer.set_day_night(DayNightGpu {
                sun_dir: lighting.sun_dir,
                horizon: lighting.horizon,
                zenith: lighting.zenith,
                fog_color: lighting.fog_color,
                night_factor,
                fog_intensity: 0.4,
            });

        }



        self.input.reset_frame();

    }



    fn render(&mut self) {

        let ew = self.engine_window.as_mut().unwrap();

        let player = self.player.as_ref().unwrap();

        let world = self.world.as_ref().unwrap();

        let assets = self.assets.as_ref().unwrap();

        let lighting = self.day_night.lighting();



        let size = ew.window.inner_size();

        let camera = player.to_camera(size.width as f32 / size.height.max(1) as f32);

        let vm = self.viewmodel.transform();



        ew.renderer.set_scene_time(self.scene_time);

        ew.renderer.begin_frame(lighting.clear);

        let _ = ew.renderer.begin_shadow_pass(&camera);

        let cam_pos = camera.position;

        let cam_fwd = camera.forward();

        for drawable in &world.drawables {

            if matches!(drawable.material, DrawMaterial::Water) {

                continue;

            }

            if !should_shadow(drawable.position, cam_pos, &drawable.model_id) {

                continue;

            }

            if let Some(gpu) = resolve_mesh(drawable, assets) {

                let _ = ew.renderer.draw_shadow(gpu, drawable.model_matrix());

            }

        }

        let _ = ew.renderer.end_shadow_pass();

        let _ = ew.renderer.begin_scene_pass(lighting.clear);

        let _ = ew.renderer.draw_sky(&camera);



        for drawable in &world.drawables {

            if matches!(drawable.material, DrawMaterial::Water) {

                continue;

            }

            if !should_draw(drawable.position, cam_pos, cam_fwd, &drawable.model_id) {

                continue;

            }

            if let Some(gpu) = resolve_mesh(drawable, assets) {

                let _ = ew.renderer.draw(

                    gpu,

                    drawable.model_matrix(),

                    &camera,

                    drawable.material,

                );

            }

        }

        for remote in &self.sim.net.remotes {
            let col = [0.2, 0.85, 1.0, 0.85];
            let gy = remote.y - 1.7;
            ew.renderer.draw_line_strip(
                &camera,
                &[
                    [remote.x - 0.3, gy, remote.z],
                    [remote.x + 0.3, gy, remote.z],
                ],
                col,
            );
            ew.renderer.draw_line_strip(
                &camera,
                &[[remote.x, gy, remote.z], [remote.x, remote.y, remote.z]],
                col,
            );
        }

        for &(fx, fz, intensity) in &self.sim.fire_visuals {
            if intensity < 0.1 {
                continue;
            }
            let gy = crate::assets::sample_desert_height(fx, fz);
            let tip = gy + 0.6 + intensity * 2.5;
            let fire_col = [1.0, 0.35 + intensity * 0.4, 0.05, 0.7 * intensity];
            ew.renderer
                .draw_line_strip(&camera, &[[fx, gy, fz], [fx, tip, fz]], fire_col);
        }

        for pos in self
            .ecosystem
            .beacon_positions(player.position, BEACON_RANGE, 4)
        {
            let ground = pos.y + 0.3;
            let top = ground + 5.0;
            let beam = [[pos.x, ground, pos.z], [pos.x, top, pos.z]];
            ew.renderer
                .draw_line_strip(&camera, &beam, [0.25, 1.0, 0.4, 0.55]);
        }



        let water_planes: Vec<_> = world

            .drawables

            .iter()

            .filter(|d| matches!(d.material, DrawMaterial::Water))

            .collect();

        for water in &water_planes {

            if !should_draw(water.position, cam_pos, cam_fwd, "oasis_water") {

                continue;

            }

            let plane_y = water.position.y;

            let refl_cam = ew.renderer.begin_planar_reflection(&camera, plane_y);

            for drawable in &world.drawables {

                if matches!(drawable.material, DrawMaterial::Water) {

                    continue;

                }

                if drawable.model_id == "terrain" {

                    continue;

                }

                if !should_reflect(drawable.position, cam_pos) {

                    continue;

                }

                if let Some(gpu) = resolve_mesh(drawable, assets) {

                    let _ = ew.renderer.draw(

                        gpu,

                        drawable.model_matrix(),

                        &refl_cam,

                        drawable.material,

                    );

                }

            }

            ew.renderer.end_planar_reflection();

            if let Some(gpu) = assets.mesh(&water.model_id) {

                ew.renderer.draw_water(

                    &camera,

                    gpu,

                    water.model_matrix(),

                    water.position.y,

                );

            }

        }



        if self.input.build_mode {
            if let Some((key, block, _)) =
                aim_build(&camera, &self.blocks, self.build_level, self.build_yaw)
            {
                let kind = hotbar_to_block(self.inventory.hotbar).unwrap_or(block.kind);
                let preview = PlacedBlock {
                    kind,
                    yaw: self.build_yaw,
                };
                let (pos, _, scale) = BlockGrid::world_transform(key, preview);
                let ghost_h = scale.y;
                let ghost = [[pos.x, pos.y, pos.z], [pos.x, pos.y + ghost_h, pos.z]];
                ew.renderer
                    .draw_line_strip(&camera, &ghost, [0.3, 1.0, 0.4, 0.7]);
            }
        }



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

                kind: p.kind as f32,

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

                kind: 0.0,

            });

        }

        ew.renderer.draw_particles(&camera, &vm_parts, vm);

        ew.renderer.draw_world_particles(&camera, &world_parts);

        ew.renderer.sand_draw(&camera);



        let _ = ew.renderer.end_scene_pass();

        ew.renderer.draw_hud(&self.hud);

        let _ = ew.renderer.end_frame();



        let hour = lighting.hour as u32;

        let min = ((lighting.hour - hour as f32) * 60.0) as u32;

        let mode = if self.input.build_mode {
            format!("BUILD nv{}", self.build_level)
        } else {
            format!("{}", self.weapon.active.label())
        };

        let near_npc = if self.hud.nearest_interact_m < BEACON_RANGE {
            format!(" | NPC {:.0}m", self.hud.nearest_interact_m)
        } else if self.hud.nearest_interact_m < RADAR_RANGE {
            format!(" | criatura {:.0}m", self.hud.nearest_interact_m)
        } else {
            String::new()
        };

        let territory = self
            .sim
            .territories
            .zone_at(player.position.x, player.position.z)
            .map(|z| format!(" [{}]", z.name))
            .unwrap_or_default();

        let title = format!(
            "{:02}:{:02} {} | O:{} R:{} | La:{} Carne:{} Pass:{}{}{} | {} ch:{} | E:comprar T:domar | {}",
            hour,
            min,
            if lighting.is_night { "NOITE" } else { "DIA" },
            self.ecosystem.alive_count(),
            self.ecosystem.herded_count,
            self.inventory.wool,
            self.inventory.mutton,
            self.inventory.bird_meat,
            near_npc,
            territory,
            self.hud.net_label,
            self.hud.chunks_loaded,
            mode
        );

        ew.window.set_title(&title);

    }

}



fn hotbar_to_block(slot: HotbarSlot) -> Option<BlockKind> {
    match slot {
        HotbarSlot::Fence => Some(BlockKind::Fence),
        HotbarSlot::Dirt => Some(BlockKind::Dirt),
        HotbarSlot::Stone => Some(BlockKind::Stone),
        HotbarSlot::Wall => Some(BlockKind::Wall),
        HotbarSlot::WoodWall => Some(BlockKind::WoodWall),
        _ => None,
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

                let now = Instant::now();

                let dt = self.last_frame.elapsed().as_secs_f32().min(0.033);

                self.last_frame = now;

                self.next_frame = now + Duration::from_secs_f64(1.0 / 60.0);

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



    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {

        if let Some(ew) = &self.engine_window {

            let now = Instant::now();

            if now >= self.next_frame {

                ew.window.request_redraw();

            } else {

                event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));

            }

        }

    }

}



fn resolve_mesh<'a>(

    drawable: &crate::game::world::Drawable,

    assets: &'a GpuAssetCache,

) -> Option<&'a crate::graphics::GpuMesh> {

    if drawable.model_id == "terrain" {

        Some(&assets.terrain)

    } else if drawable.model_id == "oasis_water" {

        assets.mesh("oasis_water")

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



