@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;
@group(0) @binding(1) var texture_binding: texture_2d<f32>;
@group(0) @binding(2) var sampler_binding: sampler;



struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

fn linear_srgb_to_oklab(rgb: vec3<f32>) -> vec3<f32> {
  var l: f32 = 0.4122214708 * rgb.r + 0.5363325363 * rgb.g + 0.0514459929 * rgb.b;
  var m: f32 = 0.2119034982 * rgb.r + 0.6806995451 * rgb.g + 0.1073969566 * rgb.b;
  var s: f32 = 0.0883024619 * rgb.r + 0.2817188376 * rgb.g + 0.6299787005 * rgb.b;

  var l_: f32 =  pow(l, 1.0 / 3.0);
  var m_: f32 =  pow(m, 1.0 / 3.0);
  var s_: f32 =  pow(s, 1.0 / 3.0);

  return vec3(
    0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
    1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
    0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
  );
}

fn oklab_to_linear_srgb(lab: vec3<f32>) -> vec3<f32> {
  var l_: f32 = lab.r + 0.3963377774 * lab.g + 0.2158037573 * lab.b;
  var m_: f32 = lab.r - 0.1055613458 * lab.g - 0.0638541728 * lab.b;
  var s_: f32 = lab.r - 0.0894841775 * lab.g - 1.2914855480 * lab.b;

  var l: f32 = l_*l_*l_;
  var m: f32 = m_*m_*m_;
  var s: f32 = s_*s_*s_;

  return vec3(
	4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
	-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
	-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
  );
}

fn lab_to_lch(lab: vec3<f32>) -> vec3<f32> {
  return vec3(
    lab.x,
    sqrt(lab.y * lab.y + lab.z * lab.z),
    atan2(lab.z, lab.y)
  );
}

fn lch_to_lab(lch: vec3<f32>) -> vec3<f32> {
  return vec3(
    lch.x,
    lch.y * cos(lch.z),
    lch.y * sin(lch.z)
  );
}

fn sdf_bezier(pos: vec2<f32>, A: vec2<f32>, B: vec2<f32>, C: vec2<f32>) -> f32 {

    let a: vec2<f32> = B - A;
    let b: vec2<f32> = A - 2.0*B + C;
    let c: vec2<f32> = a * 2.0;
    let d: vec2<f32> = A - pos;
    let kk: f32 = 1.0 / dot(b, b);
    let kx: f32 = kk * dot(a, b);
    let ky: f32 = kk * (2.0 * dot(a, a) + dot(d, b)) / 3.0;
    let kz: f32 = kk * dot(d,a);      

    var res: f32 = 0.0;

    let p: f32 = ky - kx*kx;
    let p3: f32 = p * p * p;
    let q: f32 = kx * (2.0 * kx * kx - 3.0 * ky) + kz;

    var h: f32 = q * q + 4.0 * p3;

    if (h >= 0.0) { 
        h = sqrt(h);

        let x: vec2<f32> = (vec2(h,-h)-q)/2.0;
        let uv: vec2<f32> = sign(x)*pow(abs(x), vec2(1.0/3.0));
        let t: f32 = clamp( uv.x+uv.y-kx, 0.0, 1.0 );

        res = dot(d + (c + b * t) * t, d + (c + b * t) * t);
    }
    else {
        let z: f32 = sqrt(-p);
        let v: f32 = acos(q / (p * z * 2.0)) / 3.0;
        let m: f32 = cos(v);
        let n: f32 = sin(v) * 1.732050808;
        let t: vec3<f32> = clamp(vec3(m + m, -n - m, n - m) * z - kx, vec3(0.0), vec3(1.0));

        res = min( dot(d + (c + b * t.x) * t.x, d + (c + b * t.x) * t.x),
                   dot(d + (c + b * t.y) * t.y, d + (c + b * t.y) * t.y));
        // the third root cannot be the closest
        // res = min(res,dot2(d+(c+b*t.z)*t.z));
    }

    return sqrt(res);
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let x: f32 = array<f32, 3>(1.0, 1.0, -3.0)[in_vertex_index];
    let y: f32 = array<f32, 3>(-3.0, 1.0, 1.0)[in_vertex_index];

    // let x = f32(1 - i32(in_vertex_index)) * 0.5;
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = vec3<f32>(x, y, 0.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = (in.color.xy + vec2(1.0)) / 2.0 * vec2(1.0, -1.0) + vec2(0.0, 1.0);
    let screen_coord = uv * screen_size;

    // return vec4(screen_coord, vec2(0.0));
    return textureSample(texture_binding, sampler_binding, screen_coord / 100.0);
    
}
