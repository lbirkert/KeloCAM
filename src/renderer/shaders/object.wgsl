struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) world_pos: vec3<f32>,
};

struct Camera {
    proj: mat4x4<f32>,
    // x, y, z, zoom
    pos: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    out.color = in.color;
    out.color = in.normal * 0.5 + vec3(0.5);
    out.normal = in.normal.xzy;
    out.world_pos = in.pos.xzy;

    out.pos = vec4<f32>(out.world_pos, 1.0) * camera.proj;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // Lighting
    let view_normal = normalize(camera.pos.xyz - in.world_pos);
    let light = dot(in.normal, view_normal) * 0.4 + 0.6;

    return vec4<f32>(in.color * light, 1.0);
}
