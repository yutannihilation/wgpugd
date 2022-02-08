struct VertexInput {
    @location(0) pos:   vec2<f32>;
    @location(1) color: vec4<f32>;
};

struct VertexOutput {
    @builtin(position) coords: vec4<f32>;
    @location(0)       color:  vec4<f32>;
};

struct GlobalsUniform {
    @location(0) resolution: vec2<f32>;
};

@group(0)
@binding(0)
var<uniform> globals: GlobalsUniform;

@stage(vertex)
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Use the input color as is.
    out.color  = model.color;

    // Scale the positions to [-1, 1]
    out.coords = vec4<f32>(2.0 * model.pos / globals.resolution - 1.0, 0.0, 1.0);

    return out;
}

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
