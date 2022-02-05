use extendr_api::{
    graphics::{
        ClippingStrategy, DevDesc, DeviceDescriptor, DeviceDriver, R_GE_gcontext, TextMetric,
    },
    prelude::*,
};

#[allow(dead_code)]
struct WgpuDevice {
    // TODO
}

const POINT: f64 = 12.0;

impl DeviceDriver for WgpuDevice {
    const CLIPPING_STRATEGY: ClippingStrategy = ClippingStrategy::Device;
}

// R's color representation is in the order of Alpha, Blue, Green, and Red. So,
// we need to flip the order. Besides, it seems SVG spec doesn't accept
// "#RRGGBBAA" format.
//
// https://github.com/wch/r-source/blob/8ebcb33a9f70e729109b1adf60edd5a3b22d3c6f/src/include/R_ext/GraphicsDevice.h#L766-L796
fn i32_to_csscolor(x: i32) -> String {
    if x.is_na() {
        return "transparent".to_string();
    }

    let x: u32 = unsafe { std::mem::transmute(x) };

    let r = x & 255;
    let g = (x >> 8) & 255;
    let b = (x >> 16) & 255;
    let a = (x >> 24) & 255;

    todo!()
}

/// A graphic device that does nothing
///
/// @param width  Device width in inch.
/// @param height Device width in inch.
/// @export
#[extendr]
fn wgpugd(width: i32, height: i32) {
    // Typically, 72 points per inch
    let width_pt = width * 72;
    let height_pt = height * 72;

    let device_driver = WgpuDevice {};

    let device_descriptor =
        // In SVG's coordinate y=0 is at top, so, we need to flip it by setting bottom > top.
        DeviceDescriptor::new().device_size(0.0, width_pt as _, height_pt as _, 0.0);

    device_driver.create_device::<WgpuDevice>(device_descriptor, "wgpugd");
}

extendr_module! {
    mod wgpugd;
    fn wgpugd;
}
