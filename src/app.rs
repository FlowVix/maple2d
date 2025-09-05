use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ahash::{AHashMap, AHashSet};
use glam::{Vec2, vec2};
use slotmap::SlotMap;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::CanvasKey;
use crate::context::{
    CanvasContext, Context, ContextRunMode, EitherKey, MouseWheelInfo, PressInfo,
};
use crate::render::GPUData;
use crate::state::AppState;

struct AppData<S> {
    ctx: Context,
    main_canvas: CanvasKey,
    state: S,
    last: Instant,
}

struct App<S> {
    attrs: Option<WindowAttributes>,
    data: Option<AppData<S>>,
    present_mode: wgpu::PresentMode,
}

enum CustomEvent {
    FixedUpdate,
}

impl<S: AppState> ApplicationHandler<CustomEvent> for App<S> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(self.attrs.take().unwrap())
                .unwrap(),
        );

        let mut gpu_data = pollster::block_on(GPUData::new(
            window.clone(),
            wgpu::Backends::all(),
            self.present_mode,
        ));
        gpu_data.resize(window.inner_size().width, window.inner_size().height);

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
            render_frame: 0,
            fixed_tick: 0,
            key_info: AHashMap::new(),
            mouse_button_info: AHashMap::new(),
            run_mode: ContextRunMode::None,
            mouse_wheel_info: MouseWheelInfo {
                delta: Vec2::ZERO,
                render_frame: None,
                fixed_tick: None,
            },
            temp_states: AHashMap::new(),
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

                data.ctx.reset_draw();

                data.ctx.run_mode = ContextRunMode::Render;
                CanvasContext {
                    inner: &mut data.ctx,
                }
                .draw_canvas(data.main_canvas, |canvas| {
                    data.state.draw(canvas);
                });
                data.ctx.run_mode = ContextRunMode::None;
                data.ctx.render_frame += 1;

                data.ctx.render();

                data.ctx.window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !event.repeat {
                    let k1 = EitherKey::Physical(event.physical_key);
                    let k2 = EitherKey::Logical(event.logical_key.clone());
                    if event.state.is_pressed() {
                        for k in [k1, k2] {
                            let info = data.ctx.key_info.entry(k).or_insert(PressInfo {
                                pressed: false,
                                pressed_render_frame: None,
                                released_render_frame: None,
                                pressed_fixed_tick: None,
                                released_fixed_tick: None,
                            });
                            info.pressed = true;
                            info.pressed_render_frame = Some(data.ctx.render_frame);
                            info.pressed_fixed_tick = Some(data.ctx.fixed_tick);
                        }
                    } else {
                        for k in [k1, k2] {
                            let info = data.ctx.key_info.entry(k).or_insert(PressInfo {
                                pressed: false,
                                pressed_render_frame: None,
                                released_render_frame: None,
                                pressed_fixed_tick: None,
                                released_fixed_tick: None,
                            });
                            info.pressed = false;
                            info.released_render_frame = Some(data.ctx.render_frame);
                            info.released_fixed_tick = Some(data.ctx.fixed_tick);
                        }
                    }
                }

                data.state.key_event(event, &mut data.ctx);
            }
            WindowEvent::CursorMoved { position, .. } => {
                data.ctx.mouse_pos = vec2(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state.is_pressed() {
                    let info = data
                        .ctx
                        .mouse_button_info
                        .entry(button)
                        .or_insert(PressInfo {
                            pressed: false,
                            pressed_render_frame: None,
                            released_render_frame: None,
                            pressed_fixed_tick: None,
                            released_fixed_tick: None,
                        });
                    info.pressed = true;
                    info.pressed_render_frame = Some(data.ctx.render_frame);
                    info.pressed_fixed_tick = Some(data.ctx.fixed_tick);
                } else {
                    let info = data
                        .ctx
                        .mouse_button_info
                        .entry(button)
                        .or_insert(PressInfo {
                            pressed: false,
                            pressed_render_frame: None,
                            released_render_frame: None,
                            pressed_fixed_tick: None,
                            released_fixed_tick: None,
                        });
                    info.pressed = false;
                    info.released_render_frame = Some(data.ctx.render_frame);
                    info.released_fixed_tick = Some(data.ctx.fixed_tick);
                }

                data.state
                    .mouse_input(button, state.is_pressed(), &mut data.ctx);
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    data.ctx.mouse_wheel_info.delta = vec2(x, y);
                    data.ctx.mouse_wheel_info.render_frame = Some(data.ctx.render_frame);
                    data.ctx.mouse_wheel_info.fixed_tick = Some(data.ctx.fixed_tick);
                }
                winit::event::MouseScrollDelta::PixelDelta(p) => {
                    data.ctx.mouse_wheel_info.delta = vec2(p.x as f32, p.y as f32);
                    data.ctx.mouse_wheel_info.render_frame = Some(data.ctx.render_frame);
                    data.ctx.mouse_wheel_info.fixed_tick = Some(data.ctx.fixed_tick);
                }
            },
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(data) = &mut self.data else {
            return;
        };
        // match event {
        //     DeviceEvent::
        // }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: CustomEvent) {
        let Some(data) = &mut self.data else {
            return;
        };
        match event {
            CustomEvent::FixedUpdate => {
                data.ctx.run_mode = ContextRunMode::Fixed;
                data.state.fixed_update(&mut data.ctx);
                data.ctx.run_mode = ContextRunMode::None;

                data.ctx.fixed_tick += 1;
            }
        }
    }
}

pub fn run_app<S: AppState>(
    fixed_update_rate: u32,
    window_attributes: WindowAttributes,
    present_mode: wgpu::PresentMode,
) {
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

    let mut app = App::<S> {
        data: None,
        attrs: Some(window_attributes),
        present_mode,
    };
    event_loop.run_app(&mut app).unwrap();
}
