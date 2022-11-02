struct VertOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) vert_position: vec3<f32>,
    // @location(1) vert_normal: vec3<f32>,
    @location(2) vert_color: vec3<f32>,
) -> VertOut {
    var out: VertOut;

    out.pos = vec4(vert_position, 1.0);
    out.color = vert_color;

    return out;
}

@fragment
fn fs_main(
    in: VertOut
) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}