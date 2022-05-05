struct VertexInput {
    @location(0) pos:         vec3<f32>,
    @location(1) color:       u32,
};

struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
    @location(0) color:        u32,
};

struct GlobalsUniform {
    @location(0) resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> globals: GlobalsUniform;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var vs_out: VertexOutput;

    vs_out.color = model.color;

    // Scale the X and Y positions from [0, width or height] to [-1, 1]
    vs_out.coords = vec4<f32>(2.0 * model.pos.xy / globals.resolution - 1.0, model.pos.z, 1.0);

    return vs_out;
}

@fragment
fn fs_main(
    vs_out: VertexOutput
) -> @location(0) vec4<f32> {
    // R's color representation is in the order of RGBA, which can be simply
    // unpacked by unpack4x8unorm().
    //
    // https://github.com/wch/r-source/blob/8ebcb33a9f70e729109b1adf60edd5a3b22d3c6f/src/include/R_ext/GraphicsDevice.h#L766-L796
    // https://www.w3.org/TR/WGSL/#unpack-builtin-functions
    var color: vec4<f32> = unpack4x8unorm(vs_out.color);
    // return the alpha-premultiplied version of value
    return vec4<f32>(color.rgb * color.a, color.a);
}
