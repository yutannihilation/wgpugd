use std::{f32::consts::PI, os::raw::c_char};

use extendr_api::{
    graphics::{ClippingStrategy, DevDesc, DeviceDriver, R_GE_gcontext, TextMetric},
    prelude::*,
};

use lyon::path::Path;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{FillOptions, FillTessellator, FillVertex};
use lyon::tessellation::{StrokeOptions, StrokeTessellator, StrokeVertex};
use ttf_parser::GlyphId;

use glam::f32::Affine2;

use crate::text::FONTDB;

// TODO: determine tolerance nicely
pub(crate) const DEFAULT_TOLERANCE: f32 = lyon::tessellation::FillOptions::DEFAULT_TOLERANCE;

// TODO: why can't we use std::u32::MAX...?
const LAYER_FACTOR: f32 = 256.0 / std::u32::MAX as f32;

#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub count: u32,
}

impl DrawCommand {
    fn extend(&mut self, count: u32) {
        self.count += count;
    }
}

#[derive(Debug, Clone)]
pub enum WgpugdCommand {
    // Draw tessellated polygons.
    DrawPolygon(DrawCommand),
    // Draw shapes represented by an SDF.
    DrawSDF(DrawCommand),
    // Set clipping range.
    SetClipping,
}

struct VertexCtor {
    color: u32,
    transform: Affine2,
}

impl VertexCtor {
    fn new(color: i32, transform: Affine2) -> Self {
        Self {
            color: unsafe { std::mem::transmute(color) },
            transform,
        }
    }
}

impl StrokeVertexConstructor<crate::Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> crate::Vertex {
        let position_orig: mint::Point2<_> = vertex.position().into();
        let position = self.transform.transform_point2(position_orig.into());

        crate::Vertex {
            position: position.into(),
            color: self.color,
        }
    }
}

impl FillVertexConstructor<crate::Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: FillVertex) -> crate::Vertex {
        let position_orig: mint::Point2<_> = vertex.position().into();
        let position = self.transform.transform_point2(position_orig.into());

        crate::Vertex {
            position: position.into(),
            color: self.color,
        }
    }
}

impl crate::WgpuGraphicsDevice {
    fn tesselate_path_stroke(&mut self, path: &Path, stroke_options: &StrokeOptions, color: i32) {
        self.tesselate_path_stroke_with_transform(
            path,
            stroke_options,
            color,
            glam::Affine2::IDENTITY,
        );
    }

