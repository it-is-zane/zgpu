@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var<storage, read> quads: array<Quad>;

struct VertexIn {
	@location(0) position: vec2<f32>,
	@location(1) uv: vec2<f32>,
}

struct Quad {
	rect: vec4<f32>,
	uv_rect: vec4<f32>,
}

struct VertexOut {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

struct FragmentOut {
    @location(0) color: vec4<f32>,
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
fn vs_main(
	@builtin(vertex_index) vertex: u32,
	@builtin(instance_index) instance: u32,
) -> VertexOut {
	let quad = quads[instance];

	var pos = array<vec2<f32>, 4>(
		quad.rect.xy,
		quad.rect.xw,
		quad.rect.zy,
		quad.rect.zw,
	);

	var uv = array<vec2<f32>, 4>(
		quad.uv_rect.xy,
		quad.uv_rect.xw,
		quad.uv_rect.zy,
		quad.uv_rect.zw,
	);


	return VertexOut(
		vec4(pos[vertex], 0.0, 1.0),
		vec2(uv[vertex]),
	);
}

@fragment
fn fs_main(vertex: VertexOut) -> FragmentOut {
	return FragmentOut(
		textureSampleBicubic(texture, tex_sampler, vertex.uv) * vec4(0.0,0.0,0.0,1.0) + vec4(0.0,0.0,0.0,0.5),
	);
}
