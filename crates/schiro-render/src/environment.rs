//! Environment maps and BRDF integration LUT used by the PBR IBL
//! pipeline.
//!
//! Three resources are exposed:
//!
//! - A **prefilter cubemap** that holds the pre-filtered specular
//!   environment. The roughness is encoded in the mip level.
//! - An **irradiance cubemap** that holds the diffuse irradiance
//!   (convolved once, regardless of roughness).
//! - A **2D BRDF LUT** that integrates the Cook-Torrance specular
//!   BRDF for a range of (NdotV, roughness) pairs.
//!
//! # Defaults
//!
//! [`Environment::new`] builds a neutral "studio" environment
//! containing a single solid color per cubemap face and a zeroed
//! BRDF LUT. This lets the editor boot without an HDR asset and
//! produces a deterministic, faintly lit preview that can be
//! swapped for a real HDRI at runtime.

use crate::pipeline::MaterialParams;

/// Cube face indices in upload order, matching the wgpu convention
/// (PosX = 0, NegX = 1, PosY = 2, NegY = 3, PosZ = 4, NegZ = 5).
pub const CUBE_FACES: [u32; 6] = [0, 1, 2, 3, 4, 5];

/// Side of the cube faces used for the default environment.
const ENV_SIZE: u32 = 1;

/// Size of the BRDF integration LUT. 256 matches the glTF Sample
/// Viewer and the LearnOpenGL PBR tutorial.
const BRDF_LUT_SIZE: u32 = 256;

/// IBL resources: prefilter cubemap, irradiance cubemap, BRDF LUT,
/// samplers, and a default material bind group that can be reused
/// across meshes that share the same environment.
pub struct Environment {
    /// Texture view of the prefilter cubemap.
    pub prefilter_view: wgpu::TextureView,
    /// Default sampler used for both environment cubemaps.
    pub env_sampler: wgpu::Sampler,
    /// Texture view of the irradiance cubemap.
    pub irradiance_view: wgpu::TextureView,
    /// Texture view of the BRDF integration LUT.
    pub brdf_lut_view: wgpu::TextureView,
    /// Filtering sampler used for the BRDF LUT.
    pub brdf_sampler: wgpu::Sampler,
    /// Number of mip levels in the prefilter cubemap. The shader
    /// multiplies the roughness by this value to choose the right
    /// LOD.
    pub prefilter_levels: f32,
    /// Cached bind group that points to the default textures and a
    /// sensible [`MaterialParams`] uniform.
    pub bind_group: wgpu::BindGroup,
    /// Uniform buffer holding the default material parameters. Kept
    /// here so the caller can update the values via
    /// `queue.write_buffer` if needed.
    pub default_material_buffer: wgpu::Buffer,
}

impl Environment {
    /// Builds a neutral default environment and the matching
    /// bind group.
    ///
    /// `device` and `queue` are used to upload the placeholder pixel
    /// data; `material_layout` is the bind group layout created by
    /// [`crate::pipeline::PbrPipeline`].
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // ----------------------------------------------------------------
        // 1x1 white placeholder texture (reused for base color, normal
        // and metallic-roughness slots).
        // ----------------------------------------------------------------
        let dummy = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default White"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let white_pixel: [u8; 4] = [255, 255, 255, 255];
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &dummy,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &white_pixel,
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let dummy_view = dummy.create_view(&wgpu::TextureViewDescriptor::default());

