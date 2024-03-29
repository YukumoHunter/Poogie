struct VertOut {
    @builtin(position) pos: vec4<f32>,
};

var<private> positions: array<vec3<f32>, 3> = array<vec3<f32>, 3>(
    vec3<f32>(1.0, -1.0, 0.0),
    vec3<f32>(-1.0, -1.0, 0.0),
    vec3<f32>(0.0, 1.0, 0.0),
);

@vertex
fn main(
    @builtin(vertex_index) vert_idx: u32,
) -> VertOut {
    var out: VertOut;

    out.pos = vec4(positions[vert_idx], 1.0);

    return out;
}