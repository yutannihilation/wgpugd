struct VertexInput {
    @location(0) pos:   vec2<f32>;
    @location(1) color: u32;
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

    // R's color representation is in the order of Alpha, Blue, Green, and Red. So,
    // we need to flip the order. Besides, it seems SVG spec doesn't accept
    // "#RRGGBBAA" format. unpack4x8unorm() is the function for this.
    //
    // https://github.com/wch/r-source/blob/8ebcb33a9f70e729109b1adf60edd5a3b22d3c6f/src/include/R_ext/GraphicsDevice.h#L766-L796 
    // https://www.w3.org/TR/WGSL/#unpack-builtin-functions
    out.color = unpack4x8unorm(model.color);

    // Scale the positions to [-1, 1]
    out.coords = vec4<f32>(2.0 * model.pos / globals.resolution - 1.0, 0.0, 1.0);

    return out;
}

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
