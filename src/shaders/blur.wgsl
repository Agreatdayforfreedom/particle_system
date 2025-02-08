var<private> weights: array<f32, 5> = array(0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);  

@group(0) @binding(0) var brightTexture: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;

@fragment
fn vertical_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    let offset = vec2(0.0, 1.0 / f32(textureDimensions(brightTexture).y)); // Vertical

    var color = textureSample(brightTexture, s, tex_coords).rgb * weights[0];
    for(var i = 0; i < 5; i++) {
        color += textureSample(brightTexture, s, tex_coords + vec2f(0.0,offset.y * f32(i))).rgb * weights[i];
        color += textureSample(brightTexture, s, tex_coords - vec2f(0.0,offset.y * f32(i))).rgb * weights[i];   
     
    }
    return vec4f(color, 1.0);
}


@fragment
fn horizontal_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    let offset = vec2(1.0 / f32(textureDimensions(brightTexture).x), 0.0); // Horizontal
    var color = textureSample(brightTexture, s, tex_coords).rgb * weights[0];
    for(var i = 0; i < 5; i++) {
        color += textureSample(brightTexture, s, tex_coords + vec2f(offset.x * f32(i), 0.0)).rgb * weights[i];
        color += textureSample(brightTexture, s, tex_coords - vec2f(offset.x * f32(i), 0.0)).rgb * weights[i];   
  
    }
    
    return vec4f(color, 1.0);
}