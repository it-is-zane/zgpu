@group(0) @binding(0) var<uniform> rectangle: vec4<f32>;
@group(0) @binding(1) var texture_binding: texture_2d<f32>;
@group(0) @binding(2) var sampler_binding: sampler;

struct Vertex {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
	@builtin(vertex_index) in_vertex_index: u32,
) -> Vertex {
	var out: Vertex;

	let uv = array<vec2<f32>, 4>(
		vec2(0.0, 1.0),
		vec2(0.0, 0.0),
		vec2(1.0, 1.0),
		vec2(1.0, 0.0),
	)[in_vertex_index];

	out.clip_position = vec4<f32>((uv + rectangle.xy) * rectangle.zw * 2.0 + vec2(-1.0, -1.0), vec2(0.0, 1.0));

	out.uv = uv;

	return out;
}

@fragment
fn fs_main(in: Vertex) -> @location(0) vec4<f32> {
	return textureSample(texture_binding, sampler_binding, in.uv);
}
