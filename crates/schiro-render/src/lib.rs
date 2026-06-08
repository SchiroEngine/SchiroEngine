pub mod camera;
pub mod device;
pub mod egui_renderer;
pub mod gizmo;
pub mod graph;
pub mod light;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod surface;
pub mod texture;
pub mod viewport;

use std::sync::Arc;

use camera::CameraUniform;
pub use gizmo::GizmoMeshes;
pub use mesh::Mesh;
use mesh::GpuMesh;
pub use surface::create_surface;
pub use viewport::ViewportRenderer;

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: (u32, u32),
    pub egui_renderer: egui_wgpu::Renderer,
    pub viewport: Option<ViewportRenderer>,
    pub meshes: Vec<GpuMesh>,
}

impl Renderer {
    pub async fn new(
        window: Arc<winit::window::Window>,
        size: (u32, u32),
    ) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = create_surface(&instance, &window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    label: Some("SchiroEngine Device"),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        tracing::info!("wgpu device ready, format: {:?}, size: {}x{}", config.format, size.0, size.1);

        let egui_renderer = egui_wgpu::Renderer::new(&device, config.format, None, 1, false);

        let viewport = ViewportRenderer::new(&device, &queue, config.format, (1280, 720));

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            egui_renderer,
            viewport: Some(viewport),
            meshes: Vec::new(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.size = (width, height);
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        if let Some(ref mut vp) = self.viewport {
            vp.resize(&self.device, (width, height));
        }
    }

    pub fn add_mesh(&mut self, mesh_data: &Mesh, transform: &glam::Mat4) {
        let mesh = match &self.viewport {
            Some(vp) => GpuMesh::new(&self.device, mesh_data, transform, &vp.pipeline.model_layout),
            None => return,
        };
        self.meshes.push(mesh);
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn update_mesh_transform(&self, index: usize, transform: &glam::Mat4) -> bool {
        if let Some(mesh) = self.meshes.get(index) {
            mesh.update_transform(&self.queue, transform);
            true
        } else {
            false
        }
    }

    pub fn render(
        &mut self,
        egui_ctx: &egui::Context,
        egui_output: egui::FullOutput,
        camera_uniform: &CameraUniform,
    ) -> Result<(), wgpu::SurfaceError> {
        let Self {
            surface,
            device,
            queue,
            config: _,
            size,
            egui_renderer,
            viewport,
            meshes,
        } = self;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Schiro Command Encoder"),
        });

        if let Some(vp) = viewport.as_mut() {
            let mesh_refs: Vec<&GpuMesh> = meshes.iter().collect();
            vp.render(queue, &mut encoder, camera_uniform, &mesh_refs);

            vp.update_egui_texture(egui_renderer, device);
            if vp.egui_texture_id.is_none() {
                vp.register_egui_texture(egui_renderer, device);
            }
        }

        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.12,
                            g: 0.12,
                            b: 0.16,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.0, size.1],
            pixels_per_point: egui_ctx.pixels_per_point(),
        };

        let primitives = egui_ctx.tessellate(egui_output.shapes, egui_ctx.pixels_per_point());

        for (id, image_delta) in egui_output.textures_delta.set {
            egui_renderer.update_texture(device, queue, id, &image_delta);
        }

        egui_renderer.update_buffers(device, queue, &mut encoder, &primitives, &screen_descriptor);

        {
            let mut egui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // SAFETY: RenderPass<'static> — lifetime is phantom in wgpu.
            // The pass is dropped before the encoder is consumed.
            let pass_static: &mut wgpu::RenderPass<'static> =
                unsafe { std::mem::transmute(&mut egui_pass) };
            egui_renderer.render(pass_static, &primitives, &screen_descriptor);
        }

        for id in egui_output.textures_delta.free {
            egui_renderer.free_texture(&id);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("no suitable GPU adapter found")]
    NoAdapter,
    #[error("wgpu device error: {0}")]
    Wgpu(#[from] wgpu::RequestDeviceError),
    #[error("surface error: {0}")]
    Surface(#[from] wgpu::CreateSurfaceError),
}
