use std::{mem, path::Path};

use anyhow::Result;
use image::DynamicImage;
use wgpu::util::DeviceExt;
//use nalgebra::{Vector3, Scale3, Matrix4, UnitQuaternion};
use glam::{Vec3, Mat4, Quat};

use super::{RenderContext, texture::Texture, Aabb};
use crate::assets::loader;

pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
	pub position: [f32; 3],
	pub tex_coords: [f32; 2],
	pub normal: [f32; 3],
}

impl ModelVertex {
	const ATTRIBS: [wgpu::VertexAttribute; 3] =
		wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
}

impl Vertex for ModelVertex {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::ATTRIBS,
		}
	}
}

#[derive(Debug)]
pub struct Mesh {
	name: String,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	num_elements: u32,
	material_index: usize,
	pub bounding_box: Option<Aabb>,
}

impl Mesh {
	pub fn new<T: Vertex>(
		name: &str,
		vertices: &[T],
		indices: &[u32],
		material_index: usize,
		bounding_box: Option<Aabb>,
		context: RenderContext,
	) -> Self {
		let vertex_buffer = context.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some(&format!("{} vertex buffer", name)),
				contents: bytemuck::cast_slice(vertices),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		let index_buffer = context.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some(&format!("{} index buffer", name)),
				contents: bytemuck::cast_slice(indices),
				usage: wgpu::BufferUsages::INDEX,
			}
		);

		Self {
			name: name.to_owned(),
			vertex_buffer,
			index_buffer,
			num_elements: indices.len().try_into().unwrap(),
			material_index,
			bounding_box,
		}
	}

	pub fn triangle_count(&self) -> u32 {
		self.num_elements / 3
	}
}

#[derive(Debug)]
pub struct Material {
	name: String,
	diffuse_textures: Vec<Texture>,
	diffuse_sampler: wgpu::Sampler,
	bind_group: wgpu::BindGroup,
}

impl Material {
	// for now, file name is file name of diffuse texture
	pub fn load_from_file<T: AsRef<Path>>(
		file_name: T,
		name: String,
		context: RenderContext,
	) -> Result<Self> {
		let diffuse_texture = Texture::from_file(file_name, &format!("{} diffuse texture", name), context)?;

		let diffuse_sampler = context.device.create_sampler(
			&wgpu::SamplerDescriptor {
				address_mode_u: wgpu::AddressMode::ClampToEdge,
				address_mode_v: wgpu::AddressMode::ClampToEdge,
				address_mode_w: wgpu::AddressMode::ClampToEdge,
				// TODO: make adjustable
				mag_filter: wgpu::FilterMode::Nearest,
				min_filter: wgpu::FilterMode::Nearest,
				mipmap_filter: wgpu::FilterMode::Nearest,
				..Default::default()
			}
		);

		let bind_group = context.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some(&format!("{} bind group", name)),
				layout: context.texture_bind_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
					},
				],
			}
		);

		Ok(Self {
			name,
			diffuse_textures: vec![diffuse_texture],
			diffuse_sampler,
			bind_group,
		})
	}

	pub fn array_from_images(
		images: &[DynamicImage],
		name: String,
		context: RenderContext,
	) -> Self {
		let mut diffuse_textures = Vec::with_capacity(images.len());
		for image in images.iter() {
			diffuse_textures.push(Texture::from_image(image, &format!("{} diffuse texture", name), context));
		}

		let mut texture_views = Vec::with_capacity(images.len());
		for texture in diffuse_textures.iter() {
			texture_views.push(&texture.view);
		}

		let diffuse_sampler = context.device.create_sampler(
			&wgpu::SamplerDescriptor {
				address_mode_u: wgpu::AddressMode::ClampToEdge,
				address_mode_v: wgpu::AddressMode::ClampToEdge,
				address_mode_w: wgpu::AddressMode::ClampToEdge,
				// TODO: make adjustable
				mag_filter: wgpu::FilterMode::Nearest,
				min_filter: wgpu::FilterMode::Nearest,
				mipmap_filter: wgpu::FilterMode::Nearest,
				..Default::default()
			}
		);

		let bind_group = context.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some(&format!("{} bind group", name)),
				layout: context.texture_bind_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureViewArray(&texture_views),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
					},
				],
			}
		);

		Self {
			name,
			diffuse_textures,
			diffuse_sampler,
			bind_group,
		}
	}
}

#[derive(Debug)]
pub struct Model {
	pub meshes: Vec<Mesh>,
	pub materials: Vec<Material>,
}

