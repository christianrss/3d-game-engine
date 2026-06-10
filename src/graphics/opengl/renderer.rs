//! Renderer OpenGL — PBR, shadow maps, bloom, HUD.

use crate::graphics::backend::GfxBackend;
use crate::graphics::opengl::context::GlContext;
use crate::graphics::opengl::sand_gpu::GpuSandField;
use crate::graphics::renderer::{DayNightGpu, HudState};
use crate::graphics::shaders::{
    FRAGMENT_GLSL_GL33, LIGHT_DIRECTION, PARTICLE_FRAGMENT_GLSL_GL33, PARTICLE_VERTEX_GLSL_GL33,
    POST_FRAGMENT_GLSL_GL33, POST_VERTEX_GLSL_GL33, SHADOW_FRAGMENT_GLSL_GL33,
    SHADOW_VERTEX_GLSL_GL33, SKY_FRAGMENT_GLSL_GL33, SKY_VERTEX_GLSL_GL33, VERTEX_GLSL_GL33,
    WATER_FRAGMENT_GLSL_GL33, WATER_VERTEX_GLSL_GL33,
};
use crate::graphics::ParticleDraw;
use crate::graphics::{Camera, Color, DrawMaterial, GpuMesh, GpuTexture, Mesh, TextureData};
use crate::math::{Mat4, Vec3};
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;

const SHADOW_SIZE: i32 = 2048;
const REFLECT_SIZE: i32 = 512;

#[derive(Debug)]
pub enum OpenGLError {
    Shader(String),
    Gl(String),
    Context(String),
}

impl std::fmt::Display for OpenGLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenGLError::Shader(e) => write!(f, "Shader: {e}"),
            OpenGLError::Gl(e) => write!(f, "OpenGL: {e}"),
            OpenGLError::Context(e) => write!(f, "Context: {e}"),
        }
    }
}
impl std::error::Error for OpenGLError {}

struct GlMesh {
    vao: u32,
    index_count: u32,
}

struct Framebuffers {
    scene_fbo: u32,
    scene_color: u32,
    scene_depth: u32,
    shadow_fbo: u32,
    shadow_depth: u32,
    reflect_fbo: u32,
    reflect_color: u32,
    reflect_depth: u32,
}

pub struct OpenGLRenderer {
    gl_ctx: GlContext,
    shader: u32,
    shadow_shader: u32,
    sky_shader: u32,
    post_shader: u32,
    particle_shader: u32,
    particle_vao: u32,
    particle_vbo: u32,
    hud_shader: u32,
    hud_vao: u32,
    hud_panel_vao: u32,
    hud_panel_vbo: u32,
    line_shader: u32,
    line_vao: u32,
    line_vbo: u32,
    water_shader: u32,
    post_vao: u32,
    sky_vao: u32,
    sky_index_count: u32,
    meshes: HashMap<u64, GlMesh>,
    textures: HashMap<u64, u32>,
    fb: Option<Framebuffers>,
    terrain_albedo: u64,
    terrain_normal: u64,
    terrain_rough: u64,
    terrain_ao: u64,
    rock_albedo: u64,
    rock_normal: u64,
    rock_rough: u64,
    scene_time: f32,
    day_night: DayNightGpu,
    reflect_view_proj: Mat4,
    sand: Option<GpuSandField>,
    light_space: Mat4,
    next_id: u64,
    next_tex_id: u64,
    width: u32,
    height: u32,
    in_scene: bool,
}

impl OpenGLRenderer {
    pub fn from_context(gl_ctx: GlContext, width: u32, height: u32) -> Result<Self, OpenGLError> {
        gl_ctx.load_gl();

        let shader = compile_shader_program(VERTEX_GLSL_GL33, FRAGMENT_GLSL_GL33)?;
        let shadow_shader =
            compile_shader_program(SHADOW_VERTEX_GLSL_GL33, SHADOW_FRAGMENT_GLSL_GL33)?;
        let sky_shader = compile_shader_program(SKY_VERTEX_GLSL_GL33, SKY_FRAGMENT_GLSL_GL33)?;
        let post_shader = compile_shader_program(POST_VERTEX_GLSL_GL33, POST_FRAGMENT_GLSL_GL33)?;
        let particle_shader =
            compile_shader_program(PARTICLE_VERTEX_GLSL_GL33, PARTICLE_FRAGMENT_GLSL_GL33)?;
        let (particle_vao, particle_vbo) = create_particle_vao();
        let hud_shader = compile_hud_shader()?;
        let hud_vao = create_hud_vao();
        let (hud_panel_vao, hud_panel_vbo) = create_hud_panel_vao();
        let line_shader = compile_line_shader()?;
        let (line_vao, line_vbo) = create_line_vao();
        let water_shader =
            compile_shader_program(WATER_VERTEX_GLSL_GL33, WATER_FRAGMENT_GLSL_GL33)?;
        let post_vao = create_post_vao();
        let (sky_vao, sky_index_count) = create_sky_vao();

        let mut r = Self {
            gl_ctx,
            shader,
            shadow_shader,
            sky_shader,
            post_shader,
            particle_shader,
            particle_vao,
            particle_vbo,
            hud_shader,
            hud_vao,
            hud_panel_vao,
            hud_panel_vbo,
            line_shader,
            line_vao,
            line_vbo,
            water_shader,
            post_vao,
            sky_vao,
            sky_index_count,
            meshes: HashMap::new(),
            textures: HashMap::new(),
            fb: None,
            terrain_albedo: 0,
            terrain_normal: 0,
            terrain_rough: 0,
            terrain_ao: 0,
            rock_albedo: 0,
            rock_normal: 0,
            rock_rough: 0,
            scene_time: 0.0,
            day_night: DayNightGpu::default(),
            reflect_view_proj: Mat4::IDENTITY,
            sand: GpuSandField::new().ok(),
            light_space: Mat4::IDENTITY,
            next_id: 1,
            next_tex_id: 1,
            width: width.max(1),
            height: height.max(1),
            in_scene: false,
        };

        r.fb = Some(r.create_framebuffers()?);

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::Enable(gl::PROGRAM_POINT_SIZE);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        Ok(r)
    }

