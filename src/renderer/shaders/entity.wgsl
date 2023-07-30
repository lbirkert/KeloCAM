struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(1) color: vec3<f32>,
};

struct Camera {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
    // width, height, zoom
    dimensions: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;


@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    out.pos = vec4<f32>(in.pos.xzy, 1.0) * camera.view_proj;
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(@location(1) color: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
