struct VertexInput {
    @location(0) pos:   vec2<f32>,
};

struct VertexOutput {
    @builtin(position) coords:     vec4<f32>,
    @location(0)       tex_coords: vec2<f32>,
};

@group(0) @binding(0)
var r_texture: texture_2d<f32>;

@group(0) @binding(1)
var r_sampler: sampler;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var vs_out: VertexOutput;
    vs_out.coords     = vec4<f32>(model.pos, 0.0, 1.0);
    vs_out.tex_coords.x = 0.5 + 0.5 * model.pos.x;
    vs_out.tex_coords.y = 0.5 - 0.5 * model.pos.y;

    return vs_out;
}

@fragment
fn fs_main(
    vs_out: VertexOutput
) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(r_texture, r_sampler, vs_out.tex_coords);
    return vec4<f32>(1.0 - color.rgb, 1.0);
}
