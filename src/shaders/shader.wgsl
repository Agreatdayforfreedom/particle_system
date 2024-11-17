

struct In {
    @location(0) position: vec2f,
    @location(1) tex_coords: vec2f,
}

struct Out {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: In) -> Out {
    var out: Out;

    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    out.color = vec4f(0.0, 1.0, 1.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: Out) -> @location(0) vec4f {

    return in.color;
}