    pub fn set_terrain_textures(
        &mut self,
        albedo: &GpuTexture,
        normal: &GpuTexture,
        rough: &GpuTexture,
        ao: &GpuTexture,
    ) {
        self.terrain_albedo = albedo.gpu_id;
        self.terrain_normal = normal.gpu_id;
        self.terrain_rough = rough.gpu_id;
        self.terrain_ao = ao.gpu_id;
    }

    pub fn set_rock_textures(
        &mut self,
        albedo: &GpuTexture,
        normal: &GpuTexture,
        rough: &GpuTexture,
    ) {
        self.rock_albedo = albedo.gpu_id;
        self.rock_normal = normal.gpu_id;
        self.rock_rough = rough.gpu_id;
    }

    pub fn set_scene_time(&mut self, t: f32) {
        self.scene_time = t;
    }

    pub fn set_day_night(&mut self, lighting: DayNightGpu) {
        self.day_night = lighting;
    }

    fn create_framebuffers(&self) -> Result<Framebuffers, OpenGLError> {
        let mut scene_fbo = 0u32;
        let mut scene_color = 0u32;
        let mut scene_depth = 0u32;
        let mut shadow_fbo = 0u32;
        let mut shadow_depth = 0u32;
        let mut reflect_fbo = 0u32;
        let mut reflect_color = 0u32;
        let mut reflect_depth = 0u32;

        unsafe {
            gl::GenFramebuffers(1, &mut scene_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, scene_fbo);
            gl::GenTextures(1, &mut scene_color);
            gl::BindTexture(gl::TEXTURE_2D, scene_color);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA16F as i32,
                self.width as i32,
                self.height as i32,
                0,
                gl::RGBA,
                gl::FLOAT,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                scene_color,
                0,
            );

            gl::GenTextures(1, &mut scene_depth);
            gl::BindTexture(gl::TEXTURE_2D, scene_depth);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT24 as i32,
                self.width as i32,
                self.height as i32,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                ptr::null(),
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                scene_depth,
                0,
            );

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                return Err(OpenGLError::Gl("Scene FBO incompleto".into()));
            }

            gl::GenFramebuffers(1, &mut shadow_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_fbo);
            gl::GenTextures(1, &mut shadow_depth);
            gl::BindTexture(gl::TEXTURE_2D, shadow_depth);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT24 as i32,
                SHADOW_SIZE,
                SHADOW_SIZE,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32);
            let border = [1.0f32, 1.0, 1.0, 1.0];
            gl::TexParameterfv(gl::TEXTURE_2D, gl::TEXTURE_BORDER_COLOR, border.as_ptr());
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                shadow_depth,
                0,
            );
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                return Err(OpenGLError::Gl("Shadow FBO incompleto".into()));
            }

            gl::GenFramebuffers(1, &mut reflect_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, reflect_fbo);
            gl::GenTextures(1, &mut reflect_color);
            gl::BindTexture(gl::TEXTURE_2D, reflect_color);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB16F as i32,
                REFLECT_SIZE,
                REFLECT_SIZE,
                0,
                gl::RGB,
                gl::FLOAT,
                ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                reflect_color,
                0,
            );
            gl::GenTextures(1, &mut reflect_depth);
            gl::BindTexture(gl::TEXTURE_2D, reflect_depth);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT24 as i32,
                REFLECT_SIZE,
                REFLECT_SIZE,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                ptr::null(),
            );
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                reflect_depth,
                0,
            );
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                return Err(OpenGLError::Gl("Reflection FBO incompleto".into()));
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Ok(Framebuffers {
            scene_fbo,
            scene_color,
            scene_depth,
            shadow_fbo,
            shadow_depth,
            reflect_fbo,
            reflect_color,
            reflect_depth,
        })
    }

    pub fn begin_planar_reflection(&mut self, camera: &Camera, plane_y: f32) -> Camera {
        let mut refl = camera.clone();
        refl.position.y = 2.0 * plane_y - camera.position.y;
        refl.pitch = -camera.pitch;

        self.reflect_view_proj = refl.view_projection();

        if let Some(fb) = &self.fb {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, fb.reflect_fbo);
                gl::Viewport(0, 0, REFLECT_SIZE, REFLECT_SIZE);
                gl::ClearColor(0.45, 0.72, 0.95, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
        }
        refl
    }

    pub fn end_planar_reflection(&mut self) {
        if let Some(fb) = &self.fb {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, fb.scene_fbo);
                gl::Viewport(0, 0, self.width as i32, self.height as i32);
            }
        }
    }

    pub fn sand_emit(&mut self, pos: Vec3, vel: Vec3, count: usize) {
        if let Some(sand) = &mut self.sand {
            sand.emit(pos, vel, count);
        }
    }

    pub fn sand_update(&mut self, dt: f32, wind: Vec3) {
        if let Some(sand) = &mut self.sand {
            sand.update(dt, wind);
        }
    }

    pub fn sand_draw(&self, camera: &Camera) {
        if let Some(sand) = &self.sand {
            sand.draw(camera);
        }
    }

    fn compute_light_space(&self, camera: &Camera) -> Mat4 {
        let focus = camera.position + camera.forward() * 35.0;
        let light_dir = Vec3::from_array(self.day_night.sun_dir).normalize();
        let light_pos = focus - light_dir * 90.0;
        let view = Mat4::look_at_rh(light_pos, focus, Vec3::Y);
        let proj = Mat4::orthographic_rh_gl(-65.0, 65.0, -65.0, 65.0, 1.0, 200.0);
        proj * view
    }

    pub fn draw_hud(&self, hud: &HudState) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::UseProgram(self.hud_shader);
            gl::BindVertexArray(self.hud_vao);

            if hud.show_crosshair {
                let spread = hud.crosshair_spread * 0.018;
                if hud.build_mode {
                    set_hud_color(self.hud_shader, [0.35, 1.0, 0.45, 0.95]);
                } else {
                    set_hud_color(self.hud_shader, [1.0, 1.0, 1.0, 0.92]);
                }
                gl::LineWidth(2.0);
                // Cruz principal (8 vértices = 4 linhas)
                gl::DrawArrays(gl::LINES, 0, 8);
                // Anel de mira
                set_hud_color(self.hud_shader, [0.95, 0.95, 0.95, 0.55]);
                gl::LineWidth(1.5);
                gl::DrawArrays(gl::LINE_LOOP, 8, 32);
                // Poste inferior (mira de ferro)
                set_hud_color(self.hud_shader, [1.0, 0.85, 0.35, 0.9]);
                gl::LineWidth(2.5);
                gl::DrawArrays(gl::LINES, 40, 2);
                // Ponto central
                set_hud_color(self.hud_shader, [1.0, 0.9, 0.4, 0.95]);
                gl::PointSize(3.0 + spread * 40.0);
                gl::DrawArrays(gl::POINTS, 42, 1);
            }
            if hud.muzzle_flash > 0.01 {
                set_hud_color(self.hud_shader, [1.0, 0.9, 0.4, hud.muzzle_flash * 0.6]);
                gl::LineWidth(3.0);
                gl::DrawArrays(gl::LINES, 43, 4);
            }
            if hud.hit_flash > 0.01 {
                set_hud_color(self.hud_shader, [0.2, 1.0, 0.3, hud.hit_flash * 0.8]);
                gl::LineWidth(4.0);
                gl::DrawArrays(gl::LINES, 47, 8);
            }

            draw_hud_panels(self, hud);

            gl::BindVertexArray(0);
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    pub fn draw_line_strip(&self, camera: &Camera, points: &[[f32; 3]], color: [f32; 4]) {
        if points.len() < 2 {
            return;
        }
        let vp = camera.view_projection();
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::DepthMask(gl::FALSE);
            gl::UseProgram(self.line_shader);
            set_mat4(self.line_shader, "uMVP", vp);
            set_hud_color(self.line_shader, color);
            gl::BindVertexArray(self.line_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.line_vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (points.len() * 12) as isize,
                points.as_ptr() as *const _,
            );
            gl::LineWidth(2.5);
            gl::DrawArrays(gl::LINE_STRIP, 0, points.len() as i32);
            gl::BindVertexArray(0);
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
        }
    }

    pub fn draw_viewmodel(&self, camera: &Camera, gun_mesh: &GpuMesh, local: Mat4) {
        let gl_mesh = match self.meshes.get(&gun_mesh.gpu_id) {
            Some(m) => m,
            None => return,
        };
        let mut view = camera.view_matrix();
        view.w_axis.x = 0.0;
        view.w_axis.y = 0.0;
        view.w_axis.z = 0.0;
        view.w_axis.w = 1.0;
        let mvp = camera.projection_matrix() * view * local;
        self.draw_mesh_internal(
            gl_mesh,
            mvp,
            local,
            camera,
            DrawMaterial::metal(),
            0.0,
        );
    }

    pub fn draw_particles(&self, camera: &Camera, particles: &[ParticleDraw], vm_transform: Mat4) {
        if particles.is_empty() {
            return;
        }
        let mut view = camera.view_matrix();
        view.w_axis.x = 0.0;
        view.w_axis.y = 0.0;
        view.w_axis.z = 0.0;
        view.w_axis.w = 1.0;
        let vp = camera.projection_matrix() * view;

        let mut verts = Vec::with_capacity(particles.len() * 6);
        for p in particles {
            let lp = vm_transform.transform_point3(crate::math::Vec3::from_array(p.pos));
            verts.extend_from_slice(&[lp.x, lp.y, lp.z, p.size, p.alpha, p.kind]);
        }
        self.draw_particle_verts(&vp, &verts);
    }

    pub fn draw_world_particles(&self, camera: &Camera, particles: &[ParticleDraw]) {
        if particles.is_empty() {
            return;
        }
        let vp = camera.view_projection();
        let mut verts = Vec::with_capacity(particles.len() * 6);
        for p in particles {
            verts.extend_from_slice(&[
                p.pos[0], p.pos[1], p.pos[2], p.size, p.alpha, p.kind,
            ]);
        }
        self.draw_particle_verts(&vp, &verts);
    }

    fn draw_particle_verts(&self, mvp: &Mat4, verts: &[f32]) {
        if verts.is_empty() {
            return;
        }
        let count = verts.len() / 6;
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::DepthMask(gl::FALSE);
            gl::UseProgram(self.particle_shader);
            gl::BindVertexArray(self.particle_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.particle_vbo);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (verts.len() * 4) as isize,
                verts.as_ptr() as *const _,
            );
            set_mat4(self.particle_shader, "uMVP", *mvp);
            gl::DrawArrays(gl::POINTS, 0, count as i32);
            gl::BindVertexArray(0);
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
        }
    }

    pub fn draw_water(
        &self,
        camera: &Camera,
        gpu_mesh: &GpuMesh,
        model: Mat4,
        shore_height: f32,
    ) {
        let gl_mesh = match self.meshes.get(&gpu_mesh.gpu_id) {
            Some(m) => m,
            None => return,
        };
        let mvp = camera.view_projection() * model;
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::DepthMask(gl::FALSE);
            gl::UseProgram(self.water_shader);
            set_mat4(self.water_shader, "uMVP", mvp);
            set_mat4(self.water_shader, "uModel", model);
            set_float(self.water_shader, "uTime", self.scene_time);
            set_float(self.water_shader, "uShoreHeight", shore_height);
            set_float(self.water_shader, "uWaterPlane", shore_height);
            set_vec3(self.water_shader, "uCameraPos", camera.position.to_array());
            set_vec3(self.water_shader, "uLightDir", LIGHT_DIRECTION);
            set_mat4(self.water_shader, "uReflectVP", self.reflect_view_proj);
            if let Some(fb) = &self.fb {
                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, fb.reflect_color);
                set_int(self.water_shader, "uReflection", 1);
                set_int(self.water_shader, "uHasReflection", 1);
            } else {
                set_int(self.water_shader, "uHasReflection", 0);
            }
            gl::BindVertexArray(gl_mesh.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                gl_mesh.index_count as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
        }
    }

    fn draw_mesh_internal(
        &self,
        gl_mesh: &GlMesh,
        mvp: Mat4,
        model: Mat4,
        camera: &Camera,
        material: DrawMaterial,
        fog_density: f32,
    ) {
        unsafe {
            gl::UseProgram(self.shader);
            set_mat4(self.shader, "uMVP", mvp);
            set_mat4(self.shader, "uModel", model);
            set_mat4(self.shader, "uLightSpaceMatrix", self.light_space);
            set_vec3(self.shader, "uLightDir", self.day_night.sun_dir);
            set_vec3(self.shader, "uCameraPos", camera.position.to_array());
            set_vec3(self.shader, "uFogColor", self.day_night.fog_color);
            set_float(self.shader, "uFogDensity", fog_density);
            set_float(self.shader, "uTime", self.scene_time);

            bind_texture(self.fb.as_ref().map(|f| &f.shadow_depth), 4);
            set_int(self.shader, "uShadowMap", 4);

            match material {
                DrawMaterial::Standard { roughness, metallic } => {
                    set_int(self.shader, "uMatType", 0);
                    set_int(self.shader, "uUseAlbedo", 0);
                    set_int(self.shader, "uUseNormalMap", 0);
                    set_int(self.shader, "uUseRoughMap", 0);
                    set_int(self.shader, "uUseAOMap", 0);
                    set_float(self.shader, "uRoughness", roughness);
                    set_float(self.shader, "uMetallic", metallic);
                    set_float(self.shader, "uTiling", 1.0);
                }
                DrawMaterial::Terrain { tiling } => {
                    set_int(self.shader, "uMatType", 1);
                    set_int(self.shader, "uUseAlbedo", 1);
                    set_int(self.shader, "uUseNormalMap", 1);
                    set_int(self.shader, "uUseRoughMap", 1);
                    set_int(self.shader, "uUseAOMap", 1);
                    set_float(self.shader, "uRoughness", 0.9);
                    set_float(self.shader, "uMetallic", 0.0);
                    set_float(self.shader, "uTiling", tiling);
                    bind_texture(self.textures.get(&self.terrain_albedo), 0);
                    bind_texture(self.textures.get(&self.terrain_normal), 1);
                    bind_texture(self.textures.get(&self.terrain_rough), 2);
                    bind_texture(self.textures.get(&self.terrain_ao), 3);
                    set_int(self.shader, "uAlbedo", 0);
                    set_int(self.shader, "uNormalMap", 1);
                    set_int(self.shader, "uRoughMap", 2);
                    set_int(self.shader, "uAOMap", 3);
                }
                DrawMaterial::Rock { tiling } => {
                    set_int(self.shader, "uMatType", 2);
                    set_int(self.shader, "uUseAlbedo", 1);
                    set_int(self.shader, "uUseNormalMap", 1);
                    set_int(self.shader, "uUseRoughMap", 1);
                    set_int(self.shader, "uUseAOMap", 0);
                    set_float(self.shader, "uRoughness", 0.85);
                    set_float(self.shader, "uMetallic", 0.0);
                    set_float(self.shader, "uTiling", tiling);
                    bind_texture(self.textures.get(&self.rock_albedo), 0);
                    bind_texture(self.textures.get(&self.rock_normal), 1);
                    bind_texture(self.textures.get(&self.rock_rough), 2);
                    set_int(self.shader, "uAlbedo", 0);
                    set_int(self.shader, "uNormalMap", 1);
                    set_int(self.shader, "uRoughMap", 2);
                }
                DrawMaterial::Water => {}
            }

            gl::BindVertexArray(gl_mesh.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                gl_mesh.index_count as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
    }

    fn draw_shadow_mesh(&self, gl_mesh: &GlMesh, model: Mat4) {
        unsafe {
            gl::UseProgram(self.shadow_shader);
            set_mat4(self.shadow_shader, "uLightSpaceMatrix", self.light_space);
            set_mat4(self.shadow_shader, "uModel", model);
            gl::BindVertexArray(gl_mesh.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                gl_mesh.index_count as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
    }
}

impl GfxBackend for OpenGLRenderer {
    type Error = OpenGLError;

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.gl_ctx.resize(self.width, self.height);
        if let Some(fb) = self.fb.take() {
            unsafe {
                gl::DeleteFramebuffers(1, &fb.scene_fbo);
                gl::DeleteTextures(1, &fb.scene_color);
                gl::DeleteTextures(1, &fb.scene_depth);
                gl::DeleteFramebuffers(1, &fb.shadow_fbo);
                gl::DeleteTextures(1, &fb.shadow_depth);
                gl::DeleteFramebuffers(1, &fb.reflect_fbo);
                gl::DeleteTextures(1, &fb.reflect_color);
                gl::DeleteTextures(1, &fb.reflect_depth);
            }
        }
        self.fb = self.create_framebuffers().ok();
    }

    fn upload_mesh(&mut self, mesh: &Mesh) -> Result<GpuMesh, OpenGLError> {
        let id = self.next_id;
        self.next_id += 1;
        let mut vao = 0u32;
        let mut vbo = 0u32;
        let mut ebo = 0u32;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mesh.vertices.len() * 44) as isize,
                mesh.vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.indices.len() * 4) as isize,
                mesh.indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            let stride = 44i32;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, 12 as *const _);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, 24 as *const _);
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, stride, 32 as *const _);
            gl::BindVertexArray(0);
        }
        self.meshes.insert(
            id,
            GlMesh {
                vao,
                index_count: mesh.indices.len() as u32,
            },
        );
        Ok(GpuMesh {
            vertex_count: mesh.vertices.len() as u32,
            index_count: mesh.indices.len() as u32,
            gpu_id: id,
        })
    }

    fn upload_texture(&mut self, data: &TextureData) -> Result<GpuTexture, OpenGLError> {
        let id = self.next_tex_id;
        self.next_tex_id += 1;
        let mut tex = 0u32;
        unsafe {
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                data.width as i32,
                data.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.pixels.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
        self.textures.insert(id, tex);
        Ok(GpuTexture {
            gpu_id: id,
            width: data.width,
            height: data.height,
        })
    }

    fn begin_frame(&mut self, _clear: Color) {
        self.in_scene = false;
    }

    fn begin_shadow_pass(&mut self, camera: &Camera) -> Result<(), OpenGLError> {
        let fb = self.fb.as_ref().ok_or_else(|| OpenGLError::Gl("FBO".into()))?;
        self.light_space = self.compute_light_space(camera);
        unsafe {
            gl::Viewport(0, 0, SHADOW_SIZE, SHADOW_SIZE);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fb.shadow_fbo);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
            gl::CullFace(gl::FRONT);
        }
        Ok(())
    }

    fn draw_shadow(&mut self, gpu_mesh: &GpuMesh, model: Mat4) -> Result<(), OpenGLError> {
        let gl_mesh = self
            .meshes
            .get(&gpu_mesh.gpu_id)
            .ok_or_else(|| OpenGLError::Gl("Mesh shadow".into()))?;
        self.draw_shadow_mesh(gl_mesh, model);
        Ok(())
    }

    fn end_shadow_pass(&mut self) -> Result<(), OpenGLError> {
        unsafe {
            gl::CullFace(gl::BACK);
        }
        Ok(())
    }

    fn begin_scene_pass(&mut self, clear: Color) -> Result<(), OpenGLError> {
        let fb = self.fb.as_ref().ok_or_else(|| OpenGLError::Gl("FBO".into()))?;
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fb.scene_fbo);
            gl::ClearColor(clear.r, clear.g, clear.b, clear.a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.in_scene = true;
        Ok(())
    }

    fn draw_sky(&mut self, camera: &Camera) -> Result<(), OpenGLError> {
        unsafe {
            gl::DepthFunc(gl::LEQUAL);
            gl::UseProgram(self.sky_shader);
            let mut view = camera.view_matrix();
            view.w_axis.x = 0.0;
            view.w_axis.y = 0.0;
            view.w_axis.z = 0.0;
            view.w_axis.w = 1.0;
            set_mat4(self.sky_shader, "uViewProj", camera.projection_matrix() * view);
            set_vec3(self.sky_shader, "uHorizon", self.day_night.horizon);
            set_vec3(self.sky_shader, "uZenith", self.day_night.zenith);
            set_vec3(self.sky_shader, "uSunDir", self.day_night.sun_dir);
            set_float(self.sky_shader, "uNightFactor", self.day_night.night_factor);
            gl::BindVertexArray(self.sky_vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.sky_index_count as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
            gl::DepthFunc(gl::LESS);
        }
        Ok(())
    }

    fn draw(
        &mut self,
        gpu_mesh: &GpuMesh,
        model: Mat4,
        camera: &Camera,
        material: DrawMaterial,
    ) -> Result<(), OpenGLError> {
        let gl_mesh = self
            .meshes
            .get(&gpu_mesh.gpu_id)
            .ok_or_else(|| OpenGLError::Gl("Mesh GPU não encontrada".into()))?;
        let mvp = camera.view_projection() * model;
        let fog = match material {
            DrawMaterial::Terrain { .. } => 0.000025,
            DrawMaterial::Rock { .. } => 0.00004,
            DrawMaterial::Standard { .. } => 0.00006,
            DrawMaterial::Water => 0.00003,
        };
        self.draw_mesh_internal(gl_mesh, mvp, model, camera, material, fog);
        Ok(())
    }

    fn end_scene_pass(&mut self) -> Result<(), OpenGLError> {
        let fb = self.fb.as_ref().ok_or_else(|| OpenGLError::Gl("FBO".into()))?;
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            gl::Disable(gl::DEPTH_TEST);
            gl::UseProgram(self.post_shader);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, fb.scene_color);
            set_int(self.post_shader, "uScene", 0);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, fb.scene_depth);
            set_int(self.post_shader, "uDepth", 1);
            set_vec2(
                self.post_shader,
                "uTexelSize",
                [1.0 / self.width as f32, 1.0 / self.height as f32],
            );
            set_float(self.post_shader, "uNear", 0.1);
            set_float(self.post_shader, "uFar", 500.0);
            gl::BindVertexArray(self.post_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
            gl::Enable(gl::DEPTH_TEST);
        }
        self.in_scene = false;
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), OpenGLError> {
        self.gl_ctx.swap_buffers().map_err(OpenGLError::Context)
    }
}

