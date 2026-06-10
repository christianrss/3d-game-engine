//! Shaders GLSL 330 — PBR, sombras, pós-processo.

pub const LIGHT_DIRECTION: [f32; 3] = [-0.42, -0.78, -0.38];

/// Vertex principal — PBR + shadow map + velocity (TAA)
pub const VERTEX_GLSL_GL33: &str = r#"#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aUV;
layout(location = 3) in vec3 aColor;

uniform mat4 uMVP;
uniform mat4 uModel;
uniform mat4 uLightSpaceMatrix;
uniform mat4 uPrevViewProj;

out vec3 vColor;
out vec3 vNormal;
out vec3 vWorldPos;
out vec2 vUV;
out vec4 vFragPosLightSpace;
out vec2 vVelocity;

void main() {
    vec4 world = uModel * vec4(aPos, 1.0);
    vWorldPos = world.xyz;
    vNormal = mat3(transpose(inverse(uModel))) * aNormal;
    vColor = aColor;
    vUV = aUV;
    vFragPosLightSpace = uLightSpaceMatrix * world;
    gl_Position = uMVP * vec4(aPos, 1.0);

    vec4 prevClip = uPrevViewProj * world;
    vec2 currNdc = gl_Position.xy / max(gl_Position.w, 0.0001);
    vec2 prevNdc = prevClip.xy / max(prevClip.w, 0.0001);
    vVelocity = (currNdc - prevNdc) * 0.5;
}
"#;

/// Fragment PBR Cook-Torrance GGX + sombras PCF + névoa
pub const FRAGMENT_GLSL_GL33: &str = r#"#version 330 core

in vec3 vColor;
in vec3 vNormal;
in vec3 vWorldPos;
in vec2 vUV;
in vec4 vFragPosLightSpace;

layout(location = 0) out vec4 oColor;
layout(location = 1) out vec2 oVelocity;

uniform vec3 uLightDir;
uniform vec3 uCameraPos;
uniform vec3 uFogColor;
uniform vec3 uHorizon;
uniform vec3 uZenith;
uniform float uFogDensity;
uniform float uRoughness;
uniform float uMetallic;
uniform float uNightFactor;
uniform int uUseAlbedo;
uniform int uUseNormalMap;
uniform int uUseRoughMap;
uniform int uUseAOMap;
uniform int uHasEnvMap;
uniform int uUseDetailNormal;
uniform float uTiling;
uniform float uTime;
uniform int uMatType; // 0=color 1=terrain sand 2=rock triplanar

uniform sampler2D uAlbedo;
uniform sampler2D uNormalMap;
uniform sampler2D uRoughMap;
uniform sampler2D uAOMap;
uniform sampler2D uDetailNormal;
uniform sampler2D uShadowMap;
uniform samplerCube uEnvMap;

in vec2 vVelocity;

const float PI = 3.14159265;

