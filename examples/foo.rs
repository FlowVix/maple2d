use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Instant,
};

use glam::vec2;
use image::ImageReader;
use itertools::Itertools;
use maple2d::{AppState, CanvasKey, Color, TextureFilter, TextureKey, run_app};
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
        canvas.fill_color = Color::rgb(1.0, 1.0, 1.0);
        let mut t = canvas
            .text("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.")
            .w(100.0);
        println!("{}", t.measure());
        t.draw();
    }
}

fn main() {
    run_app::<State>(60, Window::default_attributes());
}
