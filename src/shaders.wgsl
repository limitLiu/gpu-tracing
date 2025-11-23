const FLT_MAX: f32 = 3.40282346638528859812e+38;

struct Uniforms {
  width: u32,
  height: u32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct Ray {
  origin: vec3f,
  direction: vec3f,
}

struct Sphere {
  center: vec3f,
  radius: f32,
}

const OBJECT_COUNT: u32 = 2;

alias Scene = array<Sphere, OBJECT_COUNT>;
var<private> scene: Scene = Scene(
  Sphere(vec3(0., 0., -1.), 0.5),
  Sphere(vec3(0., -100.5, -1.), 100.),
);

alias TriangleVertices = array<vec2f, 6>;

var<private> vertices: TriangleVertices = TriangleVertices(
// top left
vec2f(-1.,  1.),
// top right
vec2f(-1., -1.),
// bottom left
vec2f( 1.,  1.),

// bottom left
vec2f( 1.,  1.),
// top right
vec2f(-1., -1.),
// bottom right
vec2f( 1., -1.)
);

fn intersect_sphere(ray: Ray, sphere: Sphere) -> f32 {
  let v = ray.origin - sphere.center;
  let a = dot(ray.direction, ray.direction);
  let b = dot(v, ray.direction);
  let c = dot(v, v) - sphere.radius * sphere.radius;

  let d = b * b - a * c;
  if d < 0. {
    return -1.;
  }

  let sqrt_d = sqrt(d);
  let recip_a = 1. / a;
  let mb = -b;
  let t = (mb - sqrt_d) * recip_a;
  if t > 0. {
    return t;
  }
  return (mb + sqrt_d) * recip_a;
}

fn sky_color(ray: Ray) -> vec3f {
  let t = 0.5 * (normalize(ray.direction).y + 1.);
  return (1. - t) * vec3(1.) + t * vec3(0.3, 0.5, 1.);
}

@vertex fn display_vs(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4f {
  return vec4f(vertices[vid], 0., 1.);
}

@fragment fn display_fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
  let origin = vec3f(0.);
  let focus_distance = 1.;
  let aspect_ratio = f32(uniforms.width) / f32(uniforms.height);
  var uv = pos.xy / vec2f(f32(uniforms.width - 1u), f32(uniforms.height - 1u));
  uv = (2. * uv - vec2f(1.)) * vec2f(aspect_ratio, -1.);
  let direction = vec3f(uv, -focus_distance);
  let ray = Ray(origin, direction);
  var closest_t = FLT_MAX;
  for (var i = 0u; i < OBJECT_COUNT; i += 1u) {
    let t = intersect_sphere(ray, scene[i]);
    if t > 0. && t < closest_t {
      closest_t = t;
    }
  }
  if closest_t < FLT_MAX {
    // return vec4(1, 0.76, 0.03, 1);
    // return vec4(1, 0.76, 0.03, 1) * saturate(1. - closest_t);
    return vec4(saturate(closest_t) * 0.5);
  }
  return vec4f(sky_color(ray), 1.);
}
