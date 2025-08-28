use std::f32::consts::PI;

use glam::{Vec2, vec2};
use itertools::Itertools;

use crate::{
    Canvas, Color,
    context::{BufferCacheKey, BufferCacheValue},
    render::text::{HashableMetrics, find_closest_attrs, glyph::prepare_glyph},
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
    pub fn draw(self) {
        let metrics = cosmic_text::Metrics::relative(self.size, self.line_height);
        let attrs = cosmic_text::AttrsOwned::new(&find_closest_attrs(
            self.canvas.ctx.gpu_data.font_system.db(),
            self.family,
            self.weight,
            self.style,
            self.stretch,
        ));

        let v = self
            .canvas
            .ctx
            .inner
            .buffer_cache
            .entry(BufferCacheKey {
                metrics: HashableMetrics(metrics),
                attrs: attrs.clone(),
                text: self.text.into(),
            })
            .or_insert_with(|| {
                let mut buffer = cosmic_text::Buffer::new(
                    &mut self.canvas.ctx.inner.gpu_data.font_system,
                    metrics,
                );

                buffer.set_text(
                    &mut self.canvas.ctx.inner.gpu_data.font_system,
                    self.text,
                    &attrs.as_attrs(),
                    cosmic_text::Shaping::Advanced,
                );

                BufferCacheValue {
                    buffer,
                    in_use: true,
                }
            });
        v.in_use = true;

        v.buffer.set_size(
            &mut self.canvas.ctx.inner.gpu_data.font_system,
            self.w,
            self.h,
        );
        v.buffer
            .shape_until_scroll(&mut self.canvas.ctx.inner.gpu_data.font_system, true);

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
}