// Ruído procedural para micro-detalhe em objetos sem textura
float hash31(vec3 p) {
    p = fract(p * 0.3183099 + vec3(0.1, 0.2, 0.3));
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

float vnoise(vec3 p) {
    vec3 i = floor(p);
    vec3 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float n000 = hash31(i);
    float n100 = hash31(i + vec3(1, 0, 0));
    float n010 = hash31(i + vec3(0, 1, 0));
    float n110 = hash31(i + vec3(1, 1, 0));
    float n001 = hash31(i + vec3(0, 0, 1));
    float n101 = hash31(i + vec3(1, 0, 1));
    float n011 = hash31(i + vec3(0, 1, 1));
    float n111 = hash31(i + vec3(1, 1, 1));
    float nx00 = mix(n000, n100, f.x);
    float nx10 = mix(n010, n110, f.x);
    float nx01 = mix(n001, n101, f.x);
    float nx11 = mix(n011, n111, f.x);
    float nxy0 = mix(nx00, nx10, f.y);
    float nxy1 = mix(nx01, nx11, f.y);
    return mix(nxy0, nxy1, f.z);
}

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

float shadowPCF(vec4 fragPosLight, vec3 N, vec3 L) {
    vec3 proj = fragPosLight.xyz / fragPosLight.w;
    proj = proj * 0.5 + 0.5;
    if (proj.z > 1.0 || proj.x < 0.0 || proj.x > 1.0 || proj.y < 0.0 || proj.y > 1.0)
        return 0.0;

    float bias = max(0.0018 * (1.0 - dot(N, L)), 0.0006);
    vec2 texel = 1.0 / vec2(textureSize(uShadowMap, 0));
    float shadow = 0.0;
    // Kernel Poisson 12 taps — sombras mais suaves
    const vec2 k[12] = vec2[](
        vec2(-0.94, -0.32), vec2(-0.48, 0.88), vec2(0.38, -0.78),
        vec2(0.72, 0.42), vec2(-0.12, -0.58), vec2(0.18, 0.22),
        vec2(-0.62, 0.14), vec2(0.52, -0.18), vec2(-0.28, -0.92),
        vec2(0.86, 0.68), vec2(-0.78, 0.52), vec2(0.08, 0.94)
    );
    for (int i = 0; i < 12; i++) {
        vec2 off = k[i] * texel * 2.2;
        float depth = texture(uShadowMap, proj.xy + off).r;
        shadow += proj.z - bias > depth ? 1.0 : 0.0;
    }
    return shadow / 12.0;
}

/// Normal procedural — suaviza aparência de cubos/low-poly
vec3 proceduralDetailNormal(vec3 wp, vec3 baseN) {
    float scale = 4.5;
    vec3 p = wp * scale;
    float eps = 0.04;
    float h = vnoise(p);
    float hx = vnoise(p + vec3(eps, 0.0, 0.0)) - h;
    float hy = vnoise(p + vec3(0.0, eps, 0.0)) - h;
    float hz = vnoise(p + vec3(0.0, 0.0, eps)) - h;
    vec3 bump = vec3(-hx, -hy, -hz) / eps;
    float macro = sin(wp.x * 1.8 + wp.z * 1.3) * 0.06;
    bump += vec3(macro, sin(wp.y * 2.1) * 0.04, macro * 0.7);
    return normalize(baseN + bump * 0.55);
}

/// TBN para normal maps em malhas sem tangentes
vec3 applyNormalMap(vec3 N, vec3 ntex) {
    vec3 up = abs(N.y) < 0.999 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
    vec3 T = normalize(cross(up, N));
    vec3 B = cross(N, T);
    return normalize(mat3(T, B, N) * ntex);
}

/// Triplanar com swizzle correto dos eixos
vec3 triplanarNormal(vec3 wp, vec3 N, float scale) {
    vec3 w = abs(N);
    w = max(w, 0.0001);
    w /= (w.x + w.y + w.z);

    vec3 nx = texture(uNormalMap, wp.yz * scale).xyz * 2.0 - 1.0;
    vec3 ny = texture(uNormalMap, wp.xz * scale).xyz * 2.0 - 1.0;
    vec3 nz = texture(uNormalMap, wp.xy * scale).xyz * 2.0 - 1.0;
    nx = vec3(nx.z, nx.y, nx.x);
    ny = vec3(ny.x, ny.z, ny.y);

    return normalize(nx * w.x + ny * w.y + nz * w.z);
}

/// Ambiente quente — NÃO multiplica albedo por cor de céu cinza
vec3 hemisphereAmbient(vec3 N, vec3 albedo, float ao) {
    float up = clamp(N.y * 0.5 + 0.5, 0.0, 1.0);
    float night = mix(1.0, 0.55, uNightFactor);
    vec3 warmBounce = vec3(0.42, 0.36, 0.28) * (1.0 - up);
    vec3 skyBounce = mix(uHorizon, uZenith, up) * 0.06;
    return (albedo * 0.28 + warmBounce + skyBounce) * ao * night;
}

vec3 sampleIBL(vec3 N, vec3 V, vec3 albedo, float rough, float ao, vec3 kD) {
    if (uHasEnvMap != 1) return vec3(0.0);
    vec3 R = reflect(-V, N);
    float lod = clamp(rough * 4.0, 0.0, 3.0);
    vec3 specIBL = textureLod(uEnvMap, R, lod).rgb;
    float fresnel = pow(1.0 - max(dot(N, V), 0.0), 3.0);
    return specIBL * fresnel * (1.0 - rough) * 0.06 * ao;
}

vec3 boostSaturation(vec3 c, float amount) {
    float l = dot(c, vec3(0.299, 0.587, 0.114));
    return mix(vec3(l), c, amount);
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

/// Areia animada — ondas Gerstner + grãos ao vento (shader)
vec3 sandRippleNormal(vec3 wp, vec3 baseN) {
    float t = uTime;
    float wx = wp.x * 8.0 + t * 0.55;
    float wz = wp.z * 7.0 - t * 0.42;
    float ripple = sin(wx) * cos(wz) * 0.055;
    float fine = sin(wp.x * 42.0 - wp.z * 36.0 + t * 1.4) * 0.022;
    float dune = sin(wp.x * 0.11 + wp.z * 0.08 + t * 0.08) * 0.07;
    float wind = sin(wp.x * 0.35 + wp.z * 0.22 - t * 0.65) * cos(wp.z * 0.18 + t * 0.4) * 0.035;
    vec3 bump = vec3(ripple + dune + wind, fine * 0.5, fine + wind * 0.6);
    return normalize(baseN + bump);
}

vec3 sandAnimatedAlbedo(vec3 albedo, vec3 wp) {
    float t = uTime;
    float shimmer = sin(wp.x * 24.0 + wp.z * 19.0 + t * 2.0) * 0.5 + 0.5;
    float streak = smoothstep(0.35, 0.9, sin(wp.x * 0.5 + wp.z * 0.3 - t * 0.9) * 0.5 + 0.5);
    vec3 warm = vec3(1.06, 0.98, 0.82);
    return albedo * mix(vec3(1.0), warm, shimmer * 0.05 + streak * 0.03);
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
        N = triplanarNormal(wp, N, scale);
    } else {
        albedo = uUseAlbedo == 1 ? pow(texture(uAlbedo, uv).rgb, vec3(2.2)) : pow(vColor, vec3(2.2));
        rough = uUseRoughMap == 1 ? texture(uRoughMap, uv).r : uRoughness;
        ao = uUseAOMap == 1 ? texture(uAOMap, uv).r : 1.0;
        if (uUseNormalMap == 1) {
            vec3 ntex = texture(uNormalMap, uv).rgb * 2.0 - 1.0;
            N = applyNormalMap(N, ntex);
        }
        if (uMatType == 1) {
            albedo = sandAnimatedAlbedo(albedo, wp);
            N = sandRippleNormal(wp, N);
            rough = clamp(rough + sin(wp.x * 22.0 + uTime * 0.3) * 0.03, 0.42, 0.95);
        } else if (uMatType == 0) {
            if (uUseDetailNormal == 1) {
                vec3 ntex = texture(uDetailNormal, wp.xz * 2.5).rgb * 2.0 - 1.0;
                N = applyNormalMap(N, ntex);
            } else {
                N = proceduralDetailNormal(wp, N);
            }
            float grain = vnoise(wp * 8.0) * 0.04;
            albedo *= 1.0 + grain;
            rough = clamp(rough + vnoise(wp * 3.5) * 0.08, 0.08, 0.95);
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

    // Wrap diffuse — transição suave luz/sombra
    float wrap = 0.22;
    float NdotL = clamp((dot(N, L) + wrap) / (1.0 + wrap), 0.0, 1.0);
    float shadow = shadowPCF(vFragPosLightSpace, N, L);

    vec3 sunColor = mix(vec3(1.18, 1.08, 0.88), vec3(0.75, 0.82, 1.05), uNightFactor);
    vec3 ambient = hemisphereAmbient(N, albedo, ao);
    vec3 ibl = sampleIBL(N, V, albedo, rough, ao, kD);

    vec3 skyFill = mix(vec3(0.55, 0.5, 0.42), vec3(0.35, 0.42, 0.55), uNightFactor);
    float fill = max(dot(N, normalize(vec3(-L.x * 0.3, 0.7, -L.z * 0.3))), 0.0);
    fill *= 0.22 * (1.0 - shadow * 0.35);

    vec3 radiance = sunColor * (1.0 - shadow * 0.82);
    vec3 diffuse = kD * albedo * radiance * NdotL;
    vec3 specular = spec * radiance * NdotL;
    vec3 color = ambient + ibl + diffuse + specular + skyFill * fill * albedo;

    float rim = pow(1.0 - max(dot(N, V), 0.0), 3.0) * 0.15;
    color += sunColor * rim * (1.0 - shadow * 0.4);

    float dist = length(vWorldPos - uCameraPos);
    float heightFade = exp(-max(vWorldPos.y - 4.0, 0.0) * 0.006);
    float fog = (1.0 - exp(-uFogDensity * dist)) * heightFade;
    vec3 fogTint = mix(uFogColor, uHorizon, clamp(dist * 0.00035, 0.0, 0.25));
    color = mix(color, fogTint, clamp(fog, 0.0, 0.42));

    color = boostSaturation(color, 1.12);
    color = color / (color + vec3(1.05));
    oColor = vec4(max(color, vec3(0.0)), 1.0);
    oVelocity = vVelocity;
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

/// Pós-processo: TAA (neighborhood clamp) + bloom + FXAA
pub const POST_FRAGMENT_GLSL_GL33: &str = r#"#version 330 core
in vec2 vUV;
out vec4 FragColor;
uniform sampler2D uScene;
uniform sampler2D uVelocity;
uniform sampler2D uHistory;
uniform vec2 uTexelSize;
uniform float uTaaBlend;

vec3 neighborhoodMin(sampler2D tex, vec2 uv, vec2 texel) {
    vec3 m = texture(tex, uv).rgb;
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            m = min(m, texture(tex, uv + vec2(x, y) * texel).rgb);
        }
    }
    return m;
}

vec3 neighborhoodMax(sampler2D tex, vec2 uv, vec2 texel) {
    vec3 m = texture(tex, uv).rgb;
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            m = max(m, texture(tex, uv + vec2(x, y) * texel).rgb);
        }
    }
    return m;
}

