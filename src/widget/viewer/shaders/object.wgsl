struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;

    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0) * view_proj;
    out.color = vec4(f32(v_idx) / 6.0);

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
