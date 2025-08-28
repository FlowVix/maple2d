use crate::{canvas::Canvas, context::Context};

pub trait AppState {
    fn setup(ctx: &mut Context) -> Self;
    fn fixed_update(&mut self, ctx: &mut Context);
    fn draw(&mut self, canvas: &mut Canvas);
}
