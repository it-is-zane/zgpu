@group(0) @binding(0) var myTexture: texture_2d<f32>;
@group(0) @binding(1) var mySampler: sampler;

@group(1) @binding(0) var<uniform> model: mat4x4<f32>;


struct Vertex {
  @location(0) position: vec3<f32>,
  @location(1) color: vec3<f32>,
};

struct VertexPayload {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec3<f32>,
  @location(1) textureCoord: vec2<f32>,
}

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

fn cubic(v: f32) -> vec4<f32> {
    let n = vec4(1.0, 2.0, 3.0, 4.0) - v;
    let s = n * n * n;
    let x = s.x;
    let y = s.y - 4.0 * s.x;
    let z = s.z - 4.0 * s.y + 6.0 * s.x;
    let w = 6.0 - x - y - z;
    return vec4(x, y, z, w) * (1.0/6.0);
}

fn textureSampleBicubic(tex: texture_2d<f32>, tex_sampler: sampler, texCoords_original: vec2<f32>) -> vec4<f32> {
    var texture_size = vec2<f32>(textureDimensions(tex).xy);

    var invTexSize = 1.0 / texture_size;
   
    var texCoords = texCoords_original * texture_size - 0.5;

    var fxy = fract(texCoords);
    texCoords = texCoords - fxy;

    var xcubic = cubic(fxy.x);
    var ycubic = cubic(fxy.y);

    var c = texCoords.xxyy + vec2(-0.5, 1.5).xyxy;
    
    var s = vec4(xcubic.xz + xcubic.yw, ycubic.xz + ycubic.yw);
    var offset = c + vec4(xcubic.yw, ycubic.yw) / s;
    
    offset = offset * invTexSize.xxyy;
    
    var sample0 = textureSample(tex, tex_sampler, offset.xz);
    var sample1 = textureSample(tex, tex_sampler, offset.yz);
    var sample2 = textureSample(tex, tex_sampler, offset.xw);
    var sample3 = textureSample(tex, tex_sampler, offset.yw);

    var sx = s.x / (s.x + s.y);
    var sy = s.z / (s.z + s.w);

    return mix(mix(sample3, sample2, vec4(sx)), mix(sample1, sample0, vec4(sx)), vec4(sy));
}

@vertex
fn vs_main(vertex: Vertex) -> VertexPayload {
  var out: VertexPayload;

  out.position = model * vec4<f32>(vertex.position, 1.0);
  out.color = vertex.color;
  out.textureCoord = vec2<f32>(vertex.position.xy);

  return out;
}

@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
  var rgba = textureSampleBicubic(myTexture, mySampler, in.color.xy);
  var lab = linear_srgb_to_oklab(rgba.rgb);
  var lch = lab_to_lch(lab);
  lch.x = 1.0 - lch.x;
  // lch.y = 1.0 - lch.y;
  lch.y = 1.0 - smoothstep(0.0, 0.1, lch.y);
  lab = lch_to_lab(lch);
  var rgb = oklab_to_linear_srgb(lab);

  // return vec4(lch.yyy, 1.0);
  return vec4(rgb, 0.5);
  // return rgba;
}
