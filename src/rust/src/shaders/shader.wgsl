struct VertexInput {
    @location(0) pos:         vec3<f32>,
    @location(1) color:       u32,
    @location(2) clipping_id: i32,
};

struct VertexOutput {
    @builtin(position) coords: vec4<f32>,
    @location(0) color:        u32,
    @location(1) clipping_id:  i32,
};

let MAX_LAYERS = 8;

struct GlobalsUniform {
    @location(0) resolution:      vec2<f32>,
    // TODO: this @size annotation is needed otherwise I get "offset 8 is not a multiple of the required alignment 16" error
    @align(16)
    @location(1) layer_clippings: array<mat2x2<f32>, MAX_LAYERS>,
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

    vs_out.clipping_id = model.clipping_id;

    return vs_out;
}

@fragment
fn fs_main(
    vs_out: VertexOutput
) -> @location(0) vec4<f32> {
    var within_clip: bool = false;

    // If the clipping ID is negative, no clipping
    if (vs_out.clipping_id < 0) {
        within_clip = true;
    } else {
        // Note to self: at the fragment stage, `position` represents the 2D pixel
        // position in framebuffer space, which is NOT [-1, 1].
        var bottom_left: vec2<f32> = vec2<f32>(
            globals.layer_clippings[vs_out.clipping_id][0].x,
            // Y-axis is from top to bottom, so the coordinates needs to be flipped.
            // TODO: probably I can write this more nicely with vector products
            globals.resolution.y - globals.layer_clippings[vs_out.clipping_id][1].y,
        );
        var top_right:   vec2<f32> = vec2<f32>(
            globals.layer_clippings[vs_out.clipping_id][1].x,
            globals.resolution.y - globals.layer_clippings[vs_out.clipping_id][0].y,
        );

        // TOOD: Can I do this more nicely with vector products? (c.f.
        // https://math.stackexchange.com/a/190373)
        within_clip = all((bottom_left <= vs_out.coords.xy) & (vs_out.coords.xy <= top_right));
    }

    if (!within_clip) {
        return vec4<f32>(0.0);
    }

    // R's color representation is in the order of RGBA, which can be simply
    // unpacked by unpack4x8unorm().
    //
    // https://github.com/wch/r-source/blob/8ebcb33a9f70e729109b1adf60edd5a3b22d3c6f/src/include/R_ext/GraphicsDevice.h#L766-L796
    // https://www.w3.org/TR/WGSL/#unpack-builtin-functions
    return unpack4x8unorm(vs_out.color);
}
