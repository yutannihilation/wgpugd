use extendr_api::prelude::*;
use once_cell::sync::Lazy;

pub(crate) struct FontDBWrapper {
    db: fontdb::Database,
    fallback_glyph_id: Option<fontdb::ID>,
}

impl FontDBWrapper {
    pub(crate) fn query(&self, fontfamily: &str, fontface: i32) -> Option<fontdb::ID> {
        // TODO: Can I do this more nicely?
        let (weight, style) = match fontface {
            1 => (fontdb::Weight::NORMAL, fontdb::Style::Normal), // Plain
            2 => (fontdb::Weight::BOLD, fontdb::Style::Normal),   // Bold
            3 => (fontdb::Weight::NORMAL, fontdb::Style::Italic), // Italic
            4 => (fontdb::Weight::BOLD, fontdb::Style::Italic),   // BoldItalic
            // Symbolic or unknown
            _ => {
                reprintln!("[WARN] Unsupported fontface");
                (fontdb::Weight::NORMAL, fontdb::Style::Normal)
            }
        };

        if let Some(id) = self.db.query(&fontdb::Query {
            families: &[fontdb::Family::Name(fontfamily)],
            weight,
            stretch: fontdb::Stretch::Normal,
            style,
        }) {
            Some(id)
        } else {
            // TODO: This warning is shown too many times, so disabled temporarily...
            //
            // reprintln!(
            //     "[WARN] Cannot find the specified font family, falling back to the default font..."
            // );
            self.fallback_glyph_id
        }
    }

    pub(crate) fn with_face_data<P, T>(&self, id: fontdb::ID, p: P) -> Option<T>
    where
        P: FnOnce(&[u8], u32) -> T,
    {
        self.db.with_face_data(id, p)
    }
}

pub(crate) static FONTDB: Lazy<FontDBWrapper> = Lazy::new(|| {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let fallback_glyph_id = db.query(&fontdb::Query {
        families: &[fontdb::Family::SansSerif],
        ..Default::default()
    });

    FontDBWrapper {
        db,
        fallback_glyph_id,
    }
});

pub(crate) struct LyonOutlineBuilder {
    pub(crate) builder: lyon::path::path::Builder,
    // multiply by this to scale the position into the range of [0, 1].
    scale_factor: f32,

    offset_x: f32,
}

impl LyonOutlineBuilder {
    pub(crate) fn new(scale: f32) -> Self {
        Self {
            builder: lyon::path::Path::builder(),
            scale_factor: scale,
            offset_x: 0.0,
        }
    }

    pub(crate) fn build(self) -> lyon::path::Path {
        self.builder.build()
    }

    fn point(&self, x: f32, y: f32) -> lyon::math::Point {
        lyon::math::point(x * self.scale_factor + self.offset_x, y * self.scale_factor)
    }

    pub(crate) fn add_offset_x(&mut self, offset: f32) {
        self.offset_x += offset * self.scale_factor;
    }

    pub(crate) fn offset_x(&self) -> f32 {
        self.offset_x
    }
}

impl ttf_parser::OutlineBuilder for LyonOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.begin(self.point(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(self.point(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let ctrl = self.point(x1, y1);
        let to = self.point(x, y);
        self.builder.quadratic_bezier_to(ctrl, to);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let ctrl1 = self.point(x1, y1);
        let ctrl2 = self.point(x2, y2);
        let to = self.point(x, y);
        self.builder.cubic_bezier_to(ctrl1, ctrl2, to);
    }

    fn close(&mut self) {
        self.builder.close();
    }
}

pub(crate) fn find_kerning(
    facetables: &ttf_parser::FaceTables,
    left: ttf_parser::GlyphId,
    right: ttf_parser::GlyphId,
) -> i16 {
    let kern_table = if let Some(kern_table) = facetables.kern {
        kern_table
    } else {
        return 0;
    };

    for st in kern_table.subtables {
        if !st.horizontal {
            continue;
        }

        if let Some(kern) = st.glyphs_kerning(left, right) {
            return kern;
        }
    }

    0
}
