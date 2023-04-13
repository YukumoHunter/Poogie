struct VertOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct MeshPushConstants {
    data: vec4<f32>,
    render_matrix: mat4x4<f32>,
}

var<push_constant> pc: MeshPushConstants;

@vertex
fn vs_main(
    @location(0) vert_position: vec3<f32>,
    // @location(1) vert_normal: vec3<f32>,
    @location(2) vert_color: vec3<f32>,
) -> VertOut {
    var out: VertOut;

    out.pos = pc.render_matrix * vec4(vert_position, 1.0);
    out.color = vert_color;

    return out;
}

@fragment
fn fs_main(
    in: VertOut
) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}