#![deny(unused_must_use, unnameable_types)]
#![allow(clippy::too_many_arguments)]

mod app;
mod canvas;
mod context;
mod render;
mod state;

pub use app::run_app;
pub use canvas::{
    Canvas, CanvasKey,
    color::Color,
    commands::{
        ellipse::EllipseBuilder, rect::RectBuilder, text::TextBuilder, texture::TextureBuilder,
        triangle::TriangleBuilder,
    },
};
pub use context::{
    CanvasContext, Context, Key, TextureBytesLoadError, TexturePathLoadError,
    texture::{TextureFilter, TextureKey},
};
pub use state::AppState;

pub use cosmic_text;
pub use glam;
pub use winit;
