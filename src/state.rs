use crate::{canvas::Canvas, context::Context};

pub trait AppState {
    fn setup(ctx: &mut Context) -> Self;
    fn draw(&mut self, canvas: &mut Canvas);
}
