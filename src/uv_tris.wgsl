@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

struct VertexIn {
	@location(0) position: vec2<f32>,
	@location(1) uv: vec2<f32>,
}

struct VertexOut {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) uv: vec2<f32>,
}

struct FragmentOut {
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
	return VertexOut(
		vec4(vertex.position, 0.0, 1.0),
		vertex.uv,
	);
}

@fragment
fn fs_main(vertex: VertexOut) -> FragmentOut {
	return FragmentOut(
		textureSample(texture, tex_sampler, vertex.uv),
	);
}
