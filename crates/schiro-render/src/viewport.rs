use crate::camera::{CameraUniform, LightUniform};
use crate::mesh::GpuMesh;
use crate::pipeline::PbrPipeline;

pub struct ViewportRenderer {
    pub color_texture: wgpu::Texture,
    pub color_view: wgpu::TextureView,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub size: (u32, u32),
    pub pipeline: PbrPipeline,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group: wgpu::BindGroup,
    pub egui_texture_id: Option<egui::TextureId>,
}

impl ViewportRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let pipeline = PbrPipeline::new(device, surface_format);

        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Viewport Color"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Viewport Depth"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform"),
            size: size_of::<CameraUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera BG"),
            layout: &pipeline.camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Uniform"),
            size: size_of::<LightUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light BG"),
            layout: &pipeline.light_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        let light = LightUniform::default();
        queue.write_buffer(&light_buffer, 0, bytemuck::bytes_of(&light));

        Self {
            color_texture,
            color_view,
            depth_texture,
            depth_view,
            size,
            pipeline,
            camera_buffer,
            camera_bind_group,
            light_buffer,
            light_bind_group,
            egui_texture_id: None,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        if size.0 == 0 || size.1 == 0 || size == self.size {
            return;
        }
        self.size = size;

        self.color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Viewport Color"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.color_texture.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.color_view = self.color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Viewport Depth"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.egui_texture_id = None;
    }

    pub fn render(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        camera_uniform: &CameraUniform,
        meshes: &[&GpuMesh],
    ) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(camera_uniform));

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.08,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline.pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_bind_group(2, &self.light_bind_group, &[]);

            for mesh in meshes {
                mesh.draw(&mut pass);
            }
        }
    }

    pub fn register_egui_texture(
        &mut self,
        egui_renderer: &mut egui_wgpu::Renderer,
        device: &wgpu::Device,
    ) -> egui::TextureId {
        let id = egui_renderer.register_native_texture(
            device,
            &self.color_view,
            wgpu::FilterMode::Linear,
        );
        self.egui_texture_id = Some(id);
        id
    }

    pub fn update_egui_texture(
        &mut self,
        egui_renderer: &mut egui_wgpu::Renderer,
        device: &wgpu::Device,
    ) {
        if let Some(id) = self.egui_texture_id {
            egui_renderer.update_egui_texture_from_wgpu_texture(
                device,
                &self.color_view,
                wgpu::FilterMode::Linear,
                id,
            );
        }
    }
}
