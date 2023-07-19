struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(1) vpos: vec2<f32>,
    @location(2) light: f32,
};

struct Camera {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
    // width, height, zoom
    dimensions: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    let grid_size = vec2<f32>(10.0, 20.0);

    var out: VertexOut;

    let grid_pos = v_positions[v_idx] * grid_size;
    let world_pos = vec4<f32>(grid_pos.x, 0.0, grid_pos.y, 1.0);

    out.pos = world_pos * camera.view_proj;
    out.vpos = grid_pos;

    let normal = vec3<f32>(0.0, 1.0, 0.0);
    let view_normal = normalize(camera.view_pos.xyz - world_pos.xyz);

    out.light = dot(normal, view_normal);

    return out;
}

fn logn(a: f32, x: f32) -> f32 {
    return log(x) / log(a);
}

@fragment
fn fs_main(@location(1) vpos: vec2<f32>, @location(2) light: f32) -> @location(0) vec4<f32> {
    var zoom = camera.dimensions.z;
    if light < 0.0 {
        zoom = 0.0;
    }

    var color = grid(vpos) * 0.6;

    for (var i: f32 = 1.0; i < floor(logn(10.0, zoom)) + 4.0; i += 1.0) {
        let fact = min(1.0, logn(10.0, zoom) - i + 3.0);
        if fact > 0.0 {
            color = max(color, grid(vpos * pow(10.0, i)) * pow(0.3, i) * fact);
        }
    }

    if color.w == 0.0 {
        discard;
    }

    return color;
}

fn grid(vpos: vec2<f32>) -> vec4<f32> {
    // https://madebyevan.com/shaders/grid/

    let grid = abs(fract(vpos - 0.5) - 0.5) / (abs(dpdx(vpos)) + abs(dpdy(vpos)));
    let line = min(grid.x, grid.y);

    var color = 1.0 - min(line, 1.0);

    // Gama correction
    color = pow(color, 1.0 / 2.2);

    if color == 0.0 {
        return vec4(0.0);
    } else {
        return vec4(color);
    }
}