vec3 clipAabb(vec3 history, vec3 minimum, vec3 maximum) {
    vec3 center = 0.5 * (maximum + minimum);
    vec3 extents = 0.5 * (maximum - minimum) + vec3(0.001);
    vec3 offset = history - center;
    return center + clamp(offset, -extents, extents);
}

vec3 taa(sampler2D scene, sampler2D vel, sampler2D hist, vec2 uv, vec2 texel) {
    vec3 curr = texture(scene, uv).rgb;
    vec2 velocity = texture(vel, uv).rg;
    vec2 histUv = uv - velocity;
    if (histUv.x < 0.0 || histUv.x > 1.0 || histUv.y < 0.0 || histUv.y > 1.0) {
        return curr;
    }
    vec3 prev = texture(hist, histUv).rgb;
    vec3 nmin = neighborhoodMin(scene, uv, texel);
    vec3 nmax = neighborhoodMax(scene, uv, texel);
    prev = clipAabb(prev, nmin, nmax);
    float motion = clamp(length(velocity) * 90.0, 0.0, 1.0);
    float blend = mix(uTaaBlend, 0.06, motion);
    return mix(curr, prev, blend);
}

vec3 fxaa(sampler2D tex, vec2 uv) {
    vec3 rgbNW = texture(tex, uv + vec2(-1.0, -1.0) * uTexelSize).rgb;
    vec3 rgbNE = texture(tex, uv + vec2(1.0, -1.0) * uTexelSize).rgb;
    vec3 rgbSW = texture(tex, uv + vec2(-1.0, 1.0) * uTexelSize).rgb;
    vec3 rgbSE = texture(tex, uv + vec2(1.0, 1.0) * uTexelSize).rgb;
    vec3 rgbM  = texture(tex, uv).rgb;

    vec3 luma = vec3(0.299, 0.587, 0.114);
    float lNW = dot(rgbNW, luma);
    float lNE = dot(rgbNE, luma);
    float lSW = dot(rgbSW, luma);
    float lSE = dot(rgbSE, luma);
    float lM  = dot(rgbM, luma);

    float lMin = min(lM, min(min(lNW, lNE), min(lSW, lSE)));
    float lMax = max(lM, max(max(lNW, lNE), max(lSW, lSE)));
    float contrast = lMax - lMin;

    if (contrast < max(0.04, lMax * 0.125)) {
        return rgbM;
    }

    vec3 blur = (rgbNW + rgbNE + rgbSW + rgbSE) * 0.25;
    return mix(rgbM, blur, 0.5);
}