        // ----------------------------------------------------------------
        // Prefilter cubemap (Rgba16Float, one mid-gray pixel per face).
        // ----------------------------------------------------------------
        let prefilter = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("IBL Prefilter"),
            size: wgpu::Extent3d { width: ENV_SIZE, height: ENV_SIZE, depth_or_array_layers: 6 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let prefilter_view = prefilter.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        for face in CUBE_FACES {
            upload_solid_face(queue, &prefilter, face, [0.5, 0.5, 0.5, 1.0]);
        }

        // ----------------------------------------------------------------
        // Irradiance cubemap (Rgba16Float, one bluish-gray pixel).
        // ----------------------------------------------------------------
        let irradiance = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("IBL Irradiance"),
            size: wgpu::Extent3d { width: ENV_SIZE, height: ENV_SIZE, depth_or_array_layers: 6 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let irradiance_view = irradiance.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        for face in CUBE_FACES {
            upload_solid_face(queue, &irradiance, face, [0.3, 0.3, 0.35, 1.0]);
        }

        // ----------------------------------------------------------------
        // BRDF LUT (Rg16Float, zeroed -> no specular contribution).
        // ----------------------------------------------------------------
        let brdf_lut = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("BRDF LUT"),
            size: wgpu::Extent3d {
                width: BRDF_LUT_SIZE,
                height: BRDF_LUT_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let brdf_lut_view = brdf_lut.create_view(&wgpu::TextureViewDescriptor::default());
        let lut_bytes = vec![0u8; (BRDF_LUT_SIZE * BRDF_LUT_SIZE * 4) as usize];
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &brdf_lut,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &lut_bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(BRDF_LUT_SIZE * 4),
                rows_per_image: Some(BRDF_LUT_SIZE),
            },
            wgpu::Extent3d {
                width: BRDF_LUT_SIZE,
                height: BRDF_LUT_SIZE,
                depth_or_array_layers: 1,
            },
        );

        // ----------------------------------------------------------------
        // Samplers.
        // ----------------------------------------------------------------
        let env_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("IBL Env Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let brdf_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("BRDF LUT Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // ----------------------------------------------------------------
        // Default material uniform + bind group.
        // ----------------------------------------------------------------
        let mut params = MaterialParams::default();
        // Give the default material a slight blueish tint so the
        // editor preview is not flat gray.
        params.base_color = [0.7, 0.72, 0.78, 1.0];
        params.roughness = 0.5;
        params.metallic = 0.0;

        let default_material_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Default Material Params"),
            size: std::mem::size_of::<MaterialParams>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&default_material_buffer, 0, bytemuck::bytes_of(&params));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Default Material BG"),
            layout: material_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&env_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&env_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&env_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&prefilter_view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&env_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&irradiance_view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(&env_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(&brdf_lut_view),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::Sampler(&brdf_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: default_material_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            prefilter_view,
            env_sampler,
            irradiance_view,
            brdf_lut_view,
            brdf_sampler,
            prefilter_levels: 1.0,
            bind_group,
            default_material_buffer,
        }
    }

    /// Uploads a new set of [`MaterialParams`] to the default material
    /// uniform. Affects every mesh that uses the default bind group
    /// on the next frame.
    pub fn update_default_material(&self, queue: &wgpu::Queue, params: &MaterialParams) {
        queue.write_buffer(&self.default_material_buffer, 0, bytemuck::bytes_of(params));
    }
}

/// Uploads a single RGBA32 float pixel to one face of a `Rgba16Float`
/// cubemap-sized texture.
fn upload_solid_face(queue: &wgpu::Queue, texture: &wgpu::Texture, face: u32, rgba: [f32; 4]) {
    let r = half::f16::from_f32(rgba[0]);
    let g = half::f16::from_f32(rgba[1]);
    let b = half::f16::from_f32(rgba[2]);
    let a = half::f16::from_f32(rgba[3]);
    let mut bytes = [0u8; 8];
    bytes[0..2].copy_from_slice(&r.to_bits().to_le_bytes());
    bytes[2..4].copy_from_slice(&g.to_bits().to_le_bytes());
    bytes[4..6].copy_from_slice(&b.to_bits().to_le_bytes());
    bytes[6..8].copy_from_slice(&a.to_bits().to_le_bytes());

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y: 0, z: face },
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(8), rows_per_image: Some(1) },
        wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
    );
}