fn create_particle_vao() -> (u32, u32) {
    let mut vao = 0u32;
    let mut vbo = 0u32;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 4096 * 24, ptr::null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 24, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 1, gl::FLOAT, gl::FALSE, 24, 12 as *const _);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 1, gl::FLOAT, gl::FALSE, 24, 16 as *const _);
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(3, 1, gl::FLOAT, gl::FALSE, 24, 20 as *const _);
        gl::BindVertexArray(0);
    }
    (vao, vbo)
}

fn create_post_vao() -> u32 {
    let verts: [f32; 6] = [-1.0, -1.0, 3.0, -1.0, -1.0, 3.0];
    let mut vao = 0u32;
    let mut vbo = 0u32;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 24, verts.as_ptr() as *const _, gl::STATIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindVertexArray(0);
    }
    vao
}

fn create_sky_vao() -> (u32, u32) {
    let sky = crate::graphics::primitives::sky_dome(1.0, 32, 16);
    let mut vao = 0u32;
    let mut vbo = 0u32;
    let mut ebo = 0u32;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (sky.vertices.len() * 44) as isize,
            sky.vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (sky.indices.len() * 4) as isize,
            sky.indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 44, ptr::null());
        gl::BindVertexArray(0);
    }
    (vao, sky.indices.len() as u32)
}

