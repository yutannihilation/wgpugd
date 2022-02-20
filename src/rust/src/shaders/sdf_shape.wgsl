struct VertexInput {
    @location(0) pos: vec2<f32>;
};

struct VertexOutput {
    @builtin(position) coords: vec4<f32>;
    @location(0) center:       vec2<f32>;
    @location(1) radius:       f32;
    @location(2) stroke_width: f32;
    @location(3) fill_color:   u32;
    @location(4) stroke_color: u32;
    
};

let MAX_LAYERS = 8;

struct GlobalsUniform {
    @location(0) resolution:      vec2<f32>;
    @location(1) layer_clippings: array<mat2x2<f32>, MAX_LAYERS>;
};

@group(0)
@binding(0)
var<uniform> globals: GlobalsUniform;

struct InstanceInput {
    @location(1) center:       vec2<f32>;
    @location(2) radius:       f32;
    @location(3) stroke_width: f32;
    @location(4) fill_color:   u32;
    @location(5) stroke_color: u32;
    @location(6) z:            f32;
};

@stage(vertex)
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.coords = vec4<f32>(model.pos, instance.z, 1.0);
    // Y-axis is opposite
    out.center = vec2<f32>(instance.center.x, globals.resolution.y - instance.center.y);
    out.radius = instance.radius;
    out.stroke_width = instance.stroke_width;
    out.fill_color = instance.fill_color;
    out.stroke_color = instance.stroke_color;
    
    return out;
}

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var dist = distance(in.coords.xy, in.center) - in.radius;
    if (dist > 0.5) {
        return vec4<f32>(0.0);
    } else {
        var out: vec4<f32> = unpack4x8unorm(in.fill_color);
        // TODO: A poor-man's anti-aliasing. I don't know how to do it correctly atm...
        out.a *= clamp(0.5 - dist, 0.0, 1.0);
        return out;
    }
}
