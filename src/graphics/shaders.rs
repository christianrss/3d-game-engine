//! Shaders GLSL 330 — PBR, sombras, pós-processo.

pub const LIGHT_DIRECTION: [f32; 3] = [-0.42, -0.78, -0.38];

/// Vertex principal — PBR + shadow map
pub const VERTEX_GLSL_GL33: &str = r#"#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aUV;
layout(location = 3) in vec3 aColor;

uniform mat4 uMVP;
uniform mat4 uModel;
uniform mat4 uLightSpaceMatrix;

out vec3 vColor;
out vec3 vNormal;
out vec3 vWorldPos;
out vec2 vUV;
out vec4 vFragPosLightSpace;

void main() {
    vec4 world = uModel * vec4(aPos, 1.0);
    vWorldPos = world.xyz;
    vNormal = mat3(transpose(inverse(uModel))) * aNormal;
    vColor = aColor;
    vUV = aUV;
    vFragPosLightSpace = uLightSpaceMatrix * world;
    gl_Position = uMVP * vec4(aPos, 1.0);
}
"#;

/// Fragment PBR Cook-Torrance GGX + sombras PCF + névoa
pub const FRAGMENT_GLSL_GL33: &str = r#"#version 330 core

in vec3 vColor;
in vec3 vNormal;
in vec3 vWorldPos;
in vec2 vUV;
in vec4 vFragPosLightSpace;

out vec4 FragColor;

uniform vec3 uLightDir;
uniform vec3 uCameraPos;
uniform vec3 uFogColor;
uniform float uFogDensity;
uniform float uRoughness;
uniform float uMetallic;
uniform int uUseAlbedo;
uniform int uUseNormalMap;
uniform int uUseRoughMap;
uniform int uUseAOMap;
uniform float uTiling;
uniform float uTime;
uniform int uMatType; // 0=color 1=terrain sand 2=rock triplanar

uniform sampler2D uAlbedo;
uniform sampler2D uNormalMap;
uniform sampler2D uRoughMap;
uniform sampler2D uAOMap;
uniform sampler2D uShadowMap;

const float PI = 3.14159265;

float distributionGGX(vec3 N, vec3 H, float rough) {
    float a = rough * rough;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return a2 / max(PI * denom * denom, 0.0001);
}

float geometrySchlickGGX(float NdotV, float rough) {
    float r = rough + 1.0;
    float k = (r * r) / 8.0;
    return NdotV / max(NdotV * (1.0 - k) + k, 0.0001);
}

float geometrySmith(vec3 N, vec3 V, vec3 L, float rough) {
    float ggx1 = geometrySchlickGGX(max(dot(N, V), 0.0), rough);
    float ggx2 = geometrySchlickGGX(max(dot(N, L), 0.0), rough);
    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

float shadowPCF(vec4 fragPosLight) {
    vec3 proj = fragPosLight.xyz / fragPosLight.w;
    proj = proj * 0.5 + 0.5;
    if (proj.z > 1.0 || proj.x < 0.0 || proj.x > 1.0 || proj.y < 0.0 || proj.y > 1.0)
        return 0.0;

    float shadow = 0.0;
    vec2 texel = 1.0 / vec2(textureSize(uShadowMap, 0));
    float bias = 0.0025;
    for (int x = -1; x <= 1; ++x) {
        for (int y = -1; y <= 1; ++y) {
            float pcf = texture(uShadowMap, proj.xy + vec2(x, y) * texel).r;
            shadow += proj.z - bias > pcf ? 1.0 : 0.0;
        }
    }
    return shadow / 9.0;
}

vec3 aces(vec3 x) {
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), 0.0, 1.0);
}

vec3 triSample(sampler2D tex, vec3 wp, vec3 n, float scale) {
    vec3 w = abs(n);
    w = max(w, 0.0001);
    w /= (w.x + w.y + w.z);
    vec3 cx = pow(texture(tex, wp.yz * scale).rgb, vec3(2.2));
    vec3 cy = pow(texture(tex, wp.xz * scale).rgb, vec3(2.2));
    vec3 cz = pow(texture(tex, wp.xy * scale).rgb, vec3(2.2));
    return cx * w.x + cy * w.y + cz * w.z;
}

