struct Uniforms {
    scale: f32,
}

@group(0) @binding(0) var color1_tex: texture_2d<f32>;
@group(0) @binding(1) var color2_tex: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;
@group(0) @binding(3) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index) << 1u & 2) * 2.0 - 1.0;
    let y = f32(i32(in_vertex_index) & 2) * 2.0 - 1.0;
    out.tex_coords = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scaled = in.tex_coords * uniforms.scale;
    let checker = (floor(scaled.x) + floor(scaled.y)) % 2.0;
    let c1 = textureSample(color1_tex, tex_sampler, in.tex_coords);
    let c2 = textureSample(color2_tex, tex_sampler, in.tex_coords);
    return select(c2, c1, checker < 1.0);
}
