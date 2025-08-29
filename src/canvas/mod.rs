use std::f32::consts::PI;

use glam::{Affine2, Mat2, Vec2, vec2};

use itertools::Itertools;

use slotmap::new_key_type;

use crate::{
    canvas::{
        color::Color,
        commands::{
            ellipse::EllipseBuilder, rect::RectBuilder, text::TextBuilder, texture::TextureBuilder,
            triangle::TriangleBuilder,
        },
    },
    context::{CanvasContext, Context, DrawCall, DrawCallType, texture::TextureKey},
    render::shaders::wgsl_common,
};

pub mod color;
pub mod commands;

new_key_type! {
    pub struct CanvasKey;
}

pub struct Canvas<'a> {
    pub(crate) key: CanvasKey,
    pub(crate) ctx: CanvasContext<'a>,

    pub(crate) current_texture: Option<TextureKey>,

    pub fill_color: Color,
    pub stroke_color: Color,
    pub stroke_weight: f32,

    pub draw_fill: bool,
    pub draw_stroke: bool,

    pub arc_segments: u16,

    pub transform: Affine2,
}
impl<'a> Canvas<'a> {
    pub(crate) fn new(key: CanvasKey, ctx: CanvasContext<'a>) -> Self {
        Self {
            key,
            ctx,
            current_texture: None,
            fill_color: Color::rgb(0.25, 0.25, 0.25),
            stroke_color: Color::rgb(0.75, 0.75, 0.75),
            stroke_weight: 2.0,
            draw_fill: true,
            draw_stroke: true,
            arc_segments: 8,
            transform: Affine2::IDENTITY,
        }
    }
    pub fn ctx(&mut self) -> &mut CanvasContext<'a> {
        &mut self.ctx
    }
    pub fn key(&self) -> CanvasKey {
        self.key
    }

    pub fn set_texture(&mut self, tex: TextureKey) {
        if Some(tex) != self.current_texture() {
            self.ctx
                .inner
                .passes
                .last_mut()
                .unwrap()
                .calls
                .push(DrawCall {
                    start_vertex: self.ctx.inner.vertices.len() as u32,
                    typ: DrawCallType::Draw {
                        set_texture: Some(tex),
                    },
                });
            self.current_texture = Some(tex);
        }
    }
    pub fn current_texture(&mut self) -> Option<TextureKey> {
        self.current_texture
    }

    pub fn set_transform(&mut self, transform: Affine2) {
        self.transform = transform;
    }
    pub fn add_transform(&mut self, transform: Affine2) {
        self.transform *= transform;
    }
    pub fn translate(&mut self, x: f32, y: f32) {
        self.add_transform(Affine2::from_translation(vec2(x, y)));
    }
    pub fn rotate(&mut self, angle: f32) {
        self.add_transform(Affine2::from_angle(angle));
    }
    pub fn rotate_xy(&mut self, x: f32, y: f32) {
        self.add_transform(Affine2::from_mat2(Mat2::from_cols(
            vec2(x.cos(), x.sin()),
            vec2(-y.sin(), y.cos()),
        )));
    }
    pub fn scale(&mut self, x: f32, y: f32) {
        self.add_transform(Affine2::from_scale(vec2(x, y)));
    }
    pub fn skew(&mut self, x: f32, y: f32) {
        self.add_transform(Affine2::from_mat2(Mat2::from_cols(
            vec2(1.0, y),
            vec2(x, 1.0),
        )));
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
        self.ctx.inner.vertices.extend([
            wgsl_common::structs::VertexInput::new(
                self.transform.transform_point2(a).to_array(),
                color_a.to_array(),
                uv_a.to_array(),
                text_uv_a.to_array(),
            ),
            wgsl_common::structs::VertexInput::new(
                self.transform.transform_point2(b).to_array(),
                color_b.to_array(),
                uv_b.to_array(),
                text_uv_b.to_array(),
            ),
            wgsl_common::structs::VertexInput::new(
                self.transform.transform_point2(c).to_array(),
                color_c.to_array(),
                uv_c.to_array(),
                text_uv_c.to_array(),
            ),
        ]);
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
    pub fn text<'r>(&'r mut self, string: &'r str) -> TextBuilder<'a, 'r> {
        TextBuilder {
            canvas: self,
            text: string,
            x: 0.0,
            y: 0.0,
            w: None,
            h: None,
            size: 16.0,
            line_height: 1.3,
            family: cosmic_text::Family::SansSerif,
            weight: cosmic_text::Weight::NORMAL,
            style: cosmic_text::Style::Normal,
            stretch: cosmic_text::Stretch::Normal,
            align: cosmic_text::Align::Left,
        }
    }
}

// #[bon::bon]
// impl<'a> Canvas<'a> {
//     #[builder]
//     pub fn rect(&mut self, x: f32, y: f32) {}
// }
