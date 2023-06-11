struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
};

struct Camera {
    view_proj: mat4x4<f32>,
    // x, y, z, zoom
    view_pos: vec4<f32>,
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
    out.pos = world_pos * camera.view_proj;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return vec4(1.0);
}