fn compile_shader_program(vert: &str, frag: &str) -> Result<u32, OpenGLError> {
    let vert_id = compile_shader(gl::VERTEX_SHADER, vert)?;
    let frag_id = compile_shader(gl::FRAGMENT_SHADER, frag)?;
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vert_id);
        gl::AttachShader(program, frag_id);
        gl::LinkProgram(program);
        let mut ok = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut ok);
        if ok == 0 {
            return Err(OpenGLError::Shader(get_program_log(program)));
        }
        gl::DeleteShader(vert_id);
        gl::DeleteShader(frag_id);
        Ok(program)
    }
}

fn create_hud_panel_vao() -> (u32, u32) {
    let mut vao = 0u32;
    let mut vbo = 0u32;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 8192, ptr::null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindVertexArray(0);
    }
    (vao, vbo)
}

fn push_quad(verts: &mut Vec<f32>, x0: f32, y0: f32, x1: f32, y1: f32) {
    verts.extend_from_slice(&[x0, y0, x1, y0, x1, y1, x0, y0, x1, y1, x0, y1]);
}

fn draw_hud_panels(r: &OpenGLRenderer, hud: &HudState) {
    let mut verts: Vec<f32> = Vec::with_capacity(512);

    // Barra inferior do inventario
    push_quad(&mut verts, -0.52, -0.97, 0.52, -0.78);
    let slots: [(u32, [f32; 4]); 5] = [
        (hud.fence_posts, [0.45, 0.3, 0.15, 0.95]),
        (hud.dirt_blocks, [0.42, 0.28, 0.14, 0.95]),
        (hud.stone_blocks, [0.55, 0.53, 0.5, 0.95]),
        (hud.wall_blocks, [0.48, 0.46, 0.44, 0.95]),
        (hud.wood_walls, [0.42, 0.3, 0.16, 0.95]),
    ];
    let slot_w = 0.078;
    let gap = 0.01;
    let total = 5.0 * slot_w + 4.0 * gap;
    let mut x = -total / 2.0;
    let y0 = -0.94;
    let y1 = -0.81;

    for (i, (count, color)) in slots.iter().enumerate() {
        let x0 = x;
        let x1 = x + slot_w;
        let selected = i as u8 == hud.hotbar_index;
        if selected {
            push_quad(&mut verts, x0 - 0.006, y0 - 0.012, x1 + 0.006, y1 + 0.006);
        }
        push_quad(&mut verts, x0, y0, x1, y1);
        let fill = (*count as f32 / 32.0).clamp(0.0, 1.0);
        if fill > 0.01 {
            let fy0 = y0 + 0.008;
            let fy1 = fy0 + (y1 - y0 - 0.016) * fill;
            push_quad(&mut verts, x0 + 0.008, fy0, x1 - 0.008, fy1);
        }
        x += slot_w + gap;
        let _ = color;
    }

    // Indicador dia/noite (sol ou lua)
    let hour_norm = hud.day_hour / 24.0;
    let sun_x = -0.85 + hour_norm * 1.7;
    let sun_y = 0.88 - (hour_norm * std::f32::consts::TAU - 1.2).sin().abs() * 0.12;
    let r_sun = 0.028;
    for s in 0..12 {
        let a0 = s as f32 / 12.0 * std::f32::consts::TAU;
        let a1 = (s + 1) as f32 / 12.0 * std::f32::consts::TAU;
        verts.extend_from_slice(&[
            sun_x,
            sun_y,
            sun_x + a0.cos() * r_sun,
            sun_y + a0.sin() * r_sun,
            sun_x + a1.cos() * r_sun,
            sun_y + a1.sin() * r_sun,
        ]);
    }

    // Painel de status (ovelhas)
    push_quad(&mut verts, -0.88, 0.9, -0.55, 0.97);
    push_quad(&mut verts, -0.86, 0.905, -0.57, 0.915);
    let herd_fill = (hud.sheep_herded as f32 / hud.sheep_alive.max(1) as f32).min(1.0);
    push_quad(
        &mut verts,
        -0.86,
        0.905,
        -0.86 + 0.27 * herd_fill,
        0.915,
    );

    if verts.is_empty() {
        return;
    }

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::UseProgram(r.hud_shader);
        gl::BindVertexArray(r.hud_panel_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, r.hud_panel_vbo);
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            0,
            (verts.len() * 4) as isize,
            verts.as_ptr() as *const _,
        );

        let mut offset = 0i32;
        let bg_verts = 6i32;
        set_hud_color(r.hud_shader, [0.05, 0.05, 0.08, 0.72]);
        gl::DrawArrays(gl::TRIANGLES, offset, bg_verts);
        offset += bg_verts;

        for (i, (_count, color)) in slots.iter().enumerate() {
            let selected = i as u8 == hud.hotbar_index;
            if selected {
                set_hud_color(r.hud_shader, [1.0, 0.92, 0.3, 0.95]);
                gl::DrawArrays(gl::TRIANGLES, offset, 6);
                offset += 6;
            }
            set_hud_color(r.hud_shader, *color);
            gl::DrawArrays(gl::TRIANGLES, offset, 6);
            offset += 6;
            let fill = (*_count as f32 / 32.0).clamp(0.0, 1.0);
            if fill > 0.01 {
                set_hud_color(r.hud_shader, [0.2, 0.95, 0.35, 0.85]);
                gl::DrawArrays(gl::TRIANGLES, offset, 6);
                offset += 6;
            }
        }

        if hud.is_night {
            set_hud_color(r.hud_shader, [0.85, 0.88, 1.0, 0.95]);
        } else {
            set_hud_color(r.hud_shader, [1.0, 0.88, 0.25, 0.95]);
        }
        gl::DrawArrays(gl::TRIANGLES, offset, 36);
        offset += 36;

        set_hud_color(r.hud_shader, [0.08, 0.1, 0.14, 0.75]);
        gl::DrawArrays(gl::TRIANGLES, offset, 6);
        offset += 6;
        set_hud_color(r.hud_shader, [0.35, 0.75, 1.0, 0.9]);
        gl::DrawArrays(gl::TRIANGLES, offset, 6);
        offset += 6;
        set_hud_color(r.hud_shader, [0.2, 1.0, 0.45, 0.9]);
        gl::DrawArrays(gl::TRIANGLES, offset, 6);

        draw_radar(r, hud);
        draw_trade_panel(r, hud);

        gl::BindVertexArray(0);
        gl::Disable(gl::BLEND);
    }
}

