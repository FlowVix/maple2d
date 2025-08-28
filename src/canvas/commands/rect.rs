use glam::{Vec2, vec2};
use i_overlay::mesh::{
    stroke::offset::StrokeOffset,
    style::{LineJoin, StrokeStyle},
};
use i_triangle::float::triangulatable::Triangulatable;

use crate::Canvas;

#[must_use = "this command does nothing until you call `draw()`"]
pub struct RectBuilder<'a, 'r> {
    pub(crate) canvas: &'r mut Canvas<'a>,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}
impl<'a, 'r> RectBuilder<'a, 'r> {
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
        self.w = v;
        self
    }
    #[inline]
    pub fn h(mut self, v: f32) -> Self {
        self.h = v;
        self
    }
    #[inline]
    pub fn wh(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }
    #[inline]
    pub fn xywh(mut self, x: f32, y: f32, w: f32, h: f32) -> Self {
        self.x = x;
        self.y = y;
        self.w = w;
        self.h = h;
        self
    }
    pub fn draw(self) {
        let a = vec2(self.x, self.y);
        let b = vec2(self.x + self.w, self.y);
        let c = vec2(self.x + self.w, self.y + self.h);
        let d = vec2(self.x, self.y + self.h);
        if self.canvas.draw_fill {
            self.canvas.raw_tri(
                a,
                b,
                c,
                self.canvas.fill_color,
                self.canvas.fill_color,
                self.canvas.fill_color,
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
            );
            self.canvas.raw_tri(
                a,
                c,
                d,
                self.canvas.fill_color,
                self.canvas.fill_color,
                self.canvas.fill_color,
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
            );
        }
        if self.canvas.draw_stroke {
            self.canvas.draw_stroke([a, b, c, d].into_iter());
        }
    }
}
