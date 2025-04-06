use std::{env::current_dir, fs};

use super::bind_group::Builder;
use image::GenericImageView;

pub struct Material {
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        filename: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut filepath = current_dir().unwrap();
        filepath.push(filename);
        let filepath = filepath.into_os_string().into_string().unwrap();

        let bytes = fs::read(filepath).unwrap();

        let loaded_image = image::load_from_memory(&bytes).unwrap();
        let converted = loaded_image.to_rgba8();
        let size = loaded_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some(filename),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        };

        let texture = device.create_texture(&texture_descriptor);

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &converted,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.0 * 4),
                rows_per_image: Some(size.1),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler_descriptor = wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        };

        let sampler = device.create_sampler(&sampler_descriptor);

        let mut builder = Builder::new(&device);
        builder.set_layout(layout);
        builder.add_material(&view, &sampler);
        let bind_group = builder.build(filename);

        Material { bind_group }
    }
}