fn draw_trade_panel(r: &OpenGLRenderer, hud: &HudState) {
    if !hud.trade_visible {
        return;
    }
    let mut verts: Vec<f32> = Vec::with_capacity(128);
    push_quad(&mut verts, -0.38, 0.12, 0.38, 0.52);
    let offers = crate::game::VENDOR_OFFERS;
    let row_h = 0.075;
    let mut y = 0.46;
    for (i, offer) in offers.iter().enumerate() {
        let selected = i == hud.trade_selection;
        if selected {
            push_quad(&mut verts, -0.36, y - 0.01, 0.36, y + row_h - 0.01);
        }
        y -= row_h;
    }
    unsafe {
        gl::UseProgram(r.hud_shader);
        gl::BindVertexArray(r.hud_panel_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, r.hud_panel_vbo);
        if !verts.is_empty() {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (verts.len() * 4) as isize,
                verts.as_ptr() as *const _,
            );
            set_hud_color(r.hud_shader, [0.12, 0.1, 0.08, 0.88]);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            let mut off = 6i32;
            for (i, _) in offers.iter().enumerate() {
                if i == hud.trade_selection {
                    set_hud_color(r.hud_shader, [0.85, 0.65, 0.2, 0.55]);
                    gl::DrawArrays(gl::TRIANGLES, off, 6);
                }
                off += 6;
            }
        }
    }
    let _ = offers;
}

