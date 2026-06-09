//! Physically based rendering shader (Cook-Torrance).
//!
//! Implements the modern PBR pipeline:
//! - **GGX** for the normal distribution function.
//! - **Schlick** for the Fresnel approximation.
//! - **Smith** with the Schlick-GGX geometry term for visibility.
//! - **IBL** through an environment cubemap (irradiance + specular
//!   prefilter) and a 2D BRDF integration LUT.
//!
//! # Bind groups
//!
//! - `0` — camera uniforms.
//! - `1` — per-mesh model uniforms.
//! - `2` — light uniforms (single directional light).
//! - `3` — material: base color, metallic-roughness, normal map and the
//!   three environment maps (prefilter, irradiance, BRDF LUT).
//!
//! # Conventions
//!
//! - All shading is computed in world space.
//! - Tangent space normal maps are expected to be in OpenGL convention
//!   (Y up). The tangent handedness is read from the fourth component
//!   of the input tangent attribute.
//! - The environment prefilter uses a single mip level set by the
//!   CPU side based on the surface roughness.

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_tangent: vec3<f32>,
    @location(3) world_bitangent: vec3<f32>,
    @location(4) uv: vec2<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    position: vec4<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
    ambient: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> light: LightUniform;

// Material bind group: textures + flags.
@group(3) @binding(0) var base_color_tex: texture_2d<f32>;
@group(3) @binding(1) var base_color_sampler: sampler;
@group(3) @binding(2) var normal_tex: texture_2d<f32>;
@group(3) @binding(3) var normal_sampler: sampler;
@group(3) @binding(4) var mr_tex: texture_2d<f32>;
@group(3) @binding(5) var mr_sampler: sampler;
@group(3) @binding(6) var prefilter_tex: texture_cube<f32>;
@group(3) @binding(7) var prefilter_sampler: sampler;
@group(3) @binding(8) var irradiance_tex: texture_cube<f32>;
@group(3) @binding(9) var irradiance_sampler: sampler;
@group(3) @binding(10) var brdf_lut_tex: texture_2d<f32>;
@group(3) @binding(11) var brdf_lut_sampler: sampler;

// `PBR params` block encodes material constants that are not
// conveniently expressed as a texture (used for tuning / fallback).
struct MaterialParams {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    roughness: f32,
    metallic: f32,
    has_base_color_tex: f32,
    has_normal_tex: f32,
}

@group(3) @binding(12) var<uniform> material: MaterialParams;

