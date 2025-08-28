use std::sync::Arc;
use std::time::Instant;

use slotmap::SlotMap;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::CanvasKey;
use crate::context::Context;
use crate::render::GPUData;
use crate::state::AppState;

struct AppData<S> {
    ctx: Context,
    main_canvas: CanvasKey,
    state: S,
    last: Instant,
}

struct App<S> {
    data: Option<AppData<S>>,
}

impl<S: AppState> ApplicationHandler for App<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let gpu_data = pollster::block_on(GPUData::new(window.clone(), wgpu::Backends::GL));
        let mut ctx = Context {
            window: window.clone(),
            gpu_data,
            canvas_datas: SlotMap::default(),
            loaded_textures: SlotMap::default(),
            current_canvas: None,
            passes: vec![],
            vertices: vec![],
        };
        let main_canvas =
            ctx.create_canvas_inner(window.inner_size().width, window.inner_size().height, true);
        let state = S::setup(&mut ctx);

        self.data = Some(AppData {
            ctx,
            main_canvas,
            state,
            last: Instant::now(),
        })
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(data) = &mut self.data else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(to) => {
                data.ctx.gpu_data.resize(to.width, to.height);
                data.ctx
                    .resize_canvas(data.main_canvas, to.width, to.height);
            }
            WindowEvent::RedrawRequested => {
                let elapsed = data.last.elapsed().as_secs_f64();
                data.last = Instant::now();
                println!("fps: {}", 1.0 / elapsed);

                data.ctx.passes.clear();
                data.ctx.vertices.clear();

                data.ctx.draw_canvas(data.main_canvas, |canvas| {
                    data.state.draw(canvas);
                });

                data.ctx.render();

                data.ctx.window.request_redraw();
            }
            _ => (),
        }
    }
}

pub fn run_app<S: AppState>() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::<S> { data: None };
    event_loop.run_app(&mut app).unwrap();
}