fn draw_radar(r: &OpenGLRenderer, hud: &HudState) {
    let rcx = 0.82f32;
    let rcy = 0.78f32;
    let rr = 0.105f32;
    let mut verts: Vec<f32> = Vec::with_capacity(256);

    for i in 0..20 {
        let a0 = i as f32 / 20.0 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / 20.0 * std::f32::consts::TAU;
        verts.extend_from_slice(&[
            rcx,
            rcy,
            rcx + a0.cos() * rr,
            rcy + a0.sin() * rr,
            rcx + a1.cos() * rr,
            rcy + a1.sin() * rr,
        ]);
    }
    verts.extend_from_slice(&[rcx - 0.008, rcy - 0.02, rcx + 0.008, rcy - 0.02]);
    verts.extend_from_slice(&[rcx, rcy - 0.02, rcx, rcy + 0.025]);

    let mut blip_verts: Vec<f32> = Vec::new();
    for &(rel, dist_n, kind) in &hud.radar_blips {
        let ang = rel * std::f32::consts::PI;
        let rad = dist_n.clamp(0.05, 1.0) * rr * 0.88;
        let bx = rcx + ang.sin() * rad;
        let by = rcy + ang.cos() * rad;
        let s = if kind == 1 { 0.014 } else { 0.011 };
        push_quad(&mut blip_verts, bx - s, by - s, bx + s, by + s);
    }

    unsafe {
        gl::UseProgram(r.hud_shader);
        gl::BindVertexArray(r.hud_panel_vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, r.hud_panel_vbo);

        if !verts.is_empty() {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (verts.len() * 4) as isize,
                verts.as_ptr() as *const _,
            );
            set_hud_color(r.hud_shader, [0.15, 0.2, 0.25, 0.82]);
            gl::DrawArrays(gl::TRIANGLES, 0, 60);
            set_hud_color(r.hud_shader, [0.75, 0.8, 0.85, 0.9]);
            gl::LineWidth(1.5);
            gl::DrawArrays(gl::LINE_LOOP, 0, 20);
            set_hud_color(r.hud_shader, [0.3, 1.0, 0.45, 0.95]);
            gl::DrawArrays(gl::LINES, 60, 4);
        }

        if !blip_verts.is_empty() {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (blip_verts.len() * 4) as isize,
                blip_verts.as_ptr() as *const _,
            );
            let mut off = 0i32;
            for &(rel, _, kind) in &hud.radar_blips {
                let _ = rel;
                let color = match kind {
                    0 => [0.95, 0.93, 0.88, 1.0],
                    1 => [0.85, 0.55, 0.35, 1.0],
                    2 => [0.75, 0.7, 0.5, 1.0],
                    3 => [1.0, 0.35, 0.2, 1.0],
                    _ => [0.45, 0.9, 0.5, 1.0],
                };
                set_hud_color(r.hud_shader, color);
                gl::DrawArrays(gl::TRIANGLES, off, 6);
                off += 6;
            }
        }

        if hud.nearest_interact_m < 90.0 {
            let pulse = 0.55 + 0.45 * (hud.hud_time * 3.5).sin().abs();
            let ring = rr * (hud.nearest_interact_m / 90.0).clamp(0.12, 0.95);
            let mut ring_verts: Vec<f32> = Vec::with_capacity(64);
            for i in 0..16 {
                let a0 = i as f32 / 16.0 * std::f32::consts::TAU;
                let a1 = (i + 1) as f32 / 16.0 * std::f32::consts::TAU;
                ring_verts.extend_from_slice(&[
                    rcx + a0.cos() * ring,
                    rcy + a0.sin() * ring,
                    rcx + a1.cos() * ring,
                    rcy + a1.sin() * ring,
                ]);
            }
            if !ring_verts.is_empty() {
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (ring_verts.len() * 4) as isize,
                    ring_verts.as_ptr() as *const _,
                );
                set_hud_color(r.hud_shader, [0.3, 1.0, 0.45, 0.35 * pulse]);
                gl::LineWidth(2.0);
                gl::DrawArrays(gl::LINE_STRIP, 0, 32);
            }
        }
    }
}

