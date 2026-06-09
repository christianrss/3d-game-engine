//! Código-fonte dos shaders — GLSL (OpenGL/Vulkan) e HLSL (DirectX).
//!
//! Manter os shaders como strings facilita o estudo: você vê exatamente
//! o que a GPU executa.

/// Vertex shader GLSL 330 — OpenGL 3.3
///
/// OpenGL 3.3 **não suporta** `layout(location=N)` em uniforms.
/// Os nomes são resolvidos em runtime via `glGetUniformLocation`.
pub const VERTEX_GLSL_GL33: &str = r#"#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aColor;
layout(location = 2) in vec3 aNormal;

uniform mat4 uMVP;
uniform mat4 uModel;
uniform vec3 uLightDir;

out vec3 vColor;
out vec3 vNormal;
out vec3 vWorldPos;

void main() {
    vec4 world = uModel * vec4(aPos, 1.0);
    vWorldPos = world.xyz;
    vNormal = mat3(uModel) * aNormal;
    vColor = aColor;
    gl_Position = uMVP * vec4(aPos, 1.0);
}
"#;

/// Fragment shader GLSL 330 — OpenGL 3.3
pub const FRAGMENT_GLSL_GL33: &str = r#"#version 330 core

in vec3 vColor;
in vec3 vNormal;
in vec3 vWorldPos;

out vec4 FragColor;

uniform vec3 uLightDir;

void main() {
    vec3 norm = normalize(vNormal);
    vec3 light = normalize(-uLightDir);
    float diff = max(dot(norm, light), 0.15);
    vec3 lit = vColor * diff;
    FragColor = vec4(lit, 1.0);
}
"#;

/// Vertex shader GLSL 450 — Vulkan (suporta layout em uniforms)
pub const VERTEX_GLSL: &str = r#"#version 450 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aColor;
layout(location = 2) in vec3 aNormal;

layout(location = 0) uniform mat4 uMVP;
layout(location = 1) uniform mat4 uModel;
layout(location = 2) uniform vec3 uLightDir;

out vec3 vColor;
out vec3 vNormal;
out vec3 vWorldPos;

void main() {
    vec4 world = uModel * vec4(aPos, 1.0);
    vWorldPos = world.xyz;
    vNormal = mat3(uModel) * aNormal;
    vColor = aColor;
    gl_Position = uMVP * vec4(aPos, 1.0);
}
"#;

/// Fragment shader GLSL 450 — Vulkan
pub const FRAGMENT_GLSL: &str = r#"#version 450 core

in vec3 vColor;
in vec3 vNormal;
in vec3 vWorldPos;

out vec4 FragColor;

layout(location = 2) uniform vec3 uLightDir;

void main() {
    vec3 norm = normalize(vNormal);
    vec3 light = normalize(-uLightDir);
    float diff = max(dot(norm, light), 0.15);
    vec3 lit = vColor * diff;
    FragColor = vec4(lit, 1.0);
}
"#;

/// Vertex shader HLSL (DirectX 11)
#[cfg(all(feature = "directx", target_os = "windows"))]
pub const VERTEX_HLSL: &str = r#"
cbuffer Transform : register(b0) {
    float4x4 mvp;
    float4x4 model;
    float3 lightDir;
    float _pad;
};

struct VSIn {
    float3 pos    : POSITION;
    float3 color  : COLOR;
    float3 normal : NORMAL;
};

struct VSOut {
    float4 pos      : SV_POSITION;
    float3 color    : COLOR;
    float3 normal   : NORMAL;
    float3 worldPos : TEXCOORD0;
};

VSOut main(VSIn input) {
    VSOut o;
    float4 world = mul(model, float4(input.pos, 1.0));
    o.worldPos = world.xyz;
    o.normal = mul((float3x3)model, input.normal);
    o.color = input.color;
    o.pos = mul(mvp, float4(input.pos, 1.0));
    return o;
}
"#;

/// Fragment shader HLSL (DirectX 11)
#[cfg(all(feature = "directx", target_os = "windows"))]
pub const FRAGMENT_HLSL: &str = r#"
cbuffer Transform : register(b0) {
    float4x4 mvp;
    float4x4 model;
    float3 lightDir;
    float _pad;
};

struct PSIn {
    float4 pos      : SV_POSITION;
    float3 color    : COLOR;
    float3 normal   : NORMAL;
    float3 worldPos : TEXCOORD0;
};

float4 main(PSIn input) : SV_Target {
    float3 norm = normalize(input.normal);
    float3 light = normalize(-lightDir);
    float diff = max(dot(norm, light), 0.15);
    return float4(input.color * diff, 1.0);
}
"#;

/// Direção da luz do sol no deserto (normalizada).
pub const LIGHT_DIRECTION: [f32; 3] = [-0.4, -0.8, -0.3];
