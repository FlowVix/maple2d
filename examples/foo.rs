use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Instant,
};

use glam::vec2;
use image::ImageReader;
use itertools::Itertools;
use maple2d::{AppState, BlendMode, CanvasKey, Color, TextureFilter, TextureKey, run_app};
use winit::{
    event::MouseButton,
    keyboard::{Key, KeyCode, PhysicalKey},
    window::Window,
};

struct State {
    v: f32,
    tex: TextureKey,
}

impl AppState for State {
    fn setup(ctx: &mut maple2d::Context) -> Self {
        let tex = ctx
            .load_texture_path("examples/Untitled.png", TextureFilter::Linear)
            .unwrap();
        Self { v: 0.0, tex }
    }

    fn fixed_update(&mut self, ctx: &mut maple2d::Context) {}

    fn draw(&mut self, canvas: &mut maple2d::Canvas) {
        canvas.fill_color = Color::rgb(0.0, 0.0, 0.0);
        canvas.clear();

        canvas.set_blend_mode(BlendMode::Additive);

        canvas.draw_stroke = false;
        canvas.fill_color = Color::rgb(1.0, 0.0, 0.0);
        canvas.rect().xy(0.0, 0.0).wh(100.0, 100.0).draw();
        canvas.fill_color = Color::rgb(0.0, 1.0, 0.0);
        canvas.rect().xy(50.0, 50.0).wh(100.0, 100.0).draw();
    }
}

fn main() {
    run_app::<State>(
        60,
        Window::default_attributes(),
        wgpu::PresentMode::AutoVsync,
        wgpu::Backends::all(),
    );
}
