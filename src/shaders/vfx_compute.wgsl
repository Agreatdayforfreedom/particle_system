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
  attractors: mat4x4f
}

struct Particle {
  position: vec4f,
  dir: vec3f,
  velocity: f32,
}


@binding(0) @group(0) var<storage, read_write> particles_dst : array<Particle>;
@binding(1) @group(0) var<storage> sim_params_groups: SimulationParams;
@binding(2) @group(0) var<uniform> uniforms: Uniforms;

@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) global_invocation_id : vec3u) {
    let total = arrayLength(&particles_dst);
  
    var a = 20.0;			
    var b = 16.0 / 3.0;
    var c = 38.0;
    let idx = global_invocation_id.x;
  
    if (idx >= total) {
      return;
    }
    
    var particle: Particle = particles_dst[idx];
    init_rand(idx, vec4f(particle.position.x, particle.position.y, particle.position.z, uniforms.delta_time));

    if (particle.position.w <= 0.0) {
      // particle.dir *= 0.01;
      // particle.position = vec4(particle.position.xyz * 0.1, particle.position.w);
      particle.position.w += 1.0;
    }

    let distance = sqrt(
      particle.position.x * particle.position.x + 
      particle.position.y * particle.position.y + 
      particle.position.z * particle.position.z
      );
        let max_distance = sqrt(
          100.0*100.0+
          100.0*100.0+
          100.0*100.0
        );
   
   let float = clamp(0.0, 1.0, f32(distance / max_distance));

    let dx = a * (particle.position.y - particle.position.x);
    let dy = particle.position.x * (c - particle.position.z) - particle.position.y;
    let dz = particle.position.x * particle.position.y - b * particle.position.z;

    particle.position.x +=  dx * particle.dir.x * uniforms.delta_time;
    particle.position.y +=  dy * particle.dir.y * uniforms.delta_time;
    particle.position.z +=  dz * particle.dir.z * uniforms.delta_time;
    particle.position.w =  float;
    particles_dst[idx] = particle;
}