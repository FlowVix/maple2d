use std::f32::consts::PI;

use glam::{Vec2, vec2};
use itertools::Itertools;

use crate::{Canvas, Color};

#[must_use = "this command does nothing until you call `draw()`"]
pub struct TextureBuilder<'a, 'r> {
    pub(crate) canvas: &'r mut Canvas<'a>,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: Option<f32>,
    pub(crate) h: Option<f32>,
    pub(crate) centered: bool,
    pub(crate) region: Option<(Vec2, Vec2)>,
    pub(crate) tint: bool,
}
impl<'a, 'r> TextureBuilder<'a, 'r> {
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
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
    #[inline]
    pub fn region(mut self, pos: Vec2, size: Vec2) -> Self {
        self.region = Some((pos, size));
        self
    }
    #[inline]
    pub fn tint(mut self) -> Self {
        self.tint = true;
        self
    }
    pub fn draw(self) {
        let [tex_width, tex_height] = self
            .canvas
            .current_texture()
            .map(|v| self.canvas.ctx.texture_dimensions(v).to_array())
            .unwrap_or([2, 2]);

        let width = match self.w {
            Some(v) => v,
            None => match self.region {
                Some((_, v)) => v.x,
                None => tex_width as f32,
            },
        };
        let height = match self.h {
            Some(v) => v,
            None => match self.region {
                Some((_, v)) => v.y,
                None => tex_height as f32,
            },
        };
        let halfsize = vec2(width, height) / 2.0;
        let mut a = vec2(self.x, self.y);
        let mut b = vec2(self.x + width, self.y);
        let mut c = vec2(self.x + width, self.y + height);
        let mut d = vec2(self.x, self.y + height);
        if self.centered {
            a -= halfsize;
            b -= halfsize;
            c -= halfsize;
            d -= halfsize;
        }

        let [uv_left, uv_right, uv_top, uv_bottom] = match self.region {
            Some((pos, size)) => [pos.x, pos.x + size.x, pos.y, pos.y + size.y],
            None => [0.0, tex_width as f32, 0.0, tex_height as f32],
        };
        let uv_a = vec2(uv_left, uv_bottom);
        let uv_b = vec2(uv_right, uv_bottom);
        let uv_c = vec2(uv_right, uv_top);
        let uv_d = vec2(uv_left, uv_top);

        let color = if self.tint {
            self.canvas.fill_color
        } else {
            Color::rgb(1.0, 1.0, 1.0)
        };

        self.canvas.raw_tri(
            a,
            b,
            c,
            color,
            color,
            color,
            uv_a,
            uv_b,
            uv_c,
            vec2(-1.0, 0.0),
            vec2(-1.0, 0.0),
            vec2(-1.0, 0.0),
        );
        self.canvas.raw_tri(
            a,
            c,
            d,
            color,
            color,
            color,
            uv_a,
            uv_c,
            uv_d,
            vec2(-1.0, 0.0),
            vec2(-1.0, 0.0),
            vec2(-1.0, 0.0),
        );
    }
}
