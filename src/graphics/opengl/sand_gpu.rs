//! Areia GPU — física de grãos via Transform Feedback (OpenGL 3.3).

use crate::assets::sample_desert_height;
use crate::graphics::opengl::renderer::OpenGLError;
use crate::graphics::Camera;
use crate::math::Vec3;
use std::ffi::CString;
use std::mem;
use std::ptr;

const MAX_GRAINS: usize = 4096;
const HEIGHT_RES: u32 = 128;

#[repr(C)]
#[derive(Clone, Copy)]
struct Grain {
    pos: [f32; 3],
    vel: [f32; 3],
    life: f32,
    size: f32,
}

pub struct GpuSandField {
    tf_program: u32,
    render_program: u32,
    vao: u32,
    buf_a: u32,
    buf_b: u32,
    height_tex: u32,
    ping_a: bool,
    emit_head: usize,
}

const SAND_RENDER_VERT: &str = r#"#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 2) in float aLife;
layout(location = 3) in float aSize;
uniform mat4 uMVP;
out float vAlpha;
void main() {
    vAlpha = aLife;
    gl_Position = uMVP * vec4(aPos, 1.0);
    gl_PointSize = aSize * 420.0 / max(-gl_Position.w, 0.1);
}
"#;

const SAND_RENDER_FRAG: &str = r#"#version 330 core
in float vAlpha;
out vec4 FragColor;
void main() {
    vec2 c = gl_PointCoord - vec2(0.5);
    float d = dot(c, c);
    if (d > 0.25) discard;
    float soft = 1.0 - smoothstep(0.06, 0.25, d);
    FragColor = vec4(0.92, 0.78, 0.52, soft * vAlpha * 0.8);
}
"#;

const TF_VERTEX: &str = r#"#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aVel;
layout(location = 2) in float aLife;
layout(location = 3) in float aSize;

uniform float uDt;
uniform vec3 uWind;
uniform sampler2D uHeightMap;
uniform vec2 uHeightOrigin;
uniform float uHeightSize;

out vec3 outPos;
out vec3 outVel;
out float outLife;
out float outSize;

float terrainH(vec2 xz) {
    vec2 uv = (xz - uHeightOrigin) / uHeightSize;
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) return 0.0;
    return texture(uHeightMap, uv).r;
}

void main() {
    vec3 pos = aPos;
    vec3 vel = aVel;
    float life = aLife;
    float size = aSize;
    if (life > 0.01) {
        vel.y -= 5.5 * uDt;
        vel += uWind * uDt;
        vel *= 1.0 - 1.8 * uDt;
        pos += vel * uDt;
        float ground = terrainH(pos.xz) + 0.03;
        if (pos.y < ground) {
            pos.y = ground;
            vel.xz *= 0.35;
            vel.y = abs(vel.y) * 0.12;
            life -= 0.4;
        }
        life -= uDt * 0.22;
    }
    outPos = pos;
    outVel = vel;
    outLife = max(life, 0.0);
    outSize = size;
    gl_Position = vec4(pos, 1.0);
}
"#;

