use std::os::raw::c_char;

use extendr_api::{
    graphics::{ClippingStrategy, DevDesc, DeviceDriver, R_GE_gcontext},
    prelude::*,
};

use lyon::path::Path;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{FillOptions, FillTessellator, FillVertex};
use lyon::tessellation::{StrokeOptions, StrokeTessellator, StrokeVertex};
use ttf_parser::GlyphId;

use crate::text::FONTDB;

struct VertexCtor {
    color: u32,
    layer: u32,
}

impl VertexCtor {
    fn new(color: i32, layer: u32) -> Self {
        Self {
            color: unsafe { std::mem::transmute(color) },
            layer,
        }
    }
}

impl StrokeVertexConstructor<crate::Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> crate::Vertex {
        let pos = vertex.position();
        crate::Vertex {
            position: pos.into(),
            color: self.color,
            layer: self.layer,
        }
    }
}

impl FillVertexConstructor<crate::Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: FillVertex) -> crate::Vertex {
        let pos = vertex.position();
        crate::Vertex {
            position: pos.into(),
            color: self.color,
            layer: self.layer,
        }
    }
}

impl crate::WgpuGraphicsDevice {
    fn tesselate_path_stroke(&mut self, path: &Path, stroke_options: &StrokeOptions, color: i32) {
        if color.is_na() {
            return;
        }

        let mut stroke_tess = StrokeTessellator::new();

        let ctxt = VertexCtor::new(color, self.current_layer as _);

        stroke_tess
            .tessellate_path(
                path,
                stroke_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    fn tesselate_path_fill(&mut self, path: &Path, fill_options: &FillOptions, color: i32) {
        if color.is_na() {
            return;
        }

        let mut fill_tess = FillTessellator::new();

        let ctxt = VertexCtor::new(color, self.current_layer as _);

        fill_tess
            .tessellate_path(
                path,
                fill_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    fn tesselate_circle_stroke(
        &mut self,
        center: lyon::math::Point,
        r: f32,
        stroke_options: &StrokeOptions,
        color: i32,
    ) {
        if color.is_na() {
            return;
        }

        let mut stroke_tess = StrokeTessellator::new();

        let ctxt = VertexCtor::new(color, self.current_layer as _);

        stroke_tess
            .tessellate_circle(
                center,
                r,
                stroke_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    fn tesselate_circle_fill(
        &mut self,
        center: lyon::math::Point,
        r: f32,
        fill_options: &FillOptions,
        color: i32,
    ) {
        if color.is_na() {
            return;
        }

        let mut fill_tess = FillTessellator::new();

        let ctxt = VertexCtor::new(color, self.current_layer as _);

        fill_tess
            .tessellate_circle(
                center,
                r,
                fill_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    fn tesselate_rect_stroke(
        &mut self,
        rect: &lyon::math::Rect,
        stroke_options: &StrokeOptions,
        color: i32,
    ) {
        if color.is_na() {
            return;
        }

        let mut stroke_tess = StrokeTessellator::new();

        let ctxt = VertexCtor::new(color, self.current_layer as _);

        stroke_tess
            .tessellate_rectangle(
                rect,
                stroke_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    fn tesselate_rect_fill(
        &mut self,
        rect: &lyon::math::Rect,
        fill_options: &FillOptions,
        color: i32,
    ) {
        if color.is_na() {
            return;
        }

        let mut fill_tess = FillTessellator::new();

        let ctxt = VertexCtor::new(color, self.current_layer as _);

        fill_tess
            .tessellate_rectangle(
                rect,
                fill_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();
    }

    // This handles polygon(), polyline(), and line().
    fn polygon_inner<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        color: i32,
        fill: i32,
        line_width: f32,
    ) {
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

        let fill_options = &FillOptions::tolerance(tolerance);
        self.tesselate_path_fill(&path, fill_options, fill);

        //
        // **** Tessellate stroke ***************************
        //

        let stroke_options = &StrokeOptions::tolerance(tolerance).with_line_width(line_width);
        self.tesselate_path_stroke(&path, stroke_options, color);
    }
}

impl DeviceDriver for crate::WgpuGraphicsDevice {
    const CLIPPING_STRATEGY: ClippingStrategy = ClippingStrategy::Device;

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        let color = gc.col;
        let line_width = gc.lwd as f32;

        self.polygon_inner([from, to], color, i32::na(), line_width);
    }

    fn polyline<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        let color = gc.col;
        let line_width = gc.lwd as f32;

        self.polygon_inner(coords, color, i32::na(), line_width);
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

        self.polygon_inner(coords, color, fill, line_width);
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

        let fill_options = &FillOptions::tolerance(tolerance);
        self.tesselate_circle_fill(
            lyon::math::point(center.0 as _, center.1 as _),
            r as f32,
            fill_options,
            fill,
        );

        //
        // **** Tessellate stroke ***************************
        //

        let stroke_options = &StrokeOptions::tolerance(tolerance).with_line_width(line_width);

        self.tesselate_circle_stroke(
            lyon::math::point(center.0 as _, center.1 as _),
            r as f32,
            stroke_options,
            color,
        );
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        let color = gc.col;
        let fill = gc.fill;
        let line_width = gc.lwd as f32;
        // TODO: determine tolerance nicely
        let tolerance = 0.01;

        let x = from.0.min(to.0) as f32;
        let y = from.1.min(to.1) as f32;
        let w = (to.0 - from.0).abs() as f32;
        let h = (to.1 - from.1).abs() as f32;

        //
        // **** Tessellate fill ***************************
        //

        let fill_options = &FillOptions::tolerance(tolerance);
        self.tesselate_rect_fill(&lyon::math::rect(x, y, w, h), fill_options, fill);

        //
        // **** Tessellate stroke ***************************
        //

        let stroke_options = &StrokeOptions::tolerance(tolerance).with_line_width(line_width);

        self.tesselate_rect_stroke(&lyon::math::rect(x, y, w, h), stroke_options, color);
    }

    fn text(
        &mut self,
        pos: (f64, f64),
        text: &str,
        angle: f64,
        hadj: f64,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        eprintln!("[DEBUG] pos: {pos:?}");

        let fill = gc.col;
        // TODO: determine tolerance nicely
        let tolerance = 0.01;

        let fontfamily =
            unsafe { std::ffi::CStr::from_ptr(&gc.fontfamily as *const c_char) }.to_string_lossy();

        // TODO: Can I do this more nicely?
        let (weight, style) = match gc.fontface {
            // Plain
            1 => (fontdb::Weight::NORMAL, fontdb::Style::Normal),
            // Bold
            2 => (fontdb::Weight::BOLD, fontdb::Style::Normal),
            // Italic
            3 => (fontdb::Weight::NORMAL, fontdb::Style::Italic),
            // BoldItalic
            4 => (fontdb::Weight::BOLD, fontdb::Style::Italic),
            // Symbolic or unknown
            _ => {
                eprintln!("[WARN] Unsupported fontface");
                (fontdb::Weight::NORMAL, fontdb::Style::Normal)
            }
        };

        let query = fontdb::Query {
            families: &[fontdb::Family::Name(&fontfamily), fontdb::Family::Serif],
            weight,
            stretch: fontdb::Stretch::Normal,
            style,
        };

        let id = crate::text::FONTDB.query(&query);

        // TODO: fallback to a different font
        if id.is_none() {
            eprintln!("[WARN] font not found: {fontfamily}");
            return;
        }

        FONTDB.with_face_data(id.unwrap(), |font_data, face_index| {
            let font = ttf_parser::Face::from_slice(font_data, face_index).unwrap();

            let facetables = font.tables();

            let height = font.height() as f32;
            let line_height = height + font.line_gap() as f32;

            // TODO: what size is the correct size?
            let scale_factor = 12. / height;

            let mut builder =
                crate::text::LyonOutlineBuilder::new(scale_factor, pos.0 as _, pos.1 as _);

            let mut prev_glyph: Option<GlyphId> = None;
            for c in text.chars() {
                // Skip control characters
                if c.is_control() {
                    // If the character is a line break, move to the next line
                    if c == '\n' {
                        builder.add_offset_y(-line_height);
                        builder.set_offset_x(pos.0 as _);
                    }
                    prev_glyph = None;
                    continue;
                }
                // Even when we cannot find glyph_id, fill it with 0.
                let cur_glyph = font.glyph_index(c).unwrap_or(GlyphId(0));

                if let Some(prev_glyph) = prev_glyph {
                    builder.add_offset_x(crate::text::find_kerning(
                        facetables, prev_glyph, cur_glyph,
                    ) as _);
                }

                font.outline_glyph(cur_glyph, &mut builder);

                if let Some(ha) = font.glyph_hor_advance(cur_glyph) {
                    builder.add_offset_x(ha as _);
                }

                prev_glyph = Some(cur_glyph);
            }

            let path = builder.build();

            //
            // **** Tessellate fill ***************************
            //

            let fill_options = &FillOptions::tolerance(tolerance);
            self.tesselate_path_fill(&path, fill_options, fill);
        });
    }

    fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {
        // If the clipping contains the whole layer, skip it
        if from.0 <= 0. && from.1 <= 0. && to.0 >= self.width as _ && to.1 >= self.height as _ {
            self.current_layer = 0;
        } else {
            let layer_id = self.layer_clippings.add_clipping(from, to);

            if layer_id < crate::MAX_LAYERS {
                self.current_layer = layer_id;
            } else {
                reprintln!("[WARN] too many clippings! {from:?}, {to:?} is ignored");
            }
        }
    }

    fn new_page(&mut self, _: R_GE_gcontext, _: DevDesc) {
        // newPage() is called soon after the device is open, but there's
        // nothing to render. So, skip rendering at first.
        if self.cur_page != 0 {
            self.render().unwrap();
            pollster::block_on(self.write_png());
        }

        self.cur_page += 1;
    }

    fn close(&mut self, _: DevDesc) {
        self.render().unwrap();
        pollster::block_on(self.write_png());
    }
}
