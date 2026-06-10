//! Renderer OpenGL — PBR, shadow maps, bloom, HUD.

use crate::graphics::backend::GfxBackend;
use crate::graphics::opengl::context::GlContext;
use crate::graphics::renderer::HudState;
use crate::graphics::shaders::{
    FRAGMENT_GLSL_GL33, LIGHT_DIRECTION, PARTICLE_FRAGMENT_GLSL_GL33, PARTICLE_VERTEX_GLSL_GL33,
    POST_FRAGMENT_GLSL_GL33, POST_VERTEX_GLSL_GL33, SHADOW_FRAGMENT_GLSL_GL33,
    SHADOW_VERTEX_GLSL_GL33, SKY_FRAGMENT_GLSL_GL33, SKY_VERTEX_GLSL_GL33, VERTEX_GLSL_GL33,
};
use crate::graphics::ParticleDraw;
use crate::graphics::{Camera, Color, DrawMaterial, GpuMesh, GpuTexture, Mesh, TextureData};
use crate::math::{Mat4, Vec3};
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;

const SHADOW_SIZE: i32 = 2048;

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
    line_shader: u32,
    line_vao: u32,
    line_vbo: u32,
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
        let line_shader = compile_line_shader()?;
        let (line_vao, line_vbo) = create_line_vao();
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
            line_shader,
            line_vao,
            line_vbo,
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

    fn create_framebuffers(&self) -> Result<Framebuffers, OpenGLError> {
        let mut scene_fbo = 0u32;
        let mut scene_color = 0u32;
        let mut scene_depth = 0u32;
        let mut shadow_fbo = 0u32;
        let mut shadow_depth = 0u32;

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

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Ok(Framebuffers {
            scene_fbo,
            scene_color,
            scene_depth,
            shadow_fbo,
            shadow_depth,
        })
    }

    fn compute_light_space(&self, camera: &Camera) -> Mat4 {
        let focus = camera.position + camera.forward() * 35.0;
        let light_dir = Vec3::from_array(LIGHT_DIRECTION).normalize();
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
                set_hud_color(self.hud_shader, [1.0, 1.0, 1.0, 0.92]);
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

        let mut verts = Vec::with_capacity(particles.len() * 5);
        for p in particles {
            let lp = vm_transform.transform_point3(crate::math::Vec3::from_array(p.pos));
            verts.extend_from_slice(&[lp.x, lp.y, lp.z, p.size, p.alpha]);
        }
        self.draw_particle_verts(&vp, &verts);
    }

    pub fn draw_world_particles(&self, camera: &Camera, particles: &[ParticleDraw]) {
        if particles.is_empty() {
            return;
        }
        let vp = camera.view_projection();
        let mut verts = Vec::with_capacity(particles.len() * 5);
        for p in particles {
            verts.extend_from_slice(&[p.pos[0], p.pos[1], p.pos[2], p.size, p.alpha]);
        }
        self.draw_particle_verts(&vp, &verts);
    }

    fn draw_particle_verts(&self, mvp: &Mat4, verts: &[f32]) {
        if verts.is_empty() {
            return;
        }
        let count = verts.len() / 5;
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
            set_vec3(self.shader, "uLightDir", LIGHT_DIRECTION);
            set_vec3(self.shader, "uCameraPos", camera.position.to_array());
            set_vec3(
                self.shader,
                "uFogColor",
                Color::SKY_HORIZON.to_array()[..3].try_into().unwrap(),
            );
            set_float(self.shader, "uFogDensity", fog_density);

            bind_texture(self.fb.as_ref().map(|f| &f.shadow_depth), 4);
            set_int(self.shader, "uShadowMap", 4);

            match material {
                DrawMaterial::Standard { roughness, metallic } => {
                    set_int(self.shader, "uUseAlbedo", 0);
                    set_int(self.shader, "uUseNormalMap", 0);
                    set_int(self.shader, "uUseRoughMap", 0);
                    set_int(self.shader, "uUseAOMap", 0);
                    set_float(self.shader, "uRoughness", roughness);
                    set_float(self.shader, "uMetallic", metallic);
                    set_float(self.shader, "uTiling", 1.0);
                }
                DrawMaterial::Terrain { tiling } => {
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
            DrawMaterial::Standard { .. } => 0.00006,
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
        gl::BufferData(gl::ARRAY_BUFFER, 4096 * 20, ptr::null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 20, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 1, gl::FLOAT, gl::FALSE, 20, 12 as *const _);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 1, gl::FLOAT, gl::FALSE, 20, 16 as *const _);
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
