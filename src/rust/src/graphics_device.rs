use extendr_api::{
    graphics::{ClippingStrategy, DevDesc, DeviceDriver, R_GE_gcontext},
    prelude::*,
};

use lyon::path::Path;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{FillOptions, FillTessellator};
use lyon::tessellation::{StrokeOptions, StrokeTessellator, StrokeVertex};

struct VertexCtor {
    color: [f32; 4],
}

impl StrokeVertexConstructor<crate::Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> crate::Vertex {
        let pos = vertex.position();
        crate::Vertex {
            position: pos.into(),
            color: self.color,
        }
    }
}

impl DeviceDriver for crate::WgpuGraphicsDevice {
    const CLIPPING_STRATEGY: ClippingStrategy = ClippingStrategy::Device;

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        reprintln!("[DEBUG] from: {from:?}, to: {to:?}");

        // TODO: this should be super slow because this allocates Vec all the times.
        // Probably we need a buffer for PathEvent and render it on flush.
        let mut builder = Path::builder();

        // TODO: Move the calculation to shader
        builder.begin(lyon::math::point(
            2f32 * from.0 as f32 * self.x_scale - 1f32,
            2f32 * from.1 as f32 * self.y_scale - 1f32,
        ));
        builder.line_to(lyon::math::point(
            2f32 * to.0 as f32 * self.x_scale - 1f32,
            2f32 * to.1 as f32 * self.y_scale - 1f32,
        ));
        builder.close();
        let path = builder.build();

        let mut stroke_tess = StrokeTessellator::new();
        let stroke_options = &StrokeOptions::tolerance(0.01).with_line_width(0.2);

        let ctxt = VertexCtor {
            color: crate::util::i32_to_rgba(gc.col),
        };

        stroke_tess
            .tessellate_path(
                &path,
                stroke_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    fn close(&mut self, _: DevDesc) {
        rprintln!("[DEBUG] vertex: {:?}", self.geometry.vertices);
        rprintln!("[DEBUG] index: {:?}", self.geometry.indices);

        self.render().unwrap();
        pollster::block_on(self.write_png());
    }
}
