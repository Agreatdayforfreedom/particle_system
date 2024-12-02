
struct Camera {
    proj: mat4x4f,
    view: mat4x4f,
    position: vec3f,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct In {
    @location(0) vertex_position: vec2f,
    @location(1) position: vec4f,
}

struct Out {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: In) -> Out {
    var out: Out;
    var proj_view = camera.proj * camera.view;
    
    let view = camera.view;
    let right = vec3<f32>(view[0][0], view[1][0], view[2][0]); // right
    let up = vec3<f32>(view[0][1], view[1][1], view[2][1]); // up

    let worldPosition = in.position.xyz
                        + right * (in.vertex_position.x * 0.1)
                        + up * (in.vertex_position.y * 0.1);
    

    out.clip_position = proj_view * vec4<f32>(worldPosition, 1.0);
    out.color = mix(vec4(0.0, 0.2, 0.8, 1.0),
        vec4(0.17, 0.1, 0.2, 1.0),
        in.position.w 
        );
    return out;
}

@fragment
fn fs_main(in: Out) -> @location(0) vec4f {

    return in.color;
}