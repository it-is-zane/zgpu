struct Segment {
  a: vec2<f32>,
  b: vec2<f32>,
}

@group(0) @binding(0) var<storage, read> segments: array<Segment>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

fn sdf_segment(
  p: vec2<f32>,
  a: vec2<f32>,
  b: vec2<f32>,
) -> f32 {
  let pa = p - a;
  let ba = b - a;
  let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);


  return length(pa - ba * h);
}

fn sdf(p: vec2<f32>) -> f32 {
  var d = 1e6;
  var n = vec2(0.0);
  var s = 0.0;

  for (var i = 0u; i < arrayLength(&segments); i++) {
      let a = segments[i].a;
      let b = segments[i].b;
  
      let new_d = sdf_segment(p, a, b);
      let new_n = (b - a).yx * vec2(1.0, -1.0);

      if (abs(new_d) < abs(d)) {
        d = new_d;
        n = new_n;

        s = sign(dot(n, p - a));
      }

      if (abs(abs(new_d) - abs(d)) <= 0.001) {
        let s_old = sign(dot(n, p - a));
        let s_new = sign(dot(new_n, p - a));

        if (s_old != s_new) {
          d = new_d;
          n = normalize(n) + normalize(new_n);

          s = sign(dot(n, p - a));
        }

      }

      n += normalize(new_n) / new_d;
  }

  return d * s;
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let x: f32 = array<f32, 3>(1.0, 1.0, -3.0)[in_vertex_index];
    let y: f32 = array<f32, 3>(-3.0, 1.0, 1.0)[in_vertex_index];

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = vec3<f32>(x, y, 0.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let uv = in.color.xy;

  let d = sdf(uv * 1000.0);
  
  return vec4(f32(d < 0.0));
}
