use std::mem::offset_of;

use wgpu::util::DeviceExt;

use crate::render::{
    shaders::{make_fragment_state, make_vertex_state, wgsl_common, wgsl_draw},
    texture::TextureBundle,
};

pub mod shaders;
pub mod texture;

pub const SAMPLE_COUNT: u32 = 4;

pub struct GPUData {
    pub(crate) surface: wgpu::Surface<'static>,

    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface_format: wgpu::TextureFormat,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,

    pub(crate) draw_pipeline: wgpu::RenderPipeline,

    pub(crate) dummy_texture_bind: wgsl_draw::globals::BindGroup1,
}

impl GPUData {
    pub async fn new(
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        backends: wgpu::Backends,
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
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

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
                depth_stencil: None,
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

        Self {
            surface,
            device,
            queue,
            surface_format,
            surface_config,
            draw_pipeline,
            dummy_texture_bind,
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
