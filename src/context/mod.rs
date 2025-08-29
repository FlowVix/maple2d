pub mod texture;

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    io::{self, Cursor},
    mem::offset_of,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
};

use ahash::{AHashMap, AHashSet};
use glam::{UVec2, Vec2, uvec2};
use image::ImageReader;
use slotmap::{SlotMap, new_key_type};
use wgpu::util::DeviceExt;
use winit::{event::MouseButton, keyboard::SmolStr, window::Window};

use crate::{
    canvas::{Canvas, CanvasKey},
    context::texture::{LoadedTexture, TextureFilter, TextureKey, TextureMap},
    render::{
        GPUData, SAMPLE_COUNT,
        shaders::{wgsl_common, wgsl_draw},
        text::{HashableAlign, HashableMetrics},
        texture::TextureBundle,
    },
};

pub struct Context {
    pub(crate) window: Arc<Window>,
    pub(crate) gpu_data: GPUData,
    pub(crate) canvas_datas: SlotMap<CanvasKey, CanvasData>,
    pub(crate) loaded_textures: TextureMap,

    // maintenance
    pub(crate) render_frame: u64,
    pub(crate) fixed_tick: u64,
    pub(crate) run_mode: ContextRunMode,

    // drawing related
    pub(crate) current_canvas: Option<CanvasKey>,
    pub(crate) passes: Vec<RenderPass>,
    pub(crate) vertices: Vec<wgsl_common::structs::VertexInput>,
    pub(crate) buffer_cache: AHashMap<BufferCacheKey, BufferCacheValue>,

    // input related
    pub(crate) mouse_pos: Vec2,
    pub(crate) key_info: AHashMap<EitherKey, PressInfo>,
    pub(crate) mouse_button_info: AHashMap<MouseButton, PressInfo>,
    pub(crate) mouse_wheel_info: MouseWheelInfo,

    // state related
    pub(crate) temp_states: AHashMap<(SmolStr, TypeId), Box<dyn Any>>,
}
pub struct CanvasContext<'a> {
    pub(crate) inner: &'a mut Context,
}
impl<'a> Deref for CanvasContext<'a> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
impl<'a> DerefMut for CanvasContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

pub struct CanvasData {
    pub(crate) multisample_descriptor: wgpu::TextureDescriptor<'static>,
    pub(crate) output_multisample_view: wgpu::TextureView,
    pub(crate) depth_stencil_descriptor: wgpu::TextureDescriptor<'static>,
    pub(crate) depth_stencil_view: wgpu::TextureView,

    pub(crate) output_texture: Option<TextureBundle>,

    pub(crate) globals_buffer: wgpu::Buffer,
    pub(crate) bind_group_0: wgsl_common::globals::BindGroup0,
}

pub struct RenderPass {
    pub(crate) target_canvas: CanvasKey,
    pub(crate) calls: Vec<DrawCall>,
}