impl GpuSandField {
    pub fn new() -> Result<Self, OpenGLError> {
        let tf_program = compile_tf_program(TF_VERTEX)?;
        let render_program = compile_render_program(SAND_RENDER_VERT, SAND_RENDER_FRAG)?;
        unsafe {
            let mut vao = 0u32;
            let mut buf_a = 0u32;
            let mut buf_b = 0u32;
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut buf_a);
            gl::GenBuffers(1, &mut buf_b);

            let grains = vec![
                Grain {
                    pos: [0.0; 3],
                    vel: [0.0; 3],
                    life: 0.0,
                    size: 0.04,
                };
                MAX_GRAINS
            ];
            let bytes = (grains.len() * mem::size_of::<Grain>()) as isize;

            for buf in [buf_a, buf_b] {
                gl::BindBuffer(gl::ARRAY_BUFFER, buf);
                gl::BufferData(gl::ARRAY_BUFFER, bytes, grains.as_ptr() as *const _, gl::DYNAMIC_COPY);
            }

            bind_grain_vao(vao, buf_a);
            let height_tex = upload_heightmap()?;

            Ok(Self {
                tf_program,
                render_program,
                vao,
                buf_a,
                buf_b,
                height_tex,
                ping_a: true,
                emit_head: 0,
            })
        }
    }

    pub fn update(&mut self, dt: f32, wind: Vec3) {
        unsafe {
            gl::Enable(gl::RASTERIZER_DISCARD);
            gl::UseProgram(self.tf_program);
            set_float(self.tf_program, "uDt", dt);
            set_vec3(self.tf_program, "uWind", wind);
            set_vec2(self.tf_program, "uHeightOrigin", [-110.0, -110.0]);
            set_float(self.tf_program, "uHeightSize", 220.0);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.height_tex);
            set_int(self.tf_program, "uHeightMap", 0);

            let (src, dst) = if self.ping_a {
                (self.buf_a, self.buf_b)
            } else {
                (self.buf_b, self.buf_a)
            };

            bind_grain_vao(self.vao, src);
            gl::BindBufferBase(gl::TRANSFORM_FEEDBACK_BUFFER, 0, dst);
            gl::BeginTransformFeedback(gl::POINTS);
            gl::DrawArrays(gl::POINTS, 0, MAX_GRAINS as i32);
            gl::EndTransformFeedback();
            gl::BindBufferBase(gl::TRANSFORM_FEEDBACK_BUFFER, 0, 0);
            gl::Disable(gl::RASTERIZER_DISCARD);
            gl::UseProgram(0);
            self.ping_a = !self.ping_a;
        }
    }

    pub fn emit(&mut self, pos: Vec3, vel: Vec3, count: usize) {
        let buf = if self.ping_a {
            self.buf_a
        } else {
            self.buf_b
        };
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, buf);
            for i in 0..count {
                let spread = (i as f32 * 1.31).sin();
                let g = Grain {
                    pos: [pos.x + spread * 0.15, pos.y, pos.z + spread * 0.12],
                    vel: [vel.x + spread * 0.3, vel.y, vel.z + spread * 0.2],
                    life: 1.0,
                    size: 0.04 + (i % 3) as f32 * 0.01,
                };
                let offset = (self.emit_head % MAX_GRAINS) * mem::size_of::<Grain>();
                gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    offset as isize,
                    mem::size_of::<Grain>() as isize,
                    &g as *const _ as *const _,
                );
                self.emit_head += 1;
            }
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn draw(&self, camera: &Camera) {
        let buf = if self.ping_a {
            self.buf_b
        } else {
            self.buf_a
        };
        let vp = camera.view_projection();
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::DepthMask(gl::FALSE);
            gl::UseProgram(self.render_program);
            set_mat4(self.render_program, "uMVP", vp);
            bind_grain_vao(self.vao, buf);
            gl::DrawArrays(gl::POINTS, 0, MAX_GRAINS as i32);
            gl::BindVertexArray(0);
            gl::DepthMask(gl::TRUE);
            gl::Disable(gl::BLEND);
        }
    }
}

fn compile_render_program(vert: &str, frag: &str) -> Result<u32, OpenGLError> {
    unsafe {
        let vs = compile_shader(gl::VERTEX_SHADER, vert)?;
        let fs = compile_shader(gl::FRAGMENT_SHADER, frag)?;
        let prog = gl::CreateProgram();
        gl::AttachShader(prog, vs);
        gl::AttachShader(prog, fs);
        gl::LinkProgram(prog);
        let mut ok = 0;
        gl::GetProgramiv(prog, gl::LINK_STATUS, &mut ok);
        if ok == 0 {
            return Err(OpenGLError::Shader("Sand render link failed".into()));
        }
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
        Ok(prog)
    }
}

