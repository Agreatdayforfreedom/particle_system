
struct Particle {
  position: vec4f,
  dir: vec3f,
  velocity: f32,
  color: vec4<f32>,
}

@binding(0) @group(0) var<storage, read_write> particles_dst : array<Particle>;

@compute @workgroup_size(64)
fn loader(@builtin(global_invocation_id) global_invocation_id: vec3u) {
    let idx = global_invocation_id.x;

    if (idx >= total) {
      return;
    }
    
    var particle: Particle = particles_dst[idx];
    
    particle.position = vec4(0.0, 0.0, 0.0, 1.0);
    particle.dir = vec3(0.0, 1.0, 1.0);
    particle.velocity = 1.0;
    particle.color = vec4(1.0,1.0,0.0,1.0);

    particles_dst[idx] = particle;
}