fn compile_hud_shader() -> Result<u32, OpenGLError> {
    compile_shader_program(
        r#"#version 330 core
layout(location=0) in vec2 aPos; uniform vec4 uColor;
void main(){ gl_Position=vec4(aPos,0.0,1.0); }"#,
        r#"#version 330 core
uniform vec4 uColor; out vec4 FragColor;
void main(){ FragColor=uColor; }"#,
    )
}

fn create_hud_vao() -> u32 {
    let mut verts: Vec<f32> = Vec::new();

    // Cruz com gap (8 vértices)
    let gap = 0.012;
    let arm = 0.045;
    verts.extend_from_slice(&[-arm, 0.0, -gap, 0.0, gap, 0.0, arm, 0.0]);
    verts.extend_from_slice(&[0.0, -arm, 0.0, -gap, 0.0, gap, 0.0, arm]);

    // Anel de mira (32 segmentos)
    let ring_r = 0.028;
    for i in 0..32 {
        let a = i as f32 / 32.0 * std::f32::consts::TAU;
        verts.push(a.cos() * ring_r);
        verts.push(a.sin() * ring_r);
    }

    // Poste inferior (mira de ferro)
    verts.extend_from_slice(&[0.0, -0.006, 0.0, -0.032]);

    // Ponto central
    verts.extend_from_slice(&[0.0, 0.0]);

    // Flash do cano (4 vértices)
    verts.extend_from_slice(&[0.55, -0.75, 0.65, -0.85]);

    // Flash de acerto (8 vértices)
    verts.extend_from_slice(&[
        -0.04, -0.04, 0.04, -0.04, 0.04, -0.04, 0.04, 0.04, 0.04, -0.04, 0.04, -0.04, -0.04,
        0.04, -0.04, -0.04,
    ]);

    let mut vao = 0u32;
    let mut vbo = 0u32;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (verts.len() * 4) as isize,
            verts.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindVertexArray(0);
    }
    vao
}

