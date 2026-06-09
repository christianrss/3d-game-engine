//! Renderer OpenGL — VAO, VBO, EBO, shaders GLSL, uniform MVP.

use crate::graphics::backend::GfxBackend;
use crate::graphics::opengl::context::GlContext;
use crate::graphics::shaders::{FRAGMENT_GLSL_GL33, LIGHT_DIRECTION, VERTEX_GLSL_GL33};
use crate::graphics::{Camera, Color, GpuMesh, Mesh};
use crate::math::Mat4;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;

/// Erros do backend OpenGL.
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

/// Renderer OpenGL 3.3 Core.
pub struct OpenGLRenderer {
    gl_ctx: GlContext,
    shader: u32,
    meshes: HashMap<u64, GlMesh>,
    next_id: u64,
    width: u32,
    height: u32,
}

impl OpenGLRenderer {
    /// Cria o renderer a partir de um contexto OpenGL já inicializado.
    pub fn from_context(gl_ctx: GlContext, width: u32, height: u32) -> Result<Self, OpenGLError> {
        gl_ctx.load_gl();

        let shader = compile_shader_program()?;

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::ClearColor(0.95, 0.75, 0.5, 1.0);
        }

        Ok(Self {
            gl_ctx,
            shader,
            meshes: HashMap::new(),
            next_id: 1,
            width,
            height,
        })
    }
}

impl GfxBackend for OpenGLRenderer {
    type Error = OpenGLError;

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.gl_ctx.resize(self.width, self.height);
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
                (mesh.vertices.len() * std::mem::size_of::<crate::graphics::Vertex>()) as isize,
                mesh.vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.indices.len() * std::mem::size_of::<u32>()) as isize,
                mesh.indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let stride = std::mem::size_of::<crate::graphics::Vertex>() as i32;

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, 12 as *const _);

            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, stride, 24 as *const _);

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

    fn begin_frame(&mut self, clear: Color) {
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            gl::ClearColor(clear.r, clear.g, clear.b, clear.a);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UseProgram(self.shader);
        }
    }

    fn draw(
        &mut self,
        gpu_mesh: &GpuMesh,
        model: Mat4,
        camera: &Camera,
    ) -> Result<(), OpenGLError> {
        let gl_mesh = self
            .meshes
            .get(&gpu_mesh.gpu_id)
            .ok_or_else(|| OpenGLError::Gl("Mesh GPU não encontrada".into()))?;

        let mvp = camera.view_projection() * model;

        unsafe {
            set_mat4(self.shader, "uMVP", mvp);
            set_mat4(self.shader, "uModel", model);
            set_vec3(self.shader, "uLightDir", LIGHT_DIRECTION);

            gl::BindVertexArray(gl_mesh.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                gl_mesh.index_count as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), OpenGLError> {
        self.gl_ctx
            .swap_buffers()
            .map_err(OpenGLError::Context)
    }
}

fn compile_shader_program() -> Result<u32, OpenGLError> {
    let vert = compile_shader(gl::VERTEX_SHADER, VERTEX_GLSL_GL33)?;
    let frag = compile_shader(gl::FRAGMENT_SHADER, FRAGMENT_GLSL_GL33)?;

    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vert);
        gl::AttachShader(program, frag);
        gl::LinkProgram(program);

        let mut ok = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut ok);
        if ok == 0 {
            let log = get_program_log(program);
            return Err(OpenGLError::Shader(format!("Link: {log}")));
        }

        gl::DeleteShader(vert);
        gl::DeleteShader(frag);
        Ok(program)
    }
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
            let log = get_shader_log(shader);
            return Err(OpenGLError::Shader(format!("Compile: {log}")));
        }
        Ok(shader)
    }
}

unsafe fn set_mat4(program: u32, name: &str, mat: Mat4) {
    let cname = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, cname.as_ptr());
    if loc >= 0 {
        gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat.to_cols_array().as_ptr());
    }
}

unsafe fn set_vec3(program: u32, name: &str, v: [f32; 3]) {
    let cname = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, cname.as_ptr());
    if loc >= 0 {
        gl::Uniform3fv(loc, 1, v.as_ptr());
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
    gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
    let mut buf = vec![0u8; len as usize];
    let mut written = 0;
    gl::GetProgramInfoLog(program, len, &mut written, buf.as_mut_ptr() as *mut _);
    String::from_utf8_lossy(&buf).to_string()
}
