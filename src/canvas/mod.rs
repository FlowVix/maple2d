use std::f32::consts::PI;

use glam::{Affine2, Vec2, vec2};

use itertools::Itertools;

use slotmap::new_key_type;

use crate::{
    canvas::{
        color::Color,
        commands::{
            ellipse::EllipseBuilder, rect::RectBuilder, texture::TextureBuilder,
            triangle::TriangleBuilder,
        },
    },
    context::{Context, DrawCall, texture::TextureKey},
    render::shaders::wgsl_common,
};

pub mod color;
pub mod commands;

new_key_type! {
    pub struct CanvasKey;
}

pub struct Canvas<'a> {
    pub(crate) key: CanvasKey,
    pub(crate) ctx: &'a mut Context,

    pub fill_color: Color,
    pub stroke_color: Color,
    pub stroke_weight: f32,

    pub draw_fill: bool,
    pub draw_stroke: bool,

    pub arc_segments: u16,

    pub transform: Affine2,
}
impl<'a> Canvas<'a> {
    pub(crate) fn new(key: CanvasKey, ctx: &'a mut Context) -> Self {
        Self {
            key,
            ctx,
            fill_color: Color::rgb(0.25, 0.25, 0.25),
            stroke_color: Color::rgb(0.75, 0.75, 0.75),
            stroke_weight: 2.0,
            draw_fill: true,
            draw_stroke: true,
            arc_segments: 8,
            transform: Affine2::IDENTITY,
        }
    }
    pub fn ctx(&mut self) -> &mut Context {
        self.ctx
    }
    pub fn key(&self) -> CanvasKey {
        self.key
    }

    pub fn set_texture(&mut self, tex: TextureKey) {
        self.ctx.passes.last_mut().unwrap().calls.push(DrawCall {
            start_vertex: self.ctx.vertices.len() as u32,
            set_texture: Some(tex),
        });
    }
    pub fn current_texture(&mut self) -> Option<TextureKey> {
        self.ctx
            .passes
            .last()
            .unwrap()
            .calls
            .last()
            .unwrap()
            .set_texture
    }

    pub(crate) fn push_vertex(&mut self, v: wgsl_common::structs::VertexInput) {
        self.ctx.vertices.push(v);
    }
    pub fn raw_tri(
        &mut self,
        a: Vec2,
        b: Vec2,
        c: Vec2,
        color_a: Color,
        color_b: Color,
        color_c: Color,
        uv_a: Vec2,
        uv_b: Vec2,
        uv_c: Vec2,
        text_uv_a: Vec2,
        text_uv_b: Vec2,
        text_uv_c: Vec2,
    ) {
        self.push_vertex(wgsl_common::structs::VertexInput::new(
            a.to_array(),
            color_a.to_array(),
            uv_a.to_array(),
            text_uv_a.to_array(),
        ));
        self.push_vertex(wgsl_common::structs::VertexInput::new(
            b.to_array(),
            color_b.to_array(),
            uv_b.to_array(),
            text_uv_b.to_array(),
        ));
        self.push_vertex(wgsl_common::structs::VertexInput::new(
            c.to_array(),
            color_c.to_array(),
            uv_c.to_array(),
            text_uv_c.to_array(),
        ));
    }
    pub(crate) fn draw_stroke(&mut self, points: impl ExactSizeIterator<Item = Vec2> + Clone) {
        let n_verts = points.len() as u32 * 2;

        let (stroke_color, stroke_weight) = (self.stroke_color, self.stroke_weight);

        let points = points
            .circular_tuple_windows()
            .flat_map(|(prev, current, next)| {
                let angle_prev = (prev.y - current.y).atan2(prev.x - current.x);
                let angle_next = (next.y - current.y).atan2(next.x - current.x);
                let angle = (angle_prev + angle_next) / 2.0;

                let angle_diff = angle_next - angle_prev;
                let scale = 1.0 / ((PI - angle_diff) / 2.0).cos();

                let cos = angle.cos();
                let sin = angle.sin();

                [
                    vec2(
                        current[0] - cos * stroke_weight / 2.0 * scale,
                        current[1] - sin * stroke_weight / 2.0 * scale,
                    ),
                    vec2(
                        current[0] + cos * stroke_weight / 2.0 * scale,
                        current[1] + sin * stroke_weight / 2.0 * scale,
                    ),
                ]
            })
            .collect_vec();

        for [a, b, c] in (0..n_verts).map(|i| [i, (i + 1) % n_verts, (i + 2) % n_verts]) {
            self.raw_tri(
                points[a as usize],
                points[b as usize],
                points[c as usize],
                stroke_color,
                stroke_color,
                stroke_color,
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(-1.0, 0.0),
            );
        }
    }
    pub fn rect<'r>(&'r mut self) -> RectBuilder<'a, 'r> {
        RectBuilder {
            canvas: self,
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        }
    }
    pub fn ellipse<'r>(&'r mut self) -> EllipseBuilder<'a, 'r> {
        EllipseBuilder {
            canvas: self,
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        }
    }
    pub fn triangle<'r>(&'r mut self) -> TriangleBuilder<'a, 'r> {
        TriangleBuilder {
            canvas: self,
            x_a: 0.0,
            y_a: 0.0,
            x_b: 0.0,
            y_b: 0.0,
            x_c: 0.0,
            y_c: 0.0,
        }
    }
    pub fn texture<'r>(&'r mut self) -> TextureBuilder<'a, 'r> {
        TextureBuilder {
            canvas: self,
            x: 0.0,
            y: 0.0,
            w: None,
            h: None,
            centered: false,
            region: None,
            tint: false,
        }
    }
}

// #[bon::bon]
// impl<'a> Canvas<'a> {
//     #[builder]
//     pub fn rect(&mut self, x: f32, y: f32) {}
// }
