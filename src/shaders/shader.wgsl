

struct Camera {
    proj: mat4x4f,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct In {
    @location(0) vertex_position: vec2f,
    @location(1) position: vec3f,
}

struct Out {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: In) -> Out {
    var out: Out;

    out.clip_position = camera.proj * vec4<f32>(vec3f(in.vertex_position, 0.0) + in.position, 1.0);
    out.color = vec4f(0.0, 1.0, 1.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: Out) -> @location(0) vec4f {

    return in.color;
}