float triSampleR(sampler2D tex, vec3 wp, vec3 n, float scale) {
    vec3 w = abs(n);
    w = max(w, 0.0001);
    w /= (w.x + w.y + w.z);
    float cx = texture(tex, wp.yz * scale).r;
    float cy = texture(tex, wp.xz * scale).r;
    float cz = texture(tex, wp.xy * scale).r;
    return cx * w.x + cy * w.y + cz * w.z;
}

vec3 sandRippleNormal(vec3 wp, vec3 baseN) {
    float wx = wp.x * 14.0 + uTime * 0.35;
    float wz = wp.z * 11.0 - uTime * 0.25;
    float ripple = sin(wx) * cos(wz) * 0.04;
    float fine = sin(wp.x * 38.0 - wp.z * 32.0 + uTime * 0.8) * 0.015;
    float dune = sin(wp.x * 0.12 + wp.z * 0.09) * 0.06;
    vec3 bump = vec3(ripple + dune, 0.0, fine);
    return normalize(baseN + bump);
}

void main() {
    vec2 uv = vUV * uTiling;
    vec3 wp = vWorldPos;

    vec3 albedo;
    float rough;
    float ao;
    vec3 N = normalize(vNormal);

    if (uMatType == 2) {
        float scale = uTiling;
        albedo = triSample(uAlbedo, wp, N, scale);
        rough = triSampleR(uRoughMap, wp, N, scale);
        ao = 1.0;
        vec3 nX = texture(uNormalMap, wp.yz * scale).rgb * 2.0 - 1.0;
        vec3 nY = texture(uNormalMap, wp.xz * scale).rgb * 2.0 - 1.0;
        vec3 nZ = texture(uNormalMap, wp.xy * scale).rgb * 2.0 - 1.0;
        vec3 w = abs(N);
        w /= (w.x + w.y + w.z);
        N = normalize(nX * w.x + nY * w.y + nZ * w.z);
    } else {
        albedo = uUseAlbedo == 1 ? pow(texture(uAlbedo, uv).rgb, vec3(2.2)) : pow(vColor, vec3(2.2));
        rough = uUseRoughMap == 1 ? texture(uRoughMap, uv).r : uRoughness;
        ao = uUseAOMap == 1 ? texture(uAOMap, uv).r : 1.0;
        if (uUseNormalMap == 1) {
            vec3 ntex = texture(uNormalMap, uv).rgb * 2.0 - 1.0;
            N = normalize(N + ntex * 0.85);
        }
        if (uMatType == 1) {
            float macro = sin(wp.x * 0.07) * cos(wp.z * 0.05) * 0.08;
            float streak = sin(wp.x * 0.4 + wp.z * 0.2 + uTime * 0.15) * 0.03;
            albedo *= 1.0 + macro + streak;
            N = sandRippleNormal(wp, N);
            rough = clamp(rough + sin(wp.x * 22.0) * 0.04, 0.5, 1.0);
        }
    }
    rough = clamp(rough, 0.04, 1.0);

    vec3 V = normalize(uCameraPos - vWorldPos);
    vec3 L = normalize(-uLightDir);
    vec3 H = normalize(V + L);

    vec3 F0 = mix(vec3(0.04), albedo, uMetallic);
    float NDF = distributionGGX(N, H, rough);
    float G = geometrySmith(N, V, L, rough);
    vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);

    vec3 kS = F;
    vec3 kD = (vec3(1.0) - kS) * (1.0 - uMetallic);
    vec3 spec = (NDF * G * F) / max(4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0), 0.001);

    float NdotL = max(dot(N, L), 0.0);
    float shadow = shadowPCF(vFragPosLightSpace);

    vec3 sunColor = vec3(1.0, 0.96, 0.88);
    vec3 ambient = albedo * 0.12 * ao;
    vec3 radiance = sunColor * (1.0 - shadow * 0.85);
    vec3 color = ambient + (kD * albedo / PI + spec) * radiance * NdotL;

    float dist = length(vWorldPos - uCameraPos);
    float fog = 1.0 - exp(-uFogDensity * dist * dist);
    color = mix(color, pow(uFogColor, vec3(2.2)), clamp(fog, 0.0, 0.9));

    color = aces(color);
    color = pow(color, vec3(1.0 / 2.2));
    FragColor = vec4(color, 1.0);
}
"#;

