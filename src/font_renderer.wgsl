@group(0) @binding(0) var<storage> segments: array<array<vec2<f32>, 2>>;
@group(0) @binding(2) var<storage> quadratics: array<array<vec2<f32>, 3>>;
@group(0) @binding(1) var<storage> cubics: array<array<vec2<f32>, 4>>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

fn sdf_segment(
  p: vec2<f32>,
  a: vec2<f32>,
  b: vec2<f32>
) -> f32 {
  let pa = p - a;
  let ba = b - a;
  let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);

  return length(pa - ba * h);
}


fn sdf_quadratic_bezier(
  pos: vec2<f32>,
  A: vec2<f32>,
  B: vec2<f32>,
  C: vec2<f32>,
  D: vec2<f32>
) -> f32 {
  return 1000000.0;
}

fn sdf_cubic_bezier(
  p: vec2<f32>,
  a: vec2<f32>,
  b: vec2<f32>,
  c: vec2<f32>
) -> f32 {
  return 1000000.0;
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
  var d = 1000000.0;

  for (var i = 0; i < arrayLength(segments); i++) {
    d = min(
      sdf_segment(
        uv,
        segments[i][0],
        segments[i][1]
      ),
      d
    );
  }
  for (var i = 0; i < arrayLength(quadriatics); i++) {
    d = min(
      sdf_quadratic_bezier(
        uv,
        segments[i][0],
        segments[i][1],
        segments[i][2]
      ),
      d
    );
  }
  for (var i = 0; i < arrayLength(cubics); i++) {
    d = min(
      sdf_cubic_bezier(
        uv,
        segments[i][0],
        segments[i][1],
        segments[i][2],
        segments[i][3]
      ),
      d
    );
  }

  return vec4(vec3(d), 1.0);
}
