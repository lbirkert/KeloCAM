struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(1) light: f32,
};

struct Camera {
    proj: mat4x4<f32>,
    // x, y, z, zoom
    pos: vec4<f32>,
};

struct Object {
    proj: mat4x4<f32>
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> object: Object;

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    let world_pos = object.proj * vec4<f32>(in.pos, 1.0);
    out.pos = world_pos * camera.proj;

    // Lighting
    let view_normal = normalize(camera.pos.xyz - world_pos.xyz);

    out.light = dot(in.normal, view_normal);

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4(vec3(0.5 + in.light * 0.2), 1.0);
}
