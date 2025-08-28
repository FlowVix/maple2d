use crate::{canvas::Canvas, context::Context};

pub trait AppState {
    fn setup(ctx: &mut Context) -> Self;
    fn fixed_update(&mut self, ctx: &mut Context);
    fn draw(&mut self, canvas: &mut Canvas);

    fn key_event(&mut self, event: winit::event::KeyEvent, ctx: &mut Context) {}
    fn mouse_input(&mut self, button: winit::event::MouseButton, pressed: bool, ctx: &mut Context) {
    }
}
