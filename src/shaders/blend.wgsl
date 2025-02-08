@group(0) @binding(0) var common_texture: texture_2d<f32>;
@group(1) @binding(0) var particles_texture: texture_2d<f32>;
@group(0) @binding(1) var common_sampler: sampler;
@group(1) @binding(1) var particles_sampler: sampler;


@fragment
fn fs_main(@location(0) tex_coords : vec2<f32>) -> @location(0) vec4f {
    let common_color = textureSample(common_texture, common_sampler, tex_coords);
    let particles_color = textureSample(particles_texture, particles_sampler, tex_coords);

    return common_color.rgba + particles_color.rgba;
}