unsafe fn compile_shader(kind: u32, source: &str) -> Result<u32, OpenGLError> {
    let shader = gl::CreateShader(kind);
    let c = CString::new(source).unwrap();
    gl::ShaderSource(shader, 1, &c.as_ptr(), ptr::null());
    gl::CompileShader(shader);
    let mut ok = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut ok);
    if ok == 0 {
        return Err(OpenGLError::Shader("Sand shader compile failed".into()));
    }
    Ok(shader)
}

fn bind_grain_vao(vao: u32, buf: u32) {
    unsafe {
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, buf);
        let stride = mem::size_of::<Grain>() as i32;
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, 12 as *const _);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 1, gl::FLOAT, gl::FALSE, stride, 24 as *const _);
        gl::EnableVertexAttribArray(3);
        gl::VertexAttribPointer(3, 1, gl::FLOAT, gl::FALSE, stride, 28 as *const _);
    }
}

fn compile_tf_program(vertex_src: &str) -> Result<u32, OpenGLError> {
    unsafe {
        let vs = gl::CreateShader(gl::VERTEX_SHADER);
        let c = CString::new(vertex_src).unwrap();
        gl::ShaderSource(vs, 1, &c.as_ptr(), ptr::null());
        gl::CompileShader(vs);
        let prog = gl::CreateProgram();
        gl::AttachShader(prog, vs);
        let names = [
            CString::new("outPos").unwrap(),
            CString::new("outVel").unwrap(),
            CString::new("outLife").unwrap(),
            CString::new("outSize").unwrap(),
        ];
        let ptrs: Vec<*const i8> = names.iter().map(|s| s.as_ptr()).collect();
        gl::TransformFeedbackVaryings(prog, 4, ptrs.as_ptr(), gl::INTERLEAVED_ATTRIBS);
        gl::LinkProgram(prog);
        let mut ok = 0;
        gl::GetProgramiv(prog, gl::LINK_STATUS, &mut ok);
        if ok == 0 {
            return Err(OpenGLError::Shader("TF sand link failed".into()));
        }
        gl::DeleteShader(vs);
        Ok(prog)
    }
}

unsafe fn upload_heightmap() -> Result<u32, OpenGLError> {
    let mut pixels = vec![0f32; (HEIGHT_RES * HEIGHT_RES) as usize];
    let half = 110.0;
    let step = 220.0 / HEIGHT_RES as f32;
    for z in 0..HEIGHT_RES {
        for x in 0..HEIGHT_RES {
            let wx = -half + x as f32 * step;
            let wz = -half + z as f32 * step;
            pixels[(z * HEIGHT_RES + x) as usize] = sample_desert_height(wx, wz);
        }
    }
    let mut tex = 0u32;
    gl::GenTextures(1, &mut tex);
    gl::BindTexture(gl::TEXTURE_2D, tex);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::R32F as i32,
        HEIGHT_RES as i32,
        HEIGHT_RES as i32,
        0,
        gl::RED,
        gl::FLOAT,
        pixels.as_ptr() as *const _,
    );
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    gl::BindTexture(gl::TEXTURE_2D, 0);
    Ok(tex)
}

unsafe fn set_mat4(program: u32, name: &str, mat: glam::Mat4) {
    let c = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, c.as_ptr());
    if loc >= 0 {
        gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat.to_cols_array().as_ptr());
    }
}

unsafe fn set_float(program: u32, name: &str, v: f32) {
    let c = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, c.as_ptr());
    if loc >= 0 {
        gl::Uniform1f(loc, v);
    }
}

unsafe fn set_vec2(program: u32, name: &str, v: [f32; 2]) {
    let c = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, c.as_ptr());
    if loc >= 0 {
        gl::Uniform2fv(loc, 1, v.as_ptr());
    }
}

unsafe fn set_vec3(program: u32, name: &str, v: Vec3) {
    let c = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, c.as_ptr());
    if loc >= 0 {
        gl::Uniform3f(loc, v.x, v.y, v.z);
    }
}

unsafe fn set_int(program: u32, name: &str, v: i32) {
    let c = CString::new(name).unwrap();
    let loc = gl::GetUniformLocation(program, c.as_ptr());
    if loc >= 0 {
        gl::Uniform1i(loc, v);
    }
}
