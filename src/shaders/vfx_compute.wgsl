const PI: f32 = 3.14159265358;

var<private> rand_seed : vec2<f32>;

fn init_rand(invocation_id : u32, seed : vec4<f32>) {
  rand_seed = seed.xz;
  rand_seed = fract(rand_seed * cos(35.456+f32(invocation_id) * seed.yw));
  rand_seed = fract(rand_seed * cos(41.235+f32(invocation_id) * seed.xw));
}

fn rand() -> f32 {
  rand_seed.x = fract(cos(dot(rand_seed, vec2<f32>(23.14077926, 232.61690225))) * 136.8168);
  rand_seed.y = fract(cos(dot(rand_seed, vec2<f32>(54.47856553, 345.84153136))) * 534.7645);
  return rand_seed.y;
}

fn gen_range(min: f32, max: f32) -> f32 {
  return min + (max - min) * rand();
}

struct SimulationParams {
    dir: vec2f,
}

struct Uniforms {
  delta_time: f32,
  time: f32,
}

struct Particle {
  position: vec2f,
  dir: vec2f,
  velocity: f32,
  lifetime: f32,
  // color: vec4f,
  // velocity: f32,
  // lifetime: f32,
  // actived: f32,
}


@binding(0) @group(0) var<storage, read_write> particles_dst : array<Particle>;
@binding(1) @group(0) var<storage> sim_params_groups: SimulationParams;
@binding(2) @group(0) var<uniform> uniforms: Uniforms;

@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) global_invocation_id : vec3u) {
    let total = arrayLength(&particles_dst);
  
    let idx = global_invocation_id.x;
  
    if (idx >= total) {
      return;
    }
    
    var particle: Particle = particles_dst[idx];
    init_rand(idx, vec4f(particle.position.x, particle.position.y, -uniforms.delta_time, uniforms.delta_time));

    if (particle.lifetime <= 0.0) {

      let angle = degrees(gen_range(0.0, 1.0) * 2.0 * PI);

      let x = cos(radians(angle));
      let y = sin(radians(angle));
      let dir = normalize(vec2f(x, y));
      particle.dir = dir;
      particle.velocity = gen_range(50.0, 150.0);
      particle.lifetime = rand();
      particle.position = vec2f(0.0, 0.0);
    }
    
    particle.lifetime -= uniforms.delta_time;
    particle.position.x += particle.velocity * particle.dir.x * uniforms.delta_time;
    particle.position.y += particle.velocity * particle.dir.y * uniforms.delta_time;
    particles_dst[idx] = particle;
}