    fn tesselate_path_stroke_with_transform(
        &mut self,
        path: &Path,
        stroke_options: &StrokeOptions,
        color: i32,
        transform: glam::Affine2,
    ) {
        if color.is_na() {
            return;
        }

        let mut stroke_tess = StrokeTessellator::new();

        let ctxt = VertexCtor::new(color, transform);

        let count = stroke_tess
            .tessellate_path(
                path,
                stroke_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();

        match self.current_command {
            // If the previous command was the same, squash them into one draw
            // command.
            Some(WgpugdCommand::DrawPolygon(ref mut cmd)) => {
                cmd.extend(count.indices);
            }
            // If the previous command was different, push it to the command
            // queue (if exists) and create a new command.
            _ => {
                let prev = self
                    .current_command
                    .replace(WgpugdCommand::DrawPolygon(DrawCommand {
                        count: count.indices,
                    }));
                if let Some(prev_cmd) = prev {
                    self.command_queue.push(prev_cmd)
                }
            }
        }
    }

    fn tesselate_path_fill(&mut self, path: &Path, fill_options: &FillOptions, color: i32) {
        self.tesselate_path_fill_with_transform(path, fill_options, color, glam::Affine2::IDENTITY);
    }

    fn tesselate_path_fill_with_transform(
        &mut self,
        path: &Path,
        fill_options: &FillOptions,
        color: i32,
        transform: glam::Affine2,
    ) {
        if color.is_na() {
            return;
        }

        let mut fill_tess = FillTessellator::new();

        let ctxt = VertexCtor::new(color, transform);

        let count = fill_tess
            .tessellate_path(
                path,
                fill_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();

        match self.current_command {
            // If the previous command was the same, squash them into one draw
            // command.
            Some(WgpugdCommand::DrawPolygon(ref mut cmd)) => {
                cmd.extend(count.indices);
            }
            // If the previous command was different, push it to the command
            // queue (if exists) and create a new command.
            _ => {
                let prev = self
                    .current_command
                    .replace(WgpugdCommand::DrawPolygon(DrawCommand {
                        count: count.indices,
                    }));
                if let Some(prev_cmd) = prev {
                    self.command_queue.push(prev_cmd)
                }
            }
        }
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

        let ctxt = VertexCtor::new(color, glam::Affine2::IDENTITY);

        let count = stroke_tess
            .tessellate_rectangle(
                rect,
                stroke_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();

        match self.current_command {
            // If the previous command was the same, squash them into one draw
            // command.
            Some(WgpugdCommand::DrawPolygon(ref mut cmd)) => {
                cmd.extend(count.indices);
            }
            // If the previous command was different, push it to the command
            // queue (if exists) and create a new command.
            _ => {
                let prev = self
                    .current_command
                    .replace(WgpugdCommand::DrawPolygon(DrawCommand {
                        count: count.indices,
                    }));
                if let Some(prev_cmd) = prev {
                    self.command_queue.push(prev_cmd)
                }
            }
        }
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

        let ctxt = VertexCtor::new(color, glam::Affine2::IDENTITY);

        let count = fill_tess
            .tessellate_rectangle(
                rect,
                fill_options,
                &mut BuffersBuilder::new(&mut self.geometry, ctxt),
            )
            .unwrap();

        match self.current_command {
            // If the previous command was the same, squash them into one draw
            // command.
            Some(WgpugdCommand::DrawPolygon(ref mut cmd)) => {
                cmd.extend(count.indices);
            }
            // If the previous command was different, push it to the command
            // queue (if exists) and create a new command.
            _ => {
                let prev = self
                    .current_command
                    .replace(WgpugdCommand::DrawPolygon(DrawCommand {
                        count: count.indices,
                    }));
                if let Some(prev_cmd) = prev {
                    self.command_queue.push(prev_cmd)
                }
            }
        }
    }

    // This handles polygon(), polyline(), and line().
    #[allow(clippy::too_many_arguments)]
    fn polygon_inner<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        color: i32,
        fill: i32,
        line_width: f32,
        line_cap: lyon::tessellation::LineCap,
        line_join: lyon::tessellation::LineJoin,
        mitre_limit: f32,
        close: bool,
    ) {
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
        builder.end(close);

        let path = builder.build();

        //
        // **** Tessellate fill ***************************
        //

        let fill_options = &FillOptions::tolerance(DEFAULT_TOLERANCE);
        self.tesselate_path_fill(&path, fill_options, fill);

        //
        // **** Tessellate stroke ***************************
        //

        let stroke_options = &StrokeOptions::tolerance(DEFAULT_TOLERANCE)
            .with_line_width(line_width)
            .with_line_cap(line_cap)
            .with_line_join(line_join)
            .with_miter_limit(mitre_limit);
        self.tesselate_path_stroke(&path, stroke_options, color);
    }
}

// TODO: avoid magic numbers
fn translate_line_cap(lend: u32) -> lyon::tessellation::LineCap {
    match lend {
        1 => lyon::tessellation::LineCap::Round,
        2 => lyon::tessellation::LineCap::Butt,
        3 => lyon::tessellation::LineCap::Square,
        _ => lyon::tessellation::LineCap::Round,
    }
}

// TODO: avoid magic numbers
fn translate_line_join(ljoin: u32) -> lyon::tessellation::LineJoin {
    match ljoin {
        1 => lyon::tessellation::LineJoin::Round,
        2 => lyon::tessellation::LineJoin::Miter,
        3 => lyon::tessellation::LineJoin::Bevel,
        _ => lyon::tessellation::LineJoin::Round,
    }
}

// R Internals says:
//
// > lwd = 1 should correspond to a line width of 1/96 inch
//
// and 1 inch is 72 points.
fn translate_line_width(lwd: f64) -> f32 {
    lwd as f32 * 72. / 96.
}

impl DeviceDriver for crate::WgpuGraphicsDevice {
    const CLIPPING_STRATEGY: ClippingStrategy = ClippingStrategy::Device;

    fn line(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        let color = gc.col;
        let line_width = translate_line_width(gc.lwd);
        let line_cap = translate_line_cap(gc.lend);
        let line_join = translate_line_join(gc.ljoin);
        let mitre_limit = gc.lmitre as f32;

        self.polygon_inner(
            [from, to],
            color,
            i32::na(),
            line_width,
            line_cap,
            line_join,
            mitre_limit,
            false,
        );
    }

    fn polyline<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        let color = gc.col;
        let line_width = translate_line_width(gc.lwd);
        let line_cap = translate_line_cap(gc.lend);
        let line_join = translate_line_join(gc.ljoin);
        let mitre_limit = gc.lmitre as f32;

        self.polygon_inner(
            coords,
            color,
            i32::na(),
            line_width,
            line_cap,
            line_join,
            mitre_limit,
            false,
        );
    }

    fn polygon<T: IntoIterator<Item = (f64, f64)>>(
        &mut self,
        coords: T,
        gc: R_GE_gcontext,
        _: DevDesc,
    ) {
        let color = gc.col;
        let fill = gc.fill;
        let line_width = translate_line_width(gc.lwd);
        let line_cap = translate_line_cap(gc.lend);
        let line_join = translate_line_join(gc.ljoin);
        let mitre_limit = gc.lmitre as f32;

        self.polygon_inner(
            coords,
            color,
            fill,
            line_width,
            line_cap,
            line_join,
            mitre_limit,
            false,
        );
    }

    fn circle(&mut self, center: (f64, f64), r: f64, gc: R_GE_gcontext, _: DevDesc) {
        let color = gc.col;
        let fill = gc.fill;
        let line_width = translate_line_width(gc.lwd);

        self.sdf_instances.push(crate::SDFInstance {
            center: [center.0 as _, center.1 as _],
            radius: r as _,
            stroke_width: line_width,
            fill_color: unsafe { std::mem::transmute(fill) },
            stroke_color: unsafe { std::mem::transmute(color) },
        });

        match self.current_command {
            // If the previous command was the same, squash them into one draw
            // command.
            Some(WgpugdCommand::DrawSDF(ref mut cmd)) => {
                cmd.extend(1);
            }
            // If the previous command was different, push it to the command
            // queue (if exists) and create a new command.
            _ => {
                let prev = self
                    .current_command
                    .replace(WgpugdCommand::DrawSDF(DrawCommand { count: 1 }));
                if let Some(prev_cmd) = prev {
                    self.command_queue.push(prev_cmd)
                }
            }
        }
    }

    fn rect(&mut self, from: (f64, f64), to: (f64, f64), gc: R_GE_gcontext, _: DevDesc) {
        let color = gc.col;
        let fill = gc.fill;
        let line_width = translate_line_width(gc.lwd);
        let line_cap = translate_line_cap(gc.lend);
        let line_join = translate_line_join(gc.ljoin);
        let mitre_limit = gc.lmitre as f32;

        let x = from.0.min(to.0) as f32;
        let y = from.1.min(to.1) as f32;
        let w = (to.0 - from.0).abs() as f32;
        let h = (to.1 - from.1).abs() as f32;

        //
        // **** Tessellate fill ***************************
        //

        let fill_options = &FillOptions::tolerance(DEFAULT_TOLERANCE);
        self.tesselate_rect_fill(&lyon::math::rect(x, y, w, h), fill_options, fill);

        //
        // **** Tessellate stroke ***************************
        //

        let stroke_options = &StrokeOptions::tolerance(DEFAULT_TOLERANCE)
            .with_line_width(line_width)
            .with_line_cap(line_cap)
            .with_line_join(line_join)
            .with_miter_limit(mitre_limit);

        self.tesselate_rect_stroke(&lyon::math::rect(x, y, w, h), stroke_options, color);
    }

    // Wildly assumes 1 font has 1pt of width, and 10% of horizontal margins on
    // top and bottom. This should be properly done by querying to a font
    // database (e.g. https://github.com/RazrFalcon/fontdb).
    fn char_metric(&mut self, c: char, gc: R_GE_gcontext, _: DevDesc) -> TextMetric {
        let fontfamily =
            unsafe { std::ffi::CStr::from_ptr(&gc.fontfamily as *const c_char) }.to_string_lossy();

        let id = match crate::text::FONTDB.query(&fontfamily, gc.fontface) {
            Some(id) => id,
            None => {
                reprintln!("[WARN] No fallback font found, aborting");
                return TextMetric {
                    ascent: 0.0,
                    descent: 0.0,
                    width: 0.0,
                };
            }
        };

        FONTDB
            .with_face_data(id, |font_data, face_index| {
                let font = ttf_parser::Face::from_slice(font_data, face_index).unwrap();
                let scale = gc.cex * gc.ps / font.height() as f64;

                let glyph_id = font.glyph_index(c).unwrap_or(GlyphId(0));

                match font.glyph_bounding_box(glyph_id) {
                    Some(bbox) => TextMetric {
                        ascent: bbox.y_max as f64 * scale,
                        descent: bbox.y_min as f64 * scale,
                        width: font
                            .glyph_hor_advance(glyph_id)
                            .unwrap_or(bbox.width() as _) as f64
                            * scale,
                    },
                    // If the glyph info is not available, use font info
                    _ => TextMetric {
                        ascent: font.ascender() as f64 * scale,
                        descent: font.descender() as f64 * scale,
                        width: font.height() as f64 * scale,
                    },
                }
            })
            .unwrap()
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
        let fill = gc.col;

        let fontfamily =
            unsafe { std::ffi::CStr::from_ptr(&gc.fontfamily as *const c_char) }.to_string_lossy();

        let id = match crate::text::FONTDB.query(&fontfamily, gc.fontface) {
            Some(id) => id,
            None => {
                reprintln!("[WARN] No fallback font found, aborting");
                return;
            }
        };

        FONTDB.with_face_data(id, |font_data, face_index| {
            let font = ttf_parser::Face::from_slice(font_data, face_index).unwrap();

            let facetables = font.tables();

            // Deviding by `height` is to normalize the font coordinates to 1.
            // Then, multiply by `cex` (size of the font in device specific
            // unit) and `px` (pointsize, should be 12) to convert to the value
            // in points. Since the range of the values actually matters on
            // tessellation, we need to multiply before tessellation.
            let scale = (gc.cex * gc.ps) as f32 / font.height() as f32;

            let mut builder = crate::text::LyonOutlineBuilder::new(scale);

            let mut prev_glyph: Option<GlyphId> = None;
            for c in text.chars() {
                // Skip control characters. Note that it seems linebreaks are
                // handled on R's side, so we don't need to care about multiline
                // cases.
                if c.is_control() {
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

            // First, move the origin depending on `hadj`
            let transform_hadj =
                glam::Affine2::from_translation(glam::vec2(builder.offset_x() * -hadj as f32, 0.0));

            // Second, rotate and translate to the position
            let transform = glam::Affine2::from_angle_translation(
                angle as f32 / 360.0 * 2. * PI,
                glam::vec2(pos.0 as _, pos.1 as _),
            ) * transform_hadj;

            let path = builder.build();

            //
            // **** Tessellate fill ***************************
            //

            let fill_options = &FillOptions::tolerance(DEFAULT_TOLERANCE);
            self.tesselate_path_fill_with_transform(&path, fill_options, fill, transform);
        });
    }

    fn clip(&mut self, from: (f64, f64), to: (f64, f64), _: DevDesc) {
        // TODO

        // // If the clipping contains the whole layer, skip it
        // if from.0 <= 0. && from.1 <= 0. && to.0 >= self.width as _ && to.1 >= self.height as _ {
        //     self.current_clipping_id = -1;
        // } else {
        //     let clipping_id = self.layer_clippings.add_clipping(from, to);

        //     if clipping_id < crate::MAX_CLIPPINGS {
        //         self.current_clipping_id = clipping_id as _;
        //     } else {
        //         reprintln!("[WARN] too many clippings! {from:?}, {to:?} is ignored");
        //     }
        // }
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
