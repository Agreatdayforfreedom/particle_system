
@group(0) @binding(0) var sceneTexture: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;
@group(1) @binding(0) var blurredTexture: texture_2d<f32>;
@group(1) @binding(1) var bs: sampler;

@fragment
fn fs_main(@location(0) tex_coords : vec2<f32>) -> @location(0) vec4<f32> {
    let sceneColor = textureSample(sceneTexture, s, tex_coords).rgb;
    let bloom = textureSample(blurredTexture, bs, tex_coords).rgb;
    let finalColor = (sceneColor + bloom) * 5.0; // Adjust bloom intensity
    return vec4(finalColor, 1.0);
}