pub enum DrawCallType {
    Draw {
        // pub set_blend_mode: Option<BlendMode>,
        set_texture: Option<TextureKey>,
        reference: u32,
        end_clip_reference: Option<u32>,
    },
    ClipStart {
        reference: u32,
    },
}
pub struct DrawCall {
    pub(crate) start_vertex: u32,
    pub(crate) typ: DrawCallType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferCacheKey {
    pub(crate) metrics: HashableMetrics,
    pub(crate) attrs: cosmic_text::AttrsOwned,
    pub(crate) align: HashableAlign,
    pub(crate) text: String,
}
#[derive(Debug, Clone)]
pub struct BufferCacheValue {
    pub(crate) buffer: cosmic_text::Buffer,
    pub(crate) in_use: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EitherKey {
    Physical(winit::keyboard::PhysicalKey),
    Logical(winit::keyboard::Key),
}
impl From<winit::keyboard::PhysicalKey> for EitherKey {
    fn from(value: winit::keyboard::PhysicalKey) -> Self {
        Self::Physical(value)
    }
}
impl<S: Into<SmolStr>> From<winit::keyboard::Key<S>> for EitherKey {
    fn from(value: winit::keyboard::Key<S>) -> Self {
        Self::Logical(match value {
            winit::keyboard::Key::Named(named_key) => winit::keyboard::Key::Named(named_key),
            winit::keyboard::Key::Character(s) => winit::keyboard::Key::Character(s.into()),
            winit::keyboard::Key::Unidentified(native_key) => {
                winit::keyboard::Key::Unidentified(native_key)
            }
            winit::keyboard::Key::Dead(c) => winit::keyboard::Key::Dead(c),
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PressInfo {
    pub(crate) pressed: bool,
    pub(crate) pressed_render_frame: Option<u64>,
    pub(crate) released_render_frame: Option<u64>,
    pub(crate) pressed_fixed_tick: Option<u64>,
    pub(crate) released_fixed_tick: Option<u64>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseWheelInfo {
    pub(crate) delta: Vec2,
    pub(crate) render_frame: Option<u64>,
    pub(crate) fixed_tick: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextRunMode {
    None,
    Render,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexturePathLoadError {
    FileNotFound,
    DecodeError,
}
#[derive(Debug)]
pub enum TextureBytesLoadError {
    IoError(io::Error),
    DecodeError,
}

impl Context {
    pub(crate) fn reset_draw(&mut self) {
        self.passes.clear();
        self.vertices.clear();
        self.vertices.extend(
            [
                [0.0, 0.0],
                [self.gpu_data.surface_config.width as f32 * 2.0, 0.0],
                [0.0, self.gpu_data.surface_config.height as f32 * 2.0],
            ]
            .map(|pos| wgsl_common::structs::VertexInput::new(pos, [1.0; 4], [-1.0; 2], [-1.0; 2])),
        );

        self.gpu_data.mask_atlas.clear_in_use();
        self.gpu_data.color_atlas.clear_in_use();

        self.buffer_cache.retain(|_, v| v.in_use);
        for v in self.buffer_cache.values_mut() {
            v.in_use = false;
        }
    }
    pub(crate) fn create_canvas_inner(
        &mut self,
        width: u32,
        height: u32,
        screen: bool,
    ) -> CanvasKey {
        let globals_buffer =
            self.gpu_data
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("globals_buffer"),
                    contents: bytemuck::cast_slice(&[wgsl_common::structs::CanvasGlobals::new([
                        width as f32,
                        height as f32,
                    ])]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let bind_group_0 = wgsl_common::globals::BindGroup0::from_bindings(
            &self.gpu_data.device,
            wgsl_common::globals::BindGroup0Entries::new(
                wgsl_common::globals::BindGroup0EntriesEntriesParams {
                    GLOBALS: globals_buffer.as_entire_buffer_binding(),
                },
            ),
        );

        let multisample_descriptor = wgpu::TextureDescriptor {
            label: Some("multisample_descriptor"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: self.gpu_data.surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let output_multisample_view = self
            .gpu_data
            .device
            .create_texture(&multisample_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth_stencil_descriptor = wgpu::TextureDescriptor {
            label: Some("Depth Stencil Descriptor"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Depth24PlusStencil8],
        };
        let depth_stencil_view = self
            .gpu_data
            .device
            .create_texture(&depth_stencil_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.canvas_datas.insert(CanvasData {
            multisample_descriptor,
            output_multisample_view,
            depth_stencil_descriptor,
            depth_stencil_view,
            output_texture: (!screen).then(|| {
                TextureBundle::blank(
                    &self.gpu_data.device,
                    width,
                    height,
                    self.gpu_data.surface_format,
                    wgpu::FilterMode::Linear,
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    1,
                    1,
                )
            }),
            globals_buffer,
            bind_group_0,
        })
    }
    pub fn create_canvas(&mut self, width: u32, height: u32) -> CanvasKey {
        self.create_canvas_inner(width, height, false)
    }
    pub fn delete_canvas(&mut self, key: CanvasKey) -> bool {
        self.canvas_datas.remove(key).is_some()
    }
    pub fn resize_canvas(&mut self, key: CanvasKey, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.canvas_datas[key].multisample_descriptor = wgpu::TextureDescriptor {
                label: Some("multisample_descriptor"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: self.gpu_data.surface_config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            };

            self.canvas_datas[key].output_multisample_view = self
                .gpu_data
                .device
                .create_texture(&self.canvas_datas[key].multisample_descriptor)
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.canvas_datas[key].depth_stencil_descriptor = wgpu::TextureDescriptor {
                label: Some("Depth Stencil Descriptor"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[wgpu::TextureFormat::Depth24PlusStencil8],
            };
            self.canvas_datas[key].depth_stencil_view = self
                .gpu_data
                .device
                .create_texture(&self.canvas_datas[key].depth_stencil_descriptor)
                .create_view(&wgpu::TextureViewDescriptor::default());

            if let Some(output) = &mut self.canvas_datas[key].output_texture {
                *output = TextureBundle::blank(
                    &self.gpu_data.device,
                    width,
                    height,
                    self.gpu_data.surface_format,
                    wgpu::FilterMode::Linear,
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    1,
                    1,
                );
            }

            self.gpu_data.queue.write_buffer(
                &self.canvas_datas[key].globals_buffer,
                offset_of!(wgsl_common::structs::CanvasGlobals, screen_size) as u64,
                bytemuck::bytes_of(&[width as f32, height as f32]),
            );
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn mouse_pos(&self) -> Vec2 {
        self.mouse_pos
    }
    pub fn font_system(&mut self) -> &mut cosmic_text::FontSystem {
        &mut self.gpu_data.font_system
    }
    pub fn render_frame(&self) -> u64 {
        self.render_frame
    }
    pub fn fixed_tick(&self) -> u64 {
        self.fixed_tick
    }

    pub fn is_key_pressed(&self, key: impl Into<EitherKey>) -> bool {
        self.key_info
            .get(&key.into())
            .map(|v| v.pressed)
            .unwrap_or(false)
    }
    pub fn is_key_released(&self, key: impl Into<EitherKey>) -> bool {
        self.key_info
            .get(&key.into())
            .map(|v| !v.pressed)
            .unwrap_or(true)
    }
    pub fn is_key_just_pressed(&self, key: impl Into<EitherKey>) -> bool {
        self.key_info
            .get(&key.into())
            .map(|v| {
                self.run_mode == ContextRunMode::Render
                    && Some(self.render_frame) == v.pressed_render_frame
                    || self.run_mode == ContextRunMode::Fixed
                        && Some(self.fixed_tick) == v.pressed_fixed_tick
            })
            .unwrap_or(false)
    }
    pub fn is_key_just_released(&self, key: impl Into<EitherKey>) -> bool {
        self.key_info
            .get(&key.into())
            .map(|v| {
                self.run_mode == ContextRunMode::Render
                    && Some(self.render_frame) == v.released_render_frame
                    || self.run_mode == ContextRunMode::Fixed
                        && Some(self.fixed_tick) == v.released_fixed_tick
            })
            .unwrap_or(false)
    }
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_button_info
            .get(&button)
            .map(|v| v.pressed)
            .unwrap_or(false)
    }
    pub fn is_mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_button_info
            .get(&button)
            .map(|v| !v.pressed)
            .unwrap_or(true)
    }
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_button_info
            .get(&button)
            .map(|v| {
                self.run_mode == ContextRunMode::Render
                    && Some(self.render_frame) == v.pressed_render_frame
                    || self.run_mode == ContextRunMode::Fixed
                        && Some(self.fixed_tick) == v.pressed_fixed_tick
            })
            .unwrap_or(false)
    }
    pub fn is_mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse_button_info
            .get(&button)
            .map(|v| {
                self.run_mode == ContextRunMode::Render
                    && Some(self.render_frame) == v.released_render_frame
                    || self.run_mode == ContextRunMode::Fixed
                        && Some(self.fixed_tick) == v.released_fixed_tick
            })
            .unwrap_or(false)
    }
    pub fn mouse_wheel_delta(&self) -> Vec2 {
        if self.run_mode == ContextRunMode::Render
            && Some(self.render_frame) == self.mouse_wheel_info.render_frame
            || self.run_mode == ContextRunMode::Fixed
                && Some(self.fixed_tick) == self.mouse_wheel_info.fixed_tick
        {
            self.mouse_wheel_info.delta
        } else {
            Vec2::ZERO
        }
    }

    pub fn load_texture_rgba(
        &mut self,
        rgba: &[u8],
        width: u32,
        height: u32,
        filter: TextureFilter,
    ) -> TextureKey {
        let texture = TextureBundle::from_rgba(
            &self.gpu_data.device,
            &self.gpu_data.queue,
            rgba,
            width,
            height,
            match filter {
                TextureFilter::Linear => wgpu::FilterMode::Linear,
                TextureFilter::Nearest => wgpu::FilterMode::Nearest,
            },
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let bind_group = wgsl_draw::globals::BindGroup1::from_bindings(
            &self.gpu_data.device,
            wgsl_draw::globals::BindGroup1Entries::new(
                wgsl_draw::globals::BindGroup1EntriesEntriesParams {
                    TEXTURE_T: &texture.view,
                    TEXTURE_S: &texture.sampler,
                },
            ),
        );
        self.loaded_textures.insert(LoadedTexture {
            texture,
            bind_group,
        })
    }
    pub fn load_texture_path<P: AsRef<Path>>(
        &mut self,
        path: P,
        filter: TextureFilter,
    ) -> Result<TextureKey, TexturePathLoadError> {
        let img = ImageReader::open(path)
            .map_err(|_| TexturePathLoadError::FileNotFound)?
            .decode()
            .map_err(|_| TexturePathLoadError::DecodeError)?;
        Ok(self.load_texture_rgba(&img.to_rgba8(), img.width(), img.height(), filter))
    }
    pub fn load_texture_bytes(
        &mut self,
        bytes: &[u8],
        filter: TextureFilter,
    ) -> Result<TextureKey, TextureBytesLoadError> {
        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(TextureBytesLoadError::IoError)?
            .decode()
            .map_err(|_| TextureBytesLoadError::DecodeError)?;
        Ok(self.load_texture_rgba(&img.to_rgba8(), img.width(), img.height(), filter))
    }
    pub fn remove_texture(&mut self, texture: TextureKey) {
        self.loaded_textures.remove(texture);
    }
    pub fn texture_dimensions(&self, texture: TextureKey) -> UVec2 {
        let t = &self.loaded_textures[texture].texture.texture;
        uvec2(t.width(), t.height())
    }

    pub fn state<T: 'static, F: FnOnce() -> T>(
        &mut self,
        key: impl Into<SmolStr>,
        default: F,
    ) -> &mut T {
        let key = key.into();
        let t = TypeId::of::<T>();
        self.temp_states
            .entry((key, t))
            .or_insert_with(|| Box::new(default()))
            .downcast_mut()
            .unwrap()
    }

    pub(crate) fn render(&self) {
        let Ok(output) = self.gpu_data.surface.get_current_texture() else {
            return;
        };
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu_data
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let vertex_buffer =
            self.gpu_data
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&self.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        if !self.vertices.is_empty() {
            let num_vertices = self.vertices.len() as u32;
            for (idx, pass) in self.passes.iter().enumerate() {
                let render_pass_start_vertex = pass.calls[0].start_vertex;
                let render_pass_end_vertex = self
                    .passes
                    .get(idx + 1)
                    .map(|p| p.calls[0].start_vertex)
                    .unwrap_or(num_vertices);

                if render_pass_end_vertex - render_pass_start_vertex == 0 {
                    continue;
                }

                {
                    let pass_desc = wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &self.canvas_datas[pass.target_canvas].output_multisample_view,
                            resolve_target: Some(
                                if let Some(tex) =
                                    &self.canvas_datas[pass.target_canvas].output_texture
                                {
                                    &tex.view
                                } else {
                                    &output_view
                                },
                            ),
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.canvas_datas[pass.target_canvas].depth_stencil_view,
                            depth_ops: None,
                            stencil_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                        }),
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    };
                    let mut render_pass = encoder.begin_render_pass(&pass_desc);

                    render_pass.set_bind_group(
                        0,
                        self.canvas_datas[pass.target_canvas]
                            .bind_group_0
                            .get_bind_group(),
                        &[],
                    );
                    render_pass.set_bind_group(
                        1,
                        self.gpu_data.dummy_texture_bind.get_bind_group(),
                        &[],
                    );
                    render_pass.set_bind_group(
                        2,
                        self.gpu_data.text_atlas_bind_group.get_bind_group(),
                        &[],
                    );

                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

                    for (idx, call) in pass.calls.iter().enumerate() {
                        let call_end_vertex = pass
                            .calls
                            .get(idx + 1)
                            .map(|c| c.start_vertex)
                            .unwrap_or(render_pass_end_vertex);

                        match call.typ {
                            DrawCallType::Draw {
                                set_texture,
                                reference,
                                end_clip_reference,
                            } => {
                                if let Some(end_reference) = end_clip_reference {
                                    render_pass.set_pipeline(&self.gpu_data.end_clip_pipeline);
                                    render_pass.set_stencil_reference(end_reference);
                                    render_pass.draw(0..3, 0..1);
                                }
                                // if let Some(mode) = call.set_blend_mode {
                                //     render_pass.set_pipeline(match mode {
                                //         BlendMode::Normal => &self.normal_pipeline,
                                //         BlendMode::Additive => &self.additive_pipeline,
                                //     });
                                // }
                                if let Some(tex) = set_texture {
                                    render_pass.set_bind_group(
                                        1,
                                        self.loaded_textures[tex].bind_group.get_bind_group(),
                                        &[],
                                    );
                                }
                                render_pass.set_pipeline(&self.gpu_data.draw_pipeline);
                                render_pass.set_stencil_reference(reference);
                                render_pass.draw(call.start_vertex..call_end_vertex, 0..1);
                            }
                            DrawCallType::ClipStart { reference } => {
                                render_pass.set_pipeline(&self.gpu_data.start_clip_pipeline);
                                render_pass.set_stencil_reference(reference);
                                render_pass.draw(call.start_vertex..call_end_vertex, 0..1);
                            }
                        }
                    }
                }
            }
        }

        self.gpu_data.queue.submit([encoder.finish()]);
        output.present();
    }
}
impl<'a> CanvasContext<'a> {
    pub fn draw_canvas<F>(&mut self, key: CanvasKey, cb: F)
    where
        F: FnOnce(&mut Canvas),
    {
        let prev = self.inner.current_canvas.map(|canvas| {
            (canvas, {
                let DrawCallType::Draw { reference, .. } =
                    self.inner.passes.last().unwrap().calls.last().unwrap().typ
                else {
                    panic!("started sub-canvas draw during clip draw")
                };
                reference
            })
        });

        self.inner.current_canvas = Some(key);
        self.inner.passes.push(RenderPass {
            target_canvas: key,
            calls: vec![DrawCall {
                start_vertex: self.inner.vertices.len() as u32,
                typ: DrawCallType::Draw {
                    set_texture: None,
                    reference: 0,
                    end_clip_reference: None,
                },
            }],
        });

        let mut canvas = Canvas::new(key, CanvasContext { inner: self.inner });

        cb(&mut canvas);

        if let Some((prev_canvas, prev_reference)) = prev {
            self.inner.passes.push(RenderPass {
                target_canvas: prev_canvas,
                calls: vec![DrawCall {
                    start_vertex: self.inner.vertices.len() as u32,
                    typ: DrawCallType::Draw {
                        set_texture: None,
                        reference: prev_reference,
                        end_clip_reference: None,
                    },
                }],
            });
        }
        self.current_canvas = prev.map(|v| v.0);
    }
}
