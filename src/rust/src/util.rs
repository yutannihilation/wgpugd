use extendr_api::prelude::*;

// R's color representation is in the order of Alpha, Blue, Green, and Red. So,
// we need to flip the order. Besides, it seems SVG spec doesn't accept
// "#RRGGBBAA" format.
//
// https://github.com/wch/r-source/blob/8ebcb33a9f70e729109b1adf60edd5a3b22d3c6f/src/include/R_ext/GraphicsDevice.h#L766-L796
pub(crate) fn i32_to_rgba(x: i32) -> [f32; 4] {
    if x.is_na() {
        return [0.0, 0.0, 0.0, 0.0];
    }

    let x: u32 = unsafe { std::mem::transmute(x) };

    let r = x & 255;
    let g = (x >> 8) & 255;
    let b = (x >> 16) & 255;
    let a = (x >> 24) & 255;

    [
        (r as f32) / 255.,
        (g as f32) / 255.,
        (b as f32) / 255.,
        (a as f32) / 255.,
    ]
}
