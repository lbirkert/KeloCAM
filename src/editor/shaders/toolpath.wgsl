struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(1) vpos: vec2<f32>,
};

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct Camera {
    view_proj: mat4x4<f32>,
    // x, y, z, zoom
    view_pos: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VertexOut {
}

@fragment
fn fs_main(@location(1) vpos: vec2<f32>) -> @location(0) vec4<f32> {
    return vec4(1.0);
}
