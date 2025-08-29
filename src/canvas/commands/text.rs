use std::f32::consts::PI;

use ahash::AHashMap;
use glam::{Vec2, vec2};
use itertools::Itertools;

use crate::{
    Canvas, Color,
    context::{BufferCacheKey, BufferCacheValue},
    render::text::{
        HashableAlign, HashableMetrics, find_closest_attrs, glyph::prepare_glyph,
        text_buffer_dimensions,
    },
};

#[must_use = "this command does nothing until you call `draw()`"]
pub struct TextBuilder<'a, 'r> {
    pub(crate) canvas: &'r mut Canvas<'a>,
    pub(crate) text: &'r str,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: Option<f32>,
    pub(crate) h: Option<f32>,
    pub(crate) size: f32,
    pub(crate) line_height: f32,
    pub(crate) family: cosmic_text::Family<'a>,
    pub(crate) weight: cosmic_text::Weight,
    pub(crate) style: cosmic_text::Style,
    pub(crate) stretch: cosmic_text::Stretch,
    pub(crate) align: cosmic_text::Align,
}
impl<'a, 'r> TextBuilder<'a, 'r> {
    #[inline]
    pub fn x(mut self, v: f32) -> Self {
        self.x = v;
        self
    }
    #[inline]
    pub fn y(mut self, v: f32) -> Self {
        self.y = v;
        self
    }
    #[inline]
    pub fn xy(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
    #[inline]
    pub fn w(mut self, v: f32) -> Self {
        self.w = Some(v);
        self
    }
    #[inline]
    pub fn h(mut self, v: f32) -> Self {
        self.h = Some(v);
        self
    }
    #[inline]
    pub fn wh(mut self, w: f32, h: f32) -> Self {
        self.w = Some(w);
        self.h = Some(h);
        self
    }
    #[inline]
    pub fn xywh(mut self, x: f32, y: f32, w: f32, h: f32) -> Self {
        self.x = x;
        self.y = y;
        self.w = Some(w);
        self.h = Some(h);
        self
    }
    #[inline]
    pub fn size(mut self, v: f32) -> Self {
        self.size = v;
        self
    }
    #[inline]
    pub fn line_height(mut self, v: f32) -> Self {
        self.line_height = v;
        self
    }
    #[inline]
    pub fn family(mut self, v: cosmic_text::Family<'a>) -> Self {
        self.family = v;
        self
    }
    #[inline]
    pub fn weight(mut self, v: cosmic_text::Weight) -> Self {
        self.weight = v;
        self
    }
    #[inline]
    pub fn style(mut self, v: cosmic_text::Style) -> Self {
        self.style = v;
        self
    }
    #[inline]
    pub fn stretch(mut self, v: cosmic_text::Stretch) -> Self {
        self.stretch = v;
        self
    }
    #[inline]
    pub fn align(mut self, v: cosmic_text::Align) -> Self {
        self.align = v;
        self
    }
    pub fn draw(self) {
        let v = get_and_shape_buffer(
            &mut self.canvas.ctx.inner.gpu_data.font_system,
            &mut self.canvas.ctx.inner.buffer_cache,
            self.w,
            self.h,
            self.text,
            self.size,
            self.line_height,
            self.family,
            self.weight,
            self.style,
            self.stretch,
            self.align,
        );

        for run in v.buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical = glyph.physical((0.0, 0.0), 1.0);

                if let Some([mut a, mut b, mut c, mut d]) = prepare_glyph(
                    physical,
                    run.line_y,
                    &mut self.canvas.ctx.inner.gpu_data,
                    self.canvas.fill_color.to_array(),
                    self.x,
                    self.y,
                ) {
                    a.pos = self
                        .canvas
                        .transform
                        .transform_point2(Vec2::from_array(a.pos))
                        .to_array();
                    b.pos = self
                        .canvas
                        .transform
                        .transform_point2(Vec2::from_array(b.pos))
                        .to_array();
                    c.pos = self
                        .canvas
                        .transform
                        .transform_point2(Vec2::from_array(c.pos))
                        .to_array();
                    d.pos = self
                        .canvas
                        .transform
                        .transform_point2(Vec2::from_array(d.pos))
                        .to_array();
                    self.canvas.ctx.inner.vertices.extend([a, b, c, a, c, d]);
                }
            }
        }
    }
    pub fn measure(&mut self) -> Vec2 {
        let v = get_and_shape_buffer(
            &mut self.canvas.ctx.inner.gpu_data.font_system,
            &mut self.canvas.ctx.inner.buffer_cache,
            self.w,
            self.h,
            self.text,
            self.size,
            self.line_height,
            self.family,
            self.weight,
            self.style,
            self.stretch,
            self.align,
        );

        Vec2::from_array(text_buffer_dimensions(&v.buffer))
    }
}

// no view types so gotta do this
pub fn get_and_shape_buffer<'a>(
    font_system: &mut cosmic_text::FontSystem,
    buffer_cache: &'a mut AHashMap<BufferCacheKey, BufferCacheValue>,
    w: Option<f32>,
    h: Option<f32>,
    text: &'a str,
    size: f32,
    line_height: f32,
    family: cosmic_text::Family<'a>,
    weight: cosmic_text::Weight,
    style: cosmic_text::Style,
    stretch: cosmic_text::Stretch,
    align: cosmic_text::Align,
) -> &'a mut BufferCacheValue {
    let metrics = cosmic_text::Metrics::relative(size, line_height);
    let attrs = cosmic_text::AttrsOwned::new(&find_closest_attrs(
        font_system.db(),
        family,
        weight,
        style,
        stretch,
    ));

    let v = buffer_cache
        .entry(BufferCacheKey {
            metrics: HashableMetrics(metrics),
            attrs: attrs.clone(),
            align: HashableAlign(align),
            text: text.into(),
        })
        .or_insert_with(|| {
            let mut buffer = cosmic_text::Buffer::new(font_system, metrics);

            buffer.set_rich_text(
                font_system,
                [(text, attrs.as_attrs())],
                &attrs.as_attrs(),
                cosmic_text::Shaping::Advanced,
                Some(align),
            );

            BufferCacheValue {
                buffer,
                in_use: true,
            }
        });
    v.in_use = true;

    v.buffer.set_size(font_system, w, h);
    v.buffer.shape_until_scroll(font_system, true);

    v
}
