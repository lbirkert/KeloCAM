struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(1) vpos: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-10.0, -10.0),
    vec2<f32>(-10.0, 10.0),
    vec2<f32>(10.0, 10.0),
    vec2<f32>(-10.0, -10.0),
    vec2<f32>(10.0, 10.0),
    vec2<f32>(10.0, -10.0),
);

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;

    out.pos = vec4<f32>(v_positions[v_idx], 0.0, 1.0) * view_proj;
    out.vpos = v_positions[v_idx];

    return out;
}

@fragment
fn fs_main(@location(1) vpos: vec2<f32>) -> @location(0) vec4<f32> {
    let xfac = 4.0 * abs(vpos.x - round(vpos.x)) + abs(dpdx(vpos.x)) + abs(dpdy(vpos.x));
    let yfac = 4.0 * abs(vpos.y - round(vpos.y)) + abs(dpdx(vpos.y)) + abs(dpdy(vpos.y));

    return vec4(max(xfac - 1.95, 0.0) * 50.0 + max(yfac - 1.95, 0.0) * 50.0);
}