/// Depth-only shadow pass
pub const SHADOW_VERTEX_GLSL_GL33: &str = r#"#version 330 core
layout(location = 0) in vec3 aPos;
uniform mat4 uLightSpaceMatrix;
uniform mat4 uModel;
void main() {
    gl_Position = uLightSpaceMatrix * uModel * vec4(aPos, 1.0);
}
"#;

pub const SHADOW_FRAGMENT_GLSL_GL33: &str = r#"#version 330 core
void main() {}
"#;

/// Céu atmosférico
pub const SKY_VERTEX_GLSL_GL33: &str = r#"#version 330 core
layout(location = 0) in vec3 aPos;
uniform mat4 uViewProj;
out vec3 vDir;
void main() {
    vDir = aPos;
    vec4 clip = uViewProj * vec4(aPos, 1.0);
    gl_Position = clip.xyww;
}
"#;

pub const SKY_FRAGMENT_GLSL_GL33: &str = r#"#version 330 core
in vec3 vDir;
out vec4 FragColor;
uniform vec3 uHorizon;
uniform vec3 uZenith;
uniform vec3 uSunDir;
uniform float uNightFactor;
void main() {
    vec3 dir = normalize(vDir);
    float y = clamp(dir.y, 0.0, 1.0);
    vec3 col = mix(uHorizon, uZenith, pow(y, 0.5));
    vec3 sunDir = normalize(uSunDir);
    float sun = pow(max(dot(dir, sunDir), 0.0), 512.0);
    float glow = pow(max(dot(dir, sunDir), 0.0), 6.0) * 0.45;
    float sunStrength = mix(4.0, 0.6, uNightFactor);
    float glowStrength = mix(0.45, 0.08, uNightFactor);
    col += vec3(1.0, 0.9, 0.7) * (sun * sunStrength + glow * glowStrength);
    if (uNightFactor > 0.5) {
        vec3 moonDir = normalize(vec3(-sunDir.x, abs(sunDir.y), -sunDir.z));
        float moon = pow(max(dot(dir, moonDir), 0.0), 128.0);
        col += vec3(0.75, 0.82, 1.0) * moon * 1.2;
    }
    FragColor = vec4(col, 1.0);
}
"#;

/// Pós-processo bloom + vignette
pub const POST_VERTEX_GLSL_GL33: &str = r#"#version 330 core
layout(location = 0) in vec2 aPos;
out vec2 vUV;
void main() {
    vUV = aPos * 0.5 + 0.5;
    gl_Position = vec4(aPos, 0.0, 1.0);
}
"#;

/// Pós-processo: SSAO (screen-space, alternativa leve ao ray tracing) + bloom
pub const POST_FRAGMENT_GLSL_GL33: &str = r#"#version 330 core
in vec2 vUV;
out vec4 FragColor;
uniform sampler2D uScene;
uniform sampler2D uDepth;
uniform vec2 uTexelSize;
uniform float uNear;
uniform float uFar;

float linearizeDepth(float d) {
    float z = d * 2.0 - 1.0;
    return (2.0 * uNear * uFar) / (uFar + uNear - z * (uFar - uNear));
}

float ssao(vec2 uv) {
    float depth = linearizeDepth(texture(uDepth, uv).r);
    float ao = 0.0;
    const int S = 8;
    for (int i = 0; i < S; i++) {
        float ang = float(i) * 0.785398;
        vec2 off = vec2(cos(ang), sin(ang)) * uTexelSize * float(3 + i);
        float d2 = linearizeDepth(texture(uDepth, uv + off).r);
        if (d2 > depth + 0.8) ao += 1.0;
    }
    return 1.0 - (ao / float(S)) * 0.55;
}