impl Model {
	/*pub fn load_from_file<T: AsRef<Path>>(
		file_name: T,
		context: RenderContext,
	) -> Result<Self> {
		let file_name = file_name.as_ref();

		let (obj_meshes, obj_materials) = loader().load_obj(file_name)?;

		let mut materials = Vec::with_capacity(obj_materials.len());
		for mat in obj_materials.into_iter() {
			materials.push(Material::load_from_file(&mat.diffuse_texture, mat.diffuse_texture.clone(), context)?);
		}

		let mut meshes = Vec::with_capacity(obj_meshes.len());
		for mesh in obj_meshes.into_iter() {
			let vertices = (0..mesh.mesh.positions.len() / 3)
				.map(|i| ModelVertex {
					position: [
						mesh.mesh.positions[i * 3],
						mesh.mesh.positions[i * 3 + 1],
						mesh.mesh.positions[i * 3 + 2],
					],
					tex_coords: [
						mesh.mesh.texcoords[i * 2],
						mesh.mesh.texcoords[i * 2 + 1],
					],
					normal: [
						mesh.mesh.normals[i * 3],
						mesh.mesh.normals[i * 3 + 1],
						mesh.mesh.normals[i * 3 + 2],
					],
				})
				.collect::<Vec<_>>();

			meshes.push(Mesh::new(
				&mesh.name,
				&vertices,
				&mesh.mesh.indices,
				mesh.mesh.material_id.unwrap_or(0),
				context
			));
		}

		Ok(Model {
			meshes,
			materials,
		})
	}*/

	pub fn new(
		name: &str,
		vertices: &[ModelVertex],
		indices: &[u32],
		material: Material,
		bounding_box: Option<Aabb>,
		context: RenderContext,
	) -> Self {
		let mesh = Mesh::new(
			name,
			vertices,
			indices,
			0,
			bounding_box,
			context,
		);

		Self {
			meshes: vec![mesh],
			materials: vec![material],
		}
	}
}

#[derive(Debug)]
pub struct Instance {
	pub translation: Vec3,
	pub rotation: Quat,
	pub scale: Vec3,
}

impl Instance {
	fn to_raw(&self) -> InstanceRaw {
		InstanceRaw(Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation).to_cols_array_2d())
	}
}

impl Default for Instance {
	fn default() -> Self {
		Instance {
			translation: Vec3::ZERO,
			rotation: Quat::IDENTITY,
			scale: Vec3::ONE,
		}
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw([[f32; 4]; 4]);

impl InstanceRaw {
	const ATTRIBS: [wgpu::VertexAttribute; 4] =
		wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];
}

impl Vertex for InstanceRaw {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Instance,
			attributes: &Self::ATTRIBS,
		}
	}
}

#[derive(Debug)]
pub struct ModelInstance {
	model: Model,
	instances: Vec<Instance>,
	instance_buffer: wgpu::Buffer,
}

impl ModelInstance {
	pub fn new(model: Model, instances: Vec<Instance>, context: RenderContext) -> Self {
		let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
		let instance_buffer = context.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("instance buffer"),
				contents: bytemuck::cast_slice(&instance_data),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		Self {
			model,
			instances,
			instance_buffer,
		}
	}

	// ceates a model instance which draws 1 model with no changes
	pub fn identity(model: Model, context: RenderContext) -> Self {
		Self::new(model, vec![Instance::default()], context)
	}

	pub fn num_instances(&self) -> u32 {
		self.instances.len().try_into().unwrap()
	}
}

// model.rs
pub trait DrawModel<'a> {
	fn draw_mesh(
		&mut self,
		mesh: &'a Mesh,
		material: &'a Material,
		camera_bind_group: &'a wgpu::BindGroup,
	);

	// Don't use
	fn draw_model_instanced(
		&mut self,
		model: &'a ModelInstance,
		camera_bind_group: &'a wgpu::BindGroup,
	);
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
	'b: 'a,
{
	fn draw_mesh(
		&mut self,
		mesh: &'b Mesh,
		material: &'b Material,
		camera_bind_group: &'b wgpu::BindGroup,
	) {
		self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
		self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		self.set_bind_group(0, &material.bind_group, &[]);
		self.set_bind_group(1, camera_bind_group, &[]);
		self.draw_indexed(0..mesh.num_elements, 0, 0..1);
	}

	fn draw_model_instanced(
		&mut self,
		model_instance: &'b ModelInstance,
		camera_bind_group: &'b wgpu::BindGroup,
	) {
		self.set_vertex_buffer(1, model_instance.instance_buffer.slice(..));

		for mesh in model_instance.model.meshes.iter() {
			let material = &model_instance.model.materials[mesh.material_index];

			self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
			self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
			self.set_bind_group(0, &material.bind_group, &[]);
			self.set_bind_group(1, camera_bind_group, &[]);
			self.draw_indexed(0..mesh.num_elements, 0, 0..model_instance.num_instances());
		}
	}
}
