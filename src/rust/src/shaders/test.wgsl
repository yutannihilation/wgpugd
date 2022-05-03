struct GlobalsUniform {
    @location(0) @size(16) resolution: vec2<f32>,
    @location(1) mat_array:  array<mat2x2<f32>, 3>,
};

@group(0) @binding(0)
var<uniform> globals: GlobalsUniform;