void main() {
    vec3 col = texture(uScene, vUV).rgb;
    col *= ssao(vUV);
    float lum = dot(col, vec3(0.2126, 0.7152, 0.0722));
    vec3 bloom = vec3(0.0);
    if (lum > 0.75) {
        for (int x = -2; x <= 2; x++) {
            for (int y = -2; y <= 2; y++) {
                vec2 off = vec2(x, y) * uTexelSize * 2.0;
                vec3 s = texture(uScene, vUV + off).rgb;
                float l = dot(s, vec3(0.2126, 0.7152, 0.0722));
                if (l > 0.75) bloom += s;
            }
        }
        bloom /= 25.0;
    }
    col += bloom * 0.35;
    float vig = 1.0 - dot(vUV - 0.5, vUV - 0.5) * 1.8;
    col *= clamp(vig, 0.55, 1.0);
    col = pow(col, vec3(1.0 / 2.2));
    FragColor = vec4(col, 1.0);
}
"#;

pub const PARTICLE_VERTEX_GLSL_GL33: &str = r#"#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in float aSize;
layout(location = 2) in float aAlpha;
layout(location = 3) in float aKind;
uniform mat4 uMVP;
out float vAlpha;
out float vKind;
void main() {
    vAlpha = aAlpha;
    vKind = aKind;
    gl_Position = uMVP * vec4(aPos, 1.0);
    gl_PointSize = aSize * 420.0 / max(-gl_Position.w, 0.1);
}
"#;

pub const PARTICLE_FRAGMENT_GLSL_GL33: &str = r#"#version 330 core
in float vAlpha;
in float vKind;
out vec4 FragColor;
void main() {
    vec2 c = gl_PointCoord - vec2(0.5);
    float d = dot(c, c);
    if (d > 0.25) discard;
    float soft = 1.0 - smoothstep(0.08, 0.25, d);
    vec3 col;
    if (vKind > 1.5) {
        col = vec3(0.92, 0.78, 0.52);
    } else if (vKind > 0.5) {
        col = vec3(0.82, 0.68, 0.45);
    } else {
        col = vec3(0.75, 0.72, 0.68);
    }
    FragColor = vec4(col, soft * vAlpha * 0.75);
}
"#;

/// Água — ondas Gerstner + Fresnel + reflexão do céu
pub const WATER_VERTEX_GLSL_GL33: &str = r#"#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aUV;
layout(location = 3) in vec3 aColor;

uniform mat4 uMVP;
uniform mat4 uModel;
uniform float uTime;

out vec3 vWorldPos;
out vec3 vNormal;
out vec2 vUV;

vec3 gerstner(vec2 xz, float t) {
    vec3 p = vec3(xz.x, 0.0, xz.y);
    float w1 = sin(xz.x * 0.9 + t * 1.1) * 0.07;
    float w2 = sin(xz.y * 1.3 - t * 0.9) * 0.05;
    float w3 = sin((xz.x + xz.y) * 0.7 + t * 1.4) * 0.03;
    p.y = w1 + w2 + w3;
    return p;
}

void main() {
    vec4 world = uModel * vec4(aPos, 1.0);
    vec2 xz = world.xz;
    float waveY = gerstner(xz, uTime).y;
    world.y += waveY;
    vWorldPos = world.xyz;
    float dx = gerstner(xz + vec2(0.08, 0.0), uTime).y - gerstner(xz - vec2(0.08, 0.0), uTime).y;
    float dz = gerstner(xz + vec2(0.0, 0.08), uTime).y - gerstner(xz - vec2(0.0, 0.08), uTime).y;
    vNormal = normalize(vec3(-dx * 6.0, 1.0, -dz * 6.0));
    vUV = aUV;
    vec4 displaced = vec4(aPos.x, aPos.y + waveY, aPos.z, 1.0);
    gl_Position = uMVP * displaced;
}
"#;

pub const WATER_FRAGMENT_GLSL_GL33: &str = r#"#version 330 core
in vec3 vWorldPos;
in vec3 vNormal;
in vec2 vUV;

