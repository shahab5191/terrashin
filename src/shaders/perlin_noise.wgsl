struct Uniforms {
    scale: f32,
}

@group(0) @binding(0) var tex_sampler: sampler;
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}
fn pcg(v: u32) -> u32 {
    let state = v * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn hash2(ix: u32, iy: u32) -> u32 {
    return pcg(ix + pcg(iy));
}

fn fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    return a + t * (b - a);
}

fn gradient(ix: u32, iy: u32) -> vec2<f32> {
    let h = hash2(ix, iy);
    let angle = f32(h) * 2.3283064365387e-10 * 6.28318530718;
    return vec2<f32>(cos(angle), sin(angle));
}

fn dot_grid_gradient(ix: u32, iy: u32, px: f32, py: f32) -> f32 {
    let grad = gradient(ix, iy);
    let dx = px - f32(ix);
    let dy = py - f32(iy);
    return dx * grad.x + dy * grad.y;
}

fn perlin(uv: vec2<f32>, scale: f32) -> f32 {
    let p = uv * scale;

    // grid cell corners
    let x0 = u32(floor(p.x));
    let y0 = u32(floor(p.y));
    let x1 = x0 + 1u;
    let y1 = y0 + 1u;

    // position within cell [0,1]
    let fx = fract(p.x);
    let fy = fract(p.y);

    // smooth the interpolation weights
    let ux = fade(fx);
    let uy = fade(fy);

    // dot products at each corner
    let n00 = dot_grid_gradient(x0, y0, p.x, p.y);
    let n10 = dot_grid_gradient(x1, y0, p.x, p.y);
    let n01 = dot_grid_gradient(x0, y1, p.x, p.y);
    let n11 = dot_grid_gradient(x1, y1, p.x, p.y);

    // bilinear interpolation
    let nx0 = mix(n00, n10, ux);
    let nx1 = mix(n01, n11, ux);
    return mix(nx0, nx1, uy);
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((i32(in_vertex_index) << 1u) & 2) * 2.0 - 1.0;
    let y = f32(i32(in_vertex_index) & 2) * 2.0 - 1.0;
    out.tex_coords = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = perlin(in.tex_coords, uniforms.scale);
    let value = n * 0.5 + 0.5;
    return vec4<f32>(value, value, value, 1.0);
}
