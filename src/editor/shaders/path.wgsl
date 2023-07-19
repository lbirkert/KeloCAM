struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexIn {
    @location(0) before: vec3<f32>,
    @location(1) pos: vec3<f32>,
    @location(2) after: vec3<f32>,

    @location(3) color: vec3<f32>,
    @location(4) thickness: f32,
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
fn vs_main(@builtin(vertex_index) v_idx: u32, in: VertexIn) -> VertexOut {
    let vi = f32(v_idx) % 4.0;

    var out: VertexOut;

    var oa = vec4<f32>(in.before.xzy, 1.0) * camera.view_proj;
    oa /= abs(oa.w);
    let pa = oa.xy * camera.dimensions.xy;
    var ob = vec4<f32>(in.pos.xzy, 1.0) * camera.view_proj;
    ob /= abs(ob.w);
    let pb = ob.xy * camera.dimensions.xy;
    var oc = vec4<f32>(in.after.xzy, 1.0) * camera.view_proj;
    oc /= abs(oc.w);
    let pc = oc.xy * camera.dimensions.xy;

    let n1 = normalize(vec2<f32>(pa.y - pb.y, pb.x - pa.x));
    let n2 = normalize(vec2<f32>(pb.y - pc.y, pc.x - pb.x));

    let n = normalize(n1 + n2);

    let len = 1.0 / dot(n, n1);
    let max_len = 1.1;

    var offset: vec2<f32>;
    if len > max_len {
        let d = normalize(pb - pa) + normalize(pb - pc);
        if (vi < 2.0) == (vi % 2.0 == 0.0) {
            let d = normalize(pb - pa);
            offset = n1 + (max_len - dot(n, n1)) * d / dot(d, n);
        } else {
            let d = normalize(pb - pc);
            offset = n2 + (max_len - dot(n, n2)) * d / dot(d, n);
        }
    } else {
        offset = n * len;
    }

    // TODO: antialias
    out.color = in.color;

    // TODO: depth test
    out.pos = vec4<f32>((pb + offset * in.thickness / 2.0) / camera.dimensions.xy, 0.0, 1.0);

    return out;
}


@fragment
fn fs_main(@location(1) color: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