uniform vec3 uCameraPos;
uniform vec3 uLightDir;
uniform float uTime;
uniform float uShoreHeight;
uniform float uWaterPlane;
uniform mat4 uReflectVP;
uniform sampler2D uReflection;
uniform int uHasReflection;

out vec4 FragColor;

void main() {
    vec3 N = normalize(vNormal);
    vec3 V = normalize(uCameraPos - vWorldPos);
    vec3 L = normalize(-uLightDir);

    float fresnel = pow(1.0 - max(dot(N, V), 0.0), 3.5);
    vec3 deep = vec3(0.02, 0.18, 0.32);
    vec3 shallow = vec3(0.12, 0.55, 0.62);
    vec3 sky = vec3(0.45, 0.72, 0.95);

    float depth = clamp((vWorldPos.y - uShoreHeight + 0.4) * 2.0, 0.0, 1.0);
    vec3 water = mix(shallow, deep, depth);

    float spec = pow(max(dot(reflect(-L, N), V), 0.0), 128.0);
    float sun = pow(max(dot(N, L), 0.0), 4.0) * 0.15;

    vec3 col = water;

    if (uHasReflection == 1) {
        vec3 reflPos = vec3(vWorldPos.x, 2.0 * uWaterPlane - vWorldPos.y, vWorldPos.z);
        vec4 clip = uReflectVP * vec4(reflPos, 1.0);
        vec2 ruv = clip.xy / clip.w * 0.5 + 0.5;
        if (ruv.x > 0.02 && ruv.x < 0.98 && ruv.y > 0.02 && ruv.y < 0.98) {
            vec3 refl = texture(uReflection, ruv).rgb;
            col = mix(col, refl, fresnel * 0.82);
        } else {
            col = mix(col, sky, fresnel * 0.55);
        }
    } else {
        col = mix(col, sky, fresnel * 0.65);
    }

    col += vec3(1.0, 0.95, 0.85) * (spec * 0.8 + sun);

    float foam = sin(vUV.x * 40.0 + uTime * 2.0) * sin(vUV.y * 35.0 - uTime * 1.5);
    foam = smoothstep(0.55, 0.85, foam + fresnel * 0.3) * (1.0 - depth) * 0.35;
    col = mix(col, vec3(0.95, 0.98, 1.0), foam);

    float alpha = mix(0.72, 0.92, fresnel);
    FragColor = vec4(col, alpha);
}
"#;

// Vulkan / DirectX stubs (layout compatível)
pub const VERTEX_GLSL: &str = VERTEX_GLSL_GL33;
pub const FRAGMENT_GLSL: &str = FRAGMENT_GLSL_GL33;

#[cfg(all(feature = "directx", target_os = "windows"))]
pub const VERTEX_HLSL: &str = r#"
cbuffer Transform : register(b0) {
    float4x4 mvp; float4x4 model; float3 lightDir; float _pad;
};
struct VSIn { float3 pos : POSITION; float3 normal : NORMAL; float2 uv : TEXCOORD0; float3 color : COLOR; };
struct VSOut { float4 pos : SV_POSITION; float3 color : COLOR; float3 normal : NORMAL; float3 worldPos : TEXCOORD0; };
VSOut main(VSIn input) {
    VSOut o; float4 world = mul(model, float4(input.pos, 1.0));
    o.worldPos = world.xyz; o.normal = mul((float3x3)model, input.normal);
    o.color = input.color; o.pos = mul(mvp, float4(input.pos, 1.0)); return o;
}
"#;

#[cfg(all(feature = "directx", target_os = "windows"))]
pub const FRAGMENT_HLSL: &str = r#"
struct PSIn { float4 pos : SV_POSITION; float3 color : COLOR; float3 normal : NORMAL; float3 worldPos : TEXCOORD0; };
float4 main(PSIn input) : SV_Target {
    float3 norm = normalize(input.normal); float diff = max(dot(norm, float3(0.4, 0.8, 0.3)), 0.2);
    return float4(input.color * diff, 1.0);
}
"#;
