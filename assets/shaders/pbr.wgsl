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
    @location(2) uv: vec2<f32>,
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

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.world_position = world_position.xyz;
    out.world_normal = normalize((model.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let L = normalize(-light.direction.xyz);
    let V = normalize(camera.position.xyz - in.world_position);
    let H = normalize(L + V);
    let intensity = light.direction.w;

    let NdotL = max(dot(N, L), 0.0);
    let NdotH = max(dot(N, H), 0.0);

    let diffuse = NdotL * light.color.rgb * intensity;
    let specular = pow(NdotH, 64.0) * light.color.rgb * intensity * 0.5;
    let ambient = light.ambient.rgb;

    let base_color = vec3<f32>(0.7, 0.72, 0.78);
    let lit = base_color * (ambient + diffuse) + specular;

    return vec4<f32>(lit, 1.0);
}
