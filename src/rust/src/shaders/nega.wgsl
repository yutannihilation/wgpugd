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

// c.f. https://babylonjs.medium.com/retro-crt-shader-a-post-processing-effect-study-1cb3f783afbc
let CURVATURE: vec2<f32> = vec2<f32>(3.0, 3.0);
let RESOLUTION: vec2<f32> = vec2<f32>(100.0, 100.0);
let BRIGHTNESS: f32 = 4.0;

let PI: f32 = 3.14159;

fn curveRemapUV(uv_in: vec2<f32>) -> vec2<f32> {
    var uv_out: vec2<f32>;

    uv_out = uv_in * 2.0 - 1.0;
    // as we near the edge of our screen apply greater distortion using a cubic function    uv = uv * 2.0–1.0;
    var offset: vec2<f32> = abs(uv_out.yx) / CURVATURE;

    uv_out = uv_out + uv_out * offset * offset;
    return uv_out * 0.5 + 0.5;
}

fn scanLineIntensity(uv_in: f32, resolution: f32, opacity: f32) -> vec4<f32> {
     var intensity: f32 = sin(uv_in * resolution * PI * 2.0);
     intensity = ((0.5 * intensity) + 0.5) * 0.9 + 0.1;
     return vec4<f32>(vec3<f32>(pow(intensity, opacity)), 1.0);
 }

fn vignetteIntensity(uv_in: vec2<f32>, resolution: vec2<f32>, opacity: f32, roundness: f32) -> vec4<f32> {
    var intensity: f32 = uv_in.x * uv_in.y * (1.0 - uv_in.x) * (1.0 - uv_in.y);
    return vec4<f32>(vec3<f32>(clamp(pow((resolution.x / roundness) * intensity, opacity), 0.0, 1.0)), 1.0);
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var vs_out: VertexOutput;

    vs_out.coords = vec4<f32>(model.pos, 0.0, 1.0);
    
    vs_out.tex_coords.x = 0.5 + 0.5 * model.pos.x;
    vs_out.tex_coords.y = 0.5 - 0.5 * model.pos.y;

    return vs_out;
}

@fragment
fn fs_main(
    vs_out: VertexOutput
) -> @location(0) vec4<f32> {
    var remapped_tex_coords = curveRemapUV(vs_out.tex_coords);
    var color: vec4<f32> = textureSample(r_texture, r_sampler, remapped_tex_coords);
    
    color *= vignetteIntensity(remapped_tex_coords, RESOLUTION, 1.0, 2.0);
    
    color *= scanLineIntensity(remapped_tex_coords.x, RESOLUTION.y, 1.0);
    color *= scanLineIntensity(remapped_tex_coords.y, RESOLUTION.x, 1.0);
    
    return vec4<f32>(color.rgb * BRIGHTNESS, 1.0);
}
