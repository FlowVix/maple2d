use std::f32::consts::PI;

use glam::{Vec2, vec2};
use i_overlay::mesh::{
    stroke::offset::StrokeOffset,
    style::{LineJoin, StrokeStyle},
};
use i_triangle::float::triangulatable::Triangulatable;
use itertools::Itertools;

use crate::Canvas;

#[must_use = "this command does nothing until you call `draw()`"]
pub struct EllipseBuilder<'a, 'r> {
    pub(crate) canvas: &'r mut Canvas<'a>,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}
impl<'a, 'r> EllipseBuilder<'a, 'r> {
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
        let point_count = self.canvas.arc_segments * 4;

        let mut points = (0..point_count).map(|v| {
            let angle = 2.0 * PI / point_count as f32 * v as f32;
            vec2(
                self.x + self.w * angle.cos() / 2.0,
                self.y + self.h * angle.sin() / 2.0,
            )
        });
        let points2 = points.clone();

        if self.canvas.draw_fill {
            let anchor = points.next().unwrap();
            for (b, c) in points.tuple_windows() {
                self.canvas.raw_tri(
                    anchor,
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
        }
        if self.canvas.draw_stroke {
            self.canvas.draw_stroke(points2);
        }
    }
}
