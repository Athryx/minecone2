use std::num::NonZeroU32;

use egui_wgpu_backend::ScreenDescriptor;
//use nalgebra::{Point3, Vector3, Scale3, UnitQuaternion, Unit};
use glam::{Vec3, Mat4};
use winit::window::Window;
use wgpu::util::DeviceExt;

use texture::{Texture, DepthTexture};
use camera::Camera;
use model::*;
use crate::game::{BlockVertex, num_textures};

pub mod camera;
pub mod model;
mod bounding_box;
pub use bounding_box::Aabb;
pub mod texture;

#[derive(Debug)]
pub struct Renderer {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	render_pipeline: wgpu::RenderPipeline,
	texture_bind_layout: wgpu::BindGroupLayout,
	depth_texture: DepthTexture,
	camera: Camera,
	camera_modified: bool,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
	surface_texture: Option<wgpu::SurfaceTexture>,
	surface_texture_view: Option<wgpu::TextureView>,
	pub size: winit::dpi::PhysicalSize<u32>,
}

// holds references to important wgpu rendering objects
// to be passed to constructors of other rendering related objects
// which simplifies the amount of arguments thet have to be passed to them
#[derive(Debug, Clone, Copy)]
pub struct RenderContext<'a> {
	device: &'a wgpu::Device,
	queue: &'a wgpu::Queue,
	texture_bind_layout: &'a wgpu::BindGroupLayout,
}

impl Renderer {
	// Creating some of the wgpu types requires async code
	pub async fn new(window: &Window) -> Self {
		let size = window.inner_size();

		let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
		let surface = unsafe { instance.create_surface(window) };

		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			},
		).await.unwrap();

		let features = wgpu::Features::TEXTURE_BINDING_ARRAY
			| wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
			| wgpu::Features::POLYGON_MODE_LINE;

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features,
				limits: wgpu::Limits {
					max_texture_array_layers: 256,
					..Default::default()
				},
				label: None,
			},
			None,
		).await.unwrap();

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_supported_formats(&adapter)[0],
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &config);

		let texture_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("texture bind group layout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: NonZeroU32::new(num_textures()),
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
			}
		);

		let depth_texture = DepthTexture::new(&device, &config, "depth texture");

		// render pipeline
		let camera = Camera::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), config.width as f32 / config.height as f32);
		let camera_uniform = camera.get_camera_uniform();

		let camera_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("camera buffer"),
				contents: bytemuck::cast_slice(&[camera_uniform]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			}
		);

		let camera_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("camera bind group layout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					}
				],
			}
		);

		let camera_bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("camera bind group"),
				layout: &camera_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: camera_buffer.as_entire_binding(),
					},
				],
			}
		);

		let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("render pipeline layout"),
			bind_group_layouts: &[
				&texture_bind_group_layout,
				&camera_bind_group_layout,
			],
			push_constant_ranges: &[],
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("render pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[
					BlockVertex::desc(),
				],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				// Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
				polygon_mode: wgpu::PolygonMode::Fill,
				// Requires Features::DEPTH_CLIP_CONTROL
				unclipped_depth: false,
				// Requires Features::CONSERVATIVE_RASTERIZATION
				conservative: false,
			},
			depth_stencil: Some(wgpu::DepthStencilState {
				format: DepthTexture::DEPTH_FORMAT,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default(),
			}),
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		Self {
			surface,
			device,
			queue,
			config,
			render_pipeline,
			texture_bind_layout: texture_bind_group_layout,
			depth_texture,
			camera,
			camera_modified: false,
			camera_buffer,
			camera_bind_group,
			surface_texture: None,
			surface_texture_view: None,
			size,
		}
	}

	pub fn context(&self) -> RenderContext {
		RenderContext {
			device: &self.device,
			queue: &self.queue,
			texture_bind_layout: &self.texture_bind_layout,
		}
	}

	// used for egui
	pub fn device(&self) -> &wgpu::Device {
		&self.device
	}

	// used for egui
	pub fn queue(&self) -> &wgpu::Queue {
		&self.queue
	}

	// used for egui
	pub fn surface_format(&self) -> wgpu::TextureFormat {
		self.config.format
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
			self.depth_texture = DepthTexture::new(&self.device, &self.config, "depth texture");
		}
	}

	pub fn get_camera_mut(&mut self) -> &mut Camera {
		self.camera_modified = true;
		&mut self.camera
	}

	pub fn start_render_pass(&mut self) {
		let surface_texture = loop {
			match self.surface.get_current_texture() {
				Ok(texture) => break texture,
				// reconfigure surface if lost
				Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
				Err(wgpu::SurfaceError::OutOfMemory) => {
					panic!("out of memory");
				}
				Err(e) => warn!("{:?}", e),
			}
		};
		let surface_texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

		self.surface_texture = Some(surface_texture);
		self.surface_texture_view = Some(surface_texture_view);

		if self.camera_modified {
			self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera.get_camera_uniform()]));
			self.camera_modified = false;
		}
	}

	pub fn finish_render_pass(&mut self) {
		let surface_texture = std::mem::replace(&mut self.surface_texture, None);
		self.surface_texture_view = None;

		surface_texture
			.expect("render pass cannot be finisehd because it was not started")
			.present();
	}

	pub fn output_texture_view(&self) -> Option<&wgpu::TextureView> {
		self.surface_texture_view.as_ref()
	}

	pub fn render(&mut self, models: &[(&Mesh, &Material)]) {
		let view = self.output_texture_view().expect("render pass has not been started");

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("render encoder"),
		});

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("render pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.1,
							g: 0.2,
							b: 0.3,
							a: 1.0,
						}),
						store: true,
					}
				})],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
					view: &self.depth_texture.view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: true,
					}),
					stencil_ops: None,
				}),
			});

			render_pass.set_pipeline(&self.render_pipeline);

			for (mesh, material) in models.iter() {
				if let Some(aabb) = mesh.bounding_box {
					if !self.camera.bounding_box_visible(aabb) {
						continue;
					}
				}
				render_pass.draw_mesh(mesh, material, &self.camera_bind_group);
			}
		}

		self.queue.submit(std::iter::once(encoder.finish()));
	}
}
