struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(1) color: vec3<f32>,
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

    let world_pos = vec4<f32>(in.pos.xzy, 1.0);
    out.pos = world_pos * camera.proj;

    // Lighting
    let normal = in.normal.xzy;
    let view_normal = normalize(camera.pos.xyz - world_pos.xyz);

    let light = dot(normal, view_normal) * 0.4 + 0.6;
    out.color = in.color * light;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
