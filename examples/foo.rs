use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Instant,
};

use glam::vec2;
use image::ImageReader;
use itertools::Itertools;
use maple2d::{AppState, CanvasKey, Color, TextureFilter, TextureKey, run_app};
use winit::{
    keyboard::{KeyCode, PhysicalKey},
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

    fn fixed_update(&mut self, ctx: &mut maple2d::Context) {
        if ctx.is_key_just_pressed(PhysicalKey::Code(KeyCode::KeyA)) {
            println!("bla fixed");
        }
    }

    fn draw(&mut self, canvas: &mut maple2d::Canvas) {
        if canvas
            .ctx()
            .is_key_just_pressed(PhysicalKey::Code(KeyCode::KeyA))
        {
            println!("bla render");
        }
    }
}

fn main() {
    run_app::<State>(10, Window::default_attributes());
}