fn compile_line_shader() -> Result<u32, OpenGLError> {
    compile_shader_program(
        r#"#version 330 core
layout(location=0) in vec3 aPos;
uniform mat4 uMVP;
void main(){ gl_Position = uMVP * vec4(aPos, 1.0); }"#,
        r#"#version 330 core
uniform vec4 uColor; out vec4 FragColor;
void main(){ FragColor = uColor; }"#,
    )
}

fn create_line_vao() -> (u32, u32) {
    let mut vao = 0u32;
    let mut vbo = 0u32;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 4096 * 12, ptr::null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::BindVertexArray(0);
    }
    (vao, vbo)
}

fn compile_shader(kind: u32, source: &str) -> Result<u32, OpenGLError> {
    unsafe {
        let shader = gl::CreateShader(kind);
        let cstr = CString::new(source).unwrap();
        gl::ShaderSource(shader, 1, &cstr.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        let mut ok = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut ok);
        if ok == 0 {
            return Err(OpenGLError::Shader(get_shader_log(shader)));
        }
        Ok(shader)
    }
}

unsafe fn bind_texture(gl_id: Option<&u32>, unit: u32) {
    gl::ActiveTexture(gl::TEXTURE0 + unit);
    gl::BindTexture(gl::TEXTURE_2D, gl_id.copied().unwrap_or(0));
}

unsafe fn set_mat4(program: u32, name: &str, mat: Mat4) {
    let cname = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, cname.as_ptr());
    if loc >= 0 {
        gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat.to_cols_array().as_ptr());
    }
}

unsafe fn set_vec2(program: u32, name: &str, v: [f32; 2]) {
    let cname = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, cname.as_ptr());
    if loc >= 0 {
        gl::Uniform2fv(loc, 1, v.as_ptr());
    }
}

unsafe fn set_vec3(program: u32, name: &str, v: [f32; 3]) {
    let cname = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, cname.as_ptr());
    if loc >= 0 {
        gl::Uniform3fv(loc, 1, v.as_ptr());
    }
}

unsafe fn set_float(program: u32, name: &str, v: f32) {
    let cname = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, cname.as_ptr());
    if loc >= 0 {
        gl::Uniform1f(loc, v);
    }
}

unsafe fn set_int(program: u32, name: &str, v: i32) {
    let cname = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, cname.as_ptr());
    if loc >= 0 {
        gl::Uniform1i(loc, v);
    }
}

unsafe fn set_hud_color(program: u32, rgba: [f32; 4]) {
    let cname = CString::new("uColor").unwrap();
    let loc = gl::GetUniformLocation(program, cname.as_ptr());
    if loc >= 0 {
        gl::Uniform4fv(loc, 1, rgba.as_ptr());
    }
}

unsafe fn get_shader_log(shader: u32) -> String {
    let mut len = 0;
    gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
    let mut buf = vec![0u8; len as usize];
    let mut written = 0;
    gl::GetShaderInfoLog(shader, len, &mut written, buf.as_mut_ptr() as *mut _);
    String::from_utf8_lossy(&buf).to_string()
}

unsafe fn get_program_log(program: u32) -> String {
    let mut len = 0;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut len);
    let mut buf = vec![0u8; 512];
    let mut written = 0;
    gl::GetProgramInfoLog(program, 512, &mut written, buf.as_mut_ptr() as *mut _);
    String::from_utf8_lossy(&buf).to_string()
}
