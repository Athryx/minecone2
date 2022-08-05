use std::{num::NonZeroU32, path::Path};

use image::{DynamicImage, GenericImageView};
use anyhow::*;

use crate::assets::loader;
use super::RenderContext;

#[derive(Debug)]
pub struct Texture {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
}

impl Texture {
	pub fn from_file<T: AsRef<Path>>(
		file_name: T,
		label: &str,
		context: RenderContext,
	) -> Result<Self> {
		let image = loader().load_image(file_name)?;
		Ok(Self::from_image(&image, label, context))
	}

	pub fn from_bytes(
		bytes: &[u8],
		label: &str,
		context: RenderContext,
	) -> Result<Self> {
		let image = image::load_from_memory(bytes)?;
		Ok(Self::from_image(&image, label, context))
	}

	pub fn from_image(
		image: &DynamicImage,
		label: &str,
		context: RenderContext,
	) -> Self {
		let dimensions = image.dimensions();
		let rgba = image.to_rgba8();

		let texture_size = wgpu::Extent3d {
			width: dimensions.0,
			height: dimensions.1,
			depth_or_array_layers: 1,
		};

		let texture = context.device.create_texture(
			&wgpu::TextureDescriptor {
				label: Some(label),
				// All textures are stored as 3D, we represent our 2D texture
				// by setting depth to 1.
				size: texture_size,
				mip_level_count: 1,
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				// Most images are stored using sRGB so we need to reflect that here.
				format: wgpu::TextureFormat::Rgba8UnormSrgb,
				// TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
				// COPY_DST means that we want to copy data to this texture
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			}
		);

		context.queue.write_texture(
			// where to copy the pixel data to
			wgpu::ImageCopyTexture {
				texture: &texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			// the actual pixel data
			&rgba,
			// the layout of the texture
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: NonZeroU32::new(4 * dimensions.0),
				rows_per_image: NonZeroU32::new(dimensions.1),
			},
			texture_size,
		);

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

		Self {
			texture,
			view,
		}
	}
}

#[derive(Debug)]
pub struct DepthTexture {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
}

impl DepthTexture {
	pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

	// this one is only used in the render code so a RenderContext is not needed
	pub fn new(
		device: &wgpu::Device,
		config: &wgpu::SurfaceConfiguration,
		label: &str
	) -> Self {
		let size = wgpu::Extent3d {
			width: config.width,
			height: config.height,
			depth_or_array_layers: 1,
		};
		let desc = wgpu::TextureDescriptor {
			label: Some(label),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: Self::DEPTH_FORMAT,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT
				| wgpu::TextureUsages::TEXTURE_BINDING,
		};
		let texture = device.create_texture(&desc);

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(
			&wgpu::SamplerDescriptor {
				address_mode_u: wgpu::AddressMode::ClampToEdge,
				address_mode_v: wgpu::AddressMode::ClampToEdge,
				address_mode_w: wgpu::AddressMode::ClampToEdge,
				mag_filter: wgpu::FilterMode::Linear,
				min_filter: wgpu::FilterMode::Linear,
				mipmap_filter: wgpu::FilterMode::Nearest,
				compare: Some(wgpu::CompareFunction::LessEqual),
				lod_min_clamp: -100.0,
				lod_max_clamp: 100.0,
				..Default::default()
			}
		);

		Self {
			texture,
			view,
			sampler
		}
	}
}