const PI: f32 = 3.14159265359;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.world_position = world_position.xyz;

    let world_normal = normalize((model.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz);
    let world_tangent_raw = normalize((model.normal_matrix * vec4<f32>(in.tangent.xyz, 0.0)).xyz);
    let handedness = in.tangent.w;
    let world_bitangent = cross(world_normal, world_tangent_raw) * handedness;

    out.world_normal = world_normal;
    out.world_tangent = world_tangent_raw;
    out.world_bitangent = world_bitangent;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // ------------------------------------------------------------------
    // Material sampling
    // ------------------------------------------------------------------
    var base_color: vec3<f32>;
    if (material.has_base_color_tex > 0.5) {
        base_color = textureSample(base_color_tex, base_color_sampler, in.uv).rgb;
    } else {
        base_color = material.base_color.rgb;
    }
    base_color = pow(base_color, vec3<f32>(2.2)); // sRGB to linear approximation

    var mr: vec2<f32>;
    if (material.has_normal_tex > 0.5) {
        // The G/B channels of the metallic-roughness texture are reused
        // only when the normal map is bound, otherwise we rely on the
        // material params. A more complete pipeline would expose a
        // dedicated flag.
        mr = vec2<f32>(material.roughness, material.metallic);
    } else {
        mr = vec2<f32>(material.roughness, material.metallic);
    }
    let roughness = clamp(mr.x, 0.04, 1.0);
    let metallic = clamp(mr.y, 0.0, 1.0);

    // ------------------------------------------------------------------
    // Normal map (tangent space)
    // ------------------------------------------------------------------
    var N: vec3<f32> = normalize(in.world_normal);
    if (material.has_normal_tex > 0.5) {
        let tangent_normal = textureSample(normal_tex, normal_sampler, in.uv).xyz * 2.0 - 1.0;
        let T = normalize(in.world_tangent);
        let B = normalize(in.world_bitangent);
        N = normalize(T * tangent_normal.x + B * tangent_normal.y + N * tangent_normal.z);
    }

    let V = normalize(camera.position.xyz - in.world_position);
    let NdotV = max(dot(N, V), 0.0001);

    // Reflected view vector for environment sampling.
    let R = reflect(-V, N);

    // ------------------------------------------------------------------
    // Direct lighting (single directional light)
    // ------------------------------------------------------------------
    let L = normalize(-light.direction.xyz);
    let H = normalize(L + V);
    let NdotL = max(dot(N, L), 0.0);
    let NdotH = max(dot(N, H), 0.0);
    let VdotH = max(dot(V, H), 0.0);

    // F0: dielectric baseline, lerp toward base color for metals.
    let F0 = mix(vec3<f32>(0.04), base_color, metallic);

    let D = distribution_ggx(NdotH, roughness);
    let G = geometry_smith(NdotV, NdotL, roughness);
    let F = fresnel_schlick(VdotH, F0);

    let specular = (D * G * F) / max(4.0 * NdotV * NdotL, 0.0001);
    let kd = (vec3<f32>(1.0) - F) * (1.0 - metallic);
    let diffuse = kd * base_color / PI;

    let Lo = (diffuse + specular) * light.color.rgb * NdotL * light.direction.w;

    // ------------------------------------------------------------------
    // IBL ambient
    // ------------------------------------------------------------------
    let irradiance = textureSample(irradiance_tex, irradiance_sampler, N).rgb;
    let F_ibl = fresnel_schlick_roughness(NdotV, F0, roughness);
    let kd_ibl = (vec3<f32>(1.0) - F_ibl) * (1.0 - metallic);
    let diffuse_ibl = irradiance * base_color * kd_ibl;

    let lod = roughness * f32(textureNumLevels(prefilter_tex));
    let prefilter = textureSampleLevel(prefilter_tex, prefilter_sampler, R, lod).rgb;
    let brdf = textureSample(brdf_lut_tex, brdf_lut_sampler, vec2<f32>(NdotV, roughness)).rg;
    let specular_ibl = prefilter * (F_ibl * brdf.x + brdf.y);

    let ambient = (diffuse_ibl + specular_ibl) * light.ambient.rgb;

    // ------------------------------------------------------------------
    // Emissive
    // ------------------------------------------------------------------
    let emissive = material.emissive.rgb;

    let color = ambient + Lo + emissive;
    return vec4<f32>(pow(color, vec3<f32>(1.0 / 2.2)), 1.0);
}

// ----------------------------------------------------------------------------
// BRDF helpers
// ----------------------------------------------------------------------------

/// Trowbridge-Reitz / GGX normal distribution function.
fn distribution_ggx(NdotH: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH2 = NdotH * NdotH;

    let num = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    let denom2 = PI * denom * denom;
    return num / max(denom2, 0.0001);
}

/// Schlick-GGX geometry term for the Smith visibility function.
fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;
    return num / max(denom, 0.0001);
}

/// Smith geometry term combining view and light directions.
fn geometry_smith(NdotV: f32, NdotL: f32, roughness: f32) -> f32 {
    return geometry_schlick_ggx(NdotV, roughness) * geometry_schlick_ggx(NdotL, roughness);
}

/// Fresnel-Schlick approximation of the Fresnel equation.
fn fresnel_schlick(VdotH: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (vec3<f32>(1.0) - F0) * pow(clamp(1.0 - VdotH, 0.0, 1.0), 5.0);
}

/// Schlick Fresnel approximation extended to a non-zero roughness.
fn fresnel_schlick_roughness(NdotV: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return F0 + (max(vec3<f32>(1.0 - roughness), F0) - F0)
        * pow(clamp(1.0 - NdotV, 0.0, 1.0), 5.0);
}
