
@group(0) @binding(0) var sceneTexture: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;

@fragment
fn fs_main( @location(0) tex_coords : vec2<f32>) -> @location(0) vec4<f32> {
    let color: vec3<f32> = textureSample(sceneTexture, s, tex_coords).rgb;
    let threshold = 0.7; // Adjust as needed
    let brightColor = max(color - vec3(threshold), vec3(0.0)); // Only keep bright parts
    return vec4(brightColor, 1.0);
}