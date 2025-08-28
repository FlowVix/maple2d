use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ahash::AHashMap;
use glam::vec2;
use slotmap::SlotMap;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::CanvasKey;
use crate::context::{CanvasContext, Context};
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

enum CustomEvent {
    FixedUpdate,
}

impl<S: AppState> ApplicationHandler<CustomEvent> for App<S> {
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
            mouse_pos: vec2(0.0, 0.0),
            current_canvas: None,
            passes: vec![],
            vertices: vec![],
            buffer_cache: AHashMap::new(),
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
        if data.state.window_event(&event, &mut data.ctx) {
            return;
        }
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

                data.ctx.reset();

                CanvasContext {
                    inner: &mut data.ctx,
                }
                .draw_canvas(data.main_canvas, |canvas| {
                    data.state.draw(canvas);
                });

                data.ctx.render();

                data.ctx.window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                data.state.key_event(event, &mut data.ctx);
            }
            WindowEvent::CursorMoved { position, .. } => {
                data.ctx.mouse_pos = vec2(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                data.state
                    .mouse_input(button, state.is_pressed(), &mut data.ctx);
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let Some(data) = &mut self.data else {
            return;
        };
        if data.state.device_event(&event, &mut data.ctx) {
            #[allow(clippy::needless_return)]
            return;
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: CustomEvent) {
        let Some(data) = &mut self.data else {
            return;
        };
        match event {
            CustomEvent::FixedUpdate => {
                data.state.fixed_update(&mut data.ctx);
            }
        }
    }
}

pub fn run_app<S: AppState>(fixed_update_rate: u32) {
    let event_loop = EventLoop::with_user_event().build().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let proxy = event_loop.create_proxy();
    std::thread::spawn(move || {
        loop {
            let Ok(_) = proxy.send_event(CustomEvent::FixedUpdate) else {
                break;
            };
            spin_sleep::sleep(Duration::from_secs_f64(1.0 / fixed_update_rate as f64));
        }
    });

    let mut app = App::<S> { data: None };
    event_loop.run_app(&mut app).unwrap();
}
