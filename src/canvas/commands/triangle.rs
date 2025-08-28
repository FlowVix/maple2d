use std::f32::consts::PI;

use glam::{Vec2, vec2};
use itertools::Itertools;

use crate::Canvas;

#[must_use = "this command does nothing until you call `draw()`"]
pub struct TriangleBuilder<'a, 'r> {
    pub(crate) canvas: &'r mut Canvas<'a>,
    pub(crate) x_a: f32,
    pub(crate) y_a: f32,
    pub(crate) x_b: f32,
    pub(crate) y_b: f32,
    pub(crate) x_c: f32,
    pub(crate) y_c: f32,
}
impl<'a, 'r> TriangleBuilder<'a, 'r> {
    #[inline]
    pub fn x_a(mut self, v: f32) -> Self {
        self.x_a = v;
        self
    }
    #[inline]
    pub fn y_a(mut self, v: f32) -> Self {
        self.y_a = v;
        self
    }
    #[inline]
    pub fn x_b(mut self, v: f32) -> Self {
        self.x_b = v;
        self
    }
    #[inline]
    pub fn y_b(mut self, v: f32) -> Self {
        self.y_b = v;
        self
    }
    #[inline]
    pub fn x_c(mut self, v: f32) -> Self {
        self.x_c = v;
        self
    }
    #[inline]
    pub fn y_c(mut self, v: f32) -> Self {
        self.y_c = v;
        self
    }
    #[inline]
    pub fn x_abc(mut self, a: f32, b: f32, c: f32) -> Self {
        self.x_a = a;
        self.x_b = b;
        self.x_c = c;
        self
    }
    #[inline]
    pub fn y_abc(mut self, a: f32, b: f32, c: f32) -> Self {
        self.y_a = a;
        self.y_b = b;
        self.y_c = c;
        self
    }
    #[inline]
    pub fn xy_abc(mut self, x_a: f32, y_a: f32, x_b: f32, y_b: f32, x_c: f32, y_c: f32) -> Self {
        self.x_a = x_a;
        self.y_a = y_a;
        self.x_b = x_b;
        self.y_b = y_b;
        self.x_c = x_c;
        self.y_c = y_c;
        self
    }
    pub fn draw(self) {
        let a = vec2(self.x_a, self.y_a);
        let b = vec2(self.x_b, self.y_b);
        let c = vec2(self.x_c, self.y_c);
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
        }
        if self.canvas.draw_stroke {
            self.canvas.draw_stroke([a, b, c].into_iter());
        }
    }
}
