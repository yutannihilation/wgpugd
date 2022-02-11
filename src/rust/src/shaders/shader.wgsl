struct VertexInput {
    @location(0) pos:   vec2<f32>;
    @location(1) color: u32;
    @location(2) layer: u32;
};

struct VertexOutput {
    @builtin(position) coords: vec4<f32>;
    @location(0)       color:  u32;
    // TODO: this probably can be stored in coords?
    @location(1)       layer:  u32;
};

let MAX_LAYERS = 8;

struct GlobalsUniform {
    @location(0) resolution: vec2<f32>;
    @location(1) layer_clippings: array<mat2x2<f32>, MAX_LAYERS>;
};

@group(0)
@binding(0)
var<uniform> globals: GlobalsUniform;

@stage(vertex)
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = model.color;

    // Scale the positions to [-1, 1]
    out.coords = vec4<f32>(2.0 * model.pos / globals.resolution - 1.0, 0.0, 1.0);

    out.layer = model.layer;

    return out;
}

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = vec4<f32>(0.0);

    // Note to self: at the fragment stage, `position` represents the 2D pixel
    // position in framebuffer space, which is NOT [-1, 1].
    var bottom_left: vec2<f32> = globals.layer_clippings[in.layer][0];
    var top_right:   vec2<f32> = globals.layer_clippings[in.layer][1];

    // TOOD: Can I do this more nicely with vector products?
    if (all((bottom_left <= in.coords.xy) & (in.coords.xy <= top_right))) {
        // R's color representation is in the order of RGBA, which can be simply
        // unpacked by unpack4x8unorm().
        //
        // https://github.com/wch/r-source/blob/8ebcb33a9f70e729109b1adf60edd5a3b22d3c6f/src/include/R_ext/GraphicsDevice.h#L766-L796
        // https://www.w3.org/TR/WGSL/#unpack-builtin-functions
        color = unpack4x8unorm(in.color);
    }


    return color;
}
