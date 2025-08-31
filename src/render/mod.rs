use std::mem::offset_of;

use wgpu::util::DeviceExt;

use crate::render::{
    shaders::{make_fragment_state, make_vertex_state, wgsl_common, wgsl_draw, wgsl_stencil},
    text::{
        atlas::{GlyphAtlas, create_atlases_bind_group},
        glyph::ContentType,
    },
    texture::TextureBundle,
};

pub mod shaders;
pub mod text;
pub mod texture;

pub const SAMPLE_COUNT: u32 = 4;

pub struct GPUData {
    pub(crate) surface: wgpu::Surface<'static>,

    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface_format: wgpu::TextureFormat,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,

    pub(crate) start_clip_pipeline: wgpu::RenderPipeline,
    pub(crate) end_clip_pipeline: wgpu::RenderPipeline,
    pub(crate) draw_pipeline: wgpu::RenderPipeline,

    pub(crate) dummy_texture_bind: wgsl_draw::globals::BindGroup1,

    pub(crate) font_system: cosmic_text::FontSystem,
    pub(crate) swash_cache: cosmic_text::SwashCache,

    pub(crate) mask_atlas: GlyphAtlas,
    pub(crate) color_atlas: GlyphAtlas,
    pub(crate) text_atlas_bind_group: wgsl_draw::globals::BindGroup2,
}

impl GPUData {
    pub async fn new(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        backends: wgpu::Backends,
        present_mode: wgpu::PresentMode,
    ) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(target).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device_descriptor"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    ..Default::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 10,
            height: 10,
            present_mode: present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let start_clip_pipeline = {
            let module = wgsl_stencil::create_shader_module(&device);

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("start_clip_pipeline"),
                layout: Some(&wgsl_stencil::create_pipeline_layout(&device)),
                vertex: make_vertex_state(
                    &module,
                    &wgsl_stencil::entries::vertex_entry_vs_main(wgpu::VertexStepMode::Vertex),
                ),
                fragment: Some(make_fragment_state(
                    &module,
                    &wgsl_stencil::entries::fragment_entry_fs_main(&[Some(
                        wgpu::ColorTargetState {
                            format: surface_config.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::empty(),
                        },
                    )]),
                )),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState {
                        front: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::Equal,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                            pass_op: wgpu::StencilOperation::IncrementClamp,
                        },
                        back: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::Equal,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                            pass_op: wgpu::StencilOperation::IncrementClamp,
                        },
                        read_mask: 0xff,
                        write_mask: 0xff,
                    },
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: SAMPLE_COUNT,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            })
        };
        let end_clip_pipeline = {
            let module = wgsl_stencil::create_shader_module(&device);

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("start_clip_pipeline"),
                layout: Some(&wgsl_stencil::create_pipeline_layout(&device)),
                vertex: make_vertex_state(
                    &module,
                    &wgsl_stencil::entries::vertex_entry_vs_main(wgpu::VertexStepMode::Vertex),
                ),
                fragment: Some(make_fragment_state(
                    &module,
                    &wgsl_stencil::entries::fragment_entry_fs_main(&[Some(
                        wgpu::ColorTargetState {
                            format: surface_config.format,
                            blend: None,
                            write_mask: wgpu::ColorWrites::empty(),
                        },
                    )]),
                )),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState {
                        front: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::Equal,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                            pass_op: wgpu::StencilOperation::DecrementClamp,
                        },
                        back: wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::Equal,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                            pass_op: wgpu::StencilOperation::DecrementClamp,
                        },
                        read_mask: 0xff,
                        write_mask: 0xff,
                    },
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: SAMPLE_COUNT,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            })
        };

        let draw_depth_stencil = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState {
                front: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                back: wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::Keep,
                },
                read_mask: 0xff,
                write_mask: 0x00,
            },
            bias: Default::default(),
        });

        let draw_pipeline = {
            let module = wgsl_draw::create_shader_module(&device);

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("draw_pipeline"),
                layout: Some(&wgsl_draw::create_pipeline_layout(&device)),
                vertex: make_vertex_state(
                    &module,
                    &wgsl_draw::entries::vertex_entry_vs_main(wgpu::VertexStepMode::Vertex),
                ),
                fragment: Some(make_fragment_state(
                    &module,
                    &wgsl_draw::entries::fragment_entry_fs_main(&[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })]),
                )),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: draw_depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: SAMPLE_COUNT,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            })
        };

        let dummy_bundle = TextureBundle::blank(
            &device,
            2,
            2,
            surface_format,
            wgpu::FilterMode::Linear,
            wgpu::TextureUsages::TEXTURE_BINDING,
            1,
            1,
        );
        let dummy_texture_bind = wgsl_draw::globals::BindGroup1::from_bindings(
            &device,
            wgsl_draw::globals::BindGroup1Entries::new(
                wgsl_draw::globals::BindGroup1EntriesEntriesParams {
                    TEXTURE_T: &dummy_bundle.view,
                    TEXTURE_S: &dummy_bundle.sampler,
                },
            ),
        );

        let mask_atlas = GlyphAtlas::new(&device, ContentType::Mask);
        let color_atlas = GlyphAtlas::new(&device, ContentType::Color);
        let text_atlas_bind_group = create_atlases_bind_group(&device, &mask_atlas, &color_atlas);

        Self {
            surface,
            device,
            queue,
            surface_format,
            surface_config,
            start_clip_pipeline,
            end_clip_pipeline,
            draw_pipeline,
            dummy_texture_bind,
            mask_atlas,
            color_atlas,
            text_atlas_bind_group,
            font_system: cosmic_text::FontSystem::new(),
            swash_cache: cosmic_text::SwashCache::new(),
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            // tracing::span!("RenderState_resize");

            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);

            // self.queue.write_buffer(
            //     &self.globals_buffer,
            //     offset_of!(wgsl_common::structs::Globals, screen_size) as u64,
            //     bytemuck::bytes_of(&[
            //         self.surface_config.width as f32,
            //         self.surface_config.height as f32,
            //     ]),
            // );
        }
    }
}
