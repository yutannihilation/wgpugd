use extendr_api::{
    graphics::{ClippingStrategy, DevDesc, DeviceDriver, R_GE_gcontext},
    prelude::*,
};

use lyon::path::{traits::PathBuilder, Path};
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{FillOptions, FillTessellator, FillVertex};
use lyon::tessellation::{StrokeOptions, StrokeTessellator, StrokeVertex};

struct VertexCtor {
    color: u32,
}

impl VertexCtor {
    fn new(color: i32) -> Self {
        Self {
            color: unsafe { std::mem::transmute(color) },
        }
    }
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

impl FillVertexConstructor<crate::Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: FillVertex) -> crate::Vertex {
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
        let color = gc.col;
        let line_width = gc.lwd as f32;
        // TODO: determine tolerance nicely
        let tolerance = 0.01;

        // TODO: this should be super slow because this allocates Vec all the times.
        // Probably we need a buffer for PathEvent and render it on flush.
        let mut builder = Path::builder();

        //
        // **** Build path ***************************
        //

        builder.begin(lyon::math::point(from.0 as _, from.1 as _));
        builder.line_to(lyon::math::point(to.0 as _, to.1 as _));
        builder.close();
        let path = builder.build();

        //
        // **** Tessellate stroke ***************************
        //

        let mut stroke_tess = StrokeTessellator::new();
        let stroke_options = &StrokeOptions::tolerance(tolerance).with_line_width(line_width);

        let ctxt = VertexCtor::new(color);

        stroke_tess
            .tessellate_path(
                &path,
                stroke_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    fn polygon<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        let color = gc.col;
        let fill = gc.fill;
        let line_width = gc.lwd as f32;

        // TODO: determine tolerance nicely
        let tolerance = 0.01;

        let mut builder = Path::builder();

        //
        // **** Build path ***************************
        //

        let mut coords = coords.into_iter();

        // First point
        let (x, y) = coords.next().unwrap();
        builder.begin(lyon::math::point(x as _, y as _));

        coords.for_each(|(x, y)| {
            builder.line_to(lyon::math::point(x as _, y as _));
        });
        builder.close();

        let path = builder.build();

        //
        // **** Tessellate fill ***************************
        //

        let mut fill_tess = FillTessellator::new();
        let fill_options = &FillOptions::tolerance(tolerance);

        let ctxt = VertexCtor::new(fill);

        fill_tess
            .tessellate_path(
                &path,
                fill_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();

        //
        // **** Tessellate stroke ***************************
        //

        let mut stroke_tess = StrokeTessellator::new();
        let stroke_options = &StrokeOptions::tolerance(tolerance).with_line_width(line_width);

        let ctxt = VertexCtor::new(color);

        stroke_tess
            .tessellate_path(
                &path,
                stroke_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        let color = gc.col;
        let fill = gc.fill;
        let line_width = gc.lwd as f32;
        // TODO: determine tolerance nicely
        let tolerance = 0.01;

        //
        // **** Tessellate fill ***************************
        //

        let mut fill_tess = FillTessellator::new();
        let fill_options = &FillOptions::tolerance(tolerance);

        let ctxt = VertexCtor::new(fill);

        fill_tess
            .tessellate_circle(
                lyon::math::point(center.0 as _, center.1 as _),
                r as f32,
                fill_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();

        //
        // **** Tessellate stroke ***************************
        //

        let mut stroke_tess = StrokeTessellator::new();
        let stroke_options = &StrokeOptions::tolerance(tolerance).with_line_width(line_width);

        let ctxt = VertexCtor::new(color);

        stroke_tess
            .tessellate_circle(
                lyon::math::point(center.0 as _, center.1 as _),
                r as f32,
                stroke_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    fn close(&mut self, _: DevDesc) {
        self.render().unwrap();
        pollster::block_on(self.write_png());
    }
}
