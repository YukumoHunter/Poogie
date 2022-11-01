struct VertOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

var<private> positions: array<vec3<f32>, 3> = array<vec3<f32>, 3>(
    vec3<f32>(0.6, -0.6, 0.0),
    vec3<f32>(-0.6, -0.6, 0.0),
    vec3<f32>(0.0, 0.6, 0.0),
);

var<private> colors: array<vec3<f32>, 3> = array<vec3<f32>, 3>(
    vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(0.0, 0.0, 1.0),
);


@vertex
fn vs_main(
    @builtin(vertex_index) vert_idx: u32,
) -> VertOut {
    var out: VertOut;

    out.pos = vec4(positions[vert_idx], 1.0);
    out.color = colors[vert_idx];

    return out;
}

@fragment
fn fs_main(
    in: VertOut
) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}