void main() {
    vec3 col = taa(uScene, uVelocity, uHistory, vUV, uTexelSize);

    float lum = dot(col, vec3(0.2126, 0.7152, 0.0722));
    vec3 bloom = vec3(0.0);
    if (lum > 0.92) {
        for (int x = -1; x <= 1; x++) {
            for (int y = -1; y <= 1; y++) {
                vec2 off = vec2(x, y) * uTexelSize * 2.0;
                vec3 s = texture(uScene, vUV + off).rgb;
                float l = dot(s, vec3(0.2126, 0.7152, 0.0722));
                if (l > 0.92) bloom += s;
            }
        }
        bloom /= 9.0;
    }
    col += bloom * 0.06;

    vec3 fx = fxaa(uScene, vUV);
    col = mix(col, fx, 0.18);

    col = pow(max(col, vec3(0.0)), vec3(1.0 / 2.2));
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

/// HUD UI — painéis com cantos arredondados, gradiente e glow.
pub const HUD_UI_VERTEX_GLSL: &str = r#"#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aUV;
out vec2 vUV;
void main() {
    vUV = aUV;
    gl_Position = vec4(aPos, 0.0, 1.0);
}
"#;

pub const HUD_UI_FRAGMENT_GLSL: &str = r#"#version 330 core
in vec2 vUV;
uniform vec4 uColor;
uniform vec4 uAccent;
uniform float uGlow;
uniform float uTime;
out vec4 FragColor;

float sdRoundBox(vec2 p, vec2 halfSize, float r) {
    vec2 q = abs(p) - halfSize + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void main() {
    vec2 p = vUV * 2.0 - 1.0;
    float d = sdRoundBox(p, vec2(0.88, 0.82), 0.22);
    float alpha = 1.0 - smoothstep(-0.01, 0.03, d);
    float edge = exp(-abs(d) * 14.0) * uGlow;
    float scan = sin(vUV.y * 24.0 - uTime * 3.0) * 0.5 + 0.5;
    vec3 base = mix(uColor.rgb, uAccent.rgb, vUV.y * 0.55 + scan * 0.08);
    vec3 col = base + uAccent.rgb * edge * 0.65;
    FragColor = vec4(col, uColor.a * alpha + edge * 0.25);
}
"#;

/// HUD texto — atlas bitmap com outline e glow.
pub const HUD_TEXT_VERTEX_GLSL: &str = r#"#version 330 core
layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aUV;
out vec2 vUV;
void main() {
    vUV = aUV;
    gl_Position = vec4(aPos, 0.0, 1.0);
}
"#;

pub const HUD_TEXT_FRAGMENT_GLSL: &str = r#"#version 330 core
in vec2 vUV;
uniform sampler2D uFont;
uniform vec4 uColor;
uniform float uGlow;
out vec4 FragColor;

void main() {
    float a = texture(uFont, vUV).a;
    float outline = 0.0;
    vec2 px = vec2(1.0 / 128.0, 1.0 / 72.0);
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            if (x == 0 && y == 0) continue;
            outline = max(outline, texture(uFont, vUV + vec2(x, y) * px).a);
        }
    }
    float glow = exp(-(1.0 - a) * 8.0) * uGlow;
    vec3 col = uColor.rgb * a + uColor.rgb * glow * 0.35;
    col += vec3(0.0) * outline * (1.0 - a) * 0.6;
    float alpha = clamp(a + outline * 0.4 + glow * 0.2, 0.0, 1.0) * uColor.a;
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
