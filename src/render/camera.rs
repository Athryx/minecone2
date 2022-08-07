//use nalgebra::{Vector3, Matrix4, Point3};
use glam::{Mat4, Vec3, Vec4, Quat};

use crate::prelude::*;
use crate::math::Plane;

use super::Aabb;

const TO_GPU_MATRIX: Mat4 = Mat4 {
	x_axis: Vec4::new(1.0, 0.0, 0.0, 0.0),
	y_axis: Vec4::new(0.0, 1.0, 0.0, 0.0),
	z_axis: Vec4::new(0.0, 0.0, 0.5, 0.0),
	w_axis: Vec4::new(0.0, 0.0, 0.5, 1.0),
};

#[derive(Debug)]
pub struct Camera {
	// these need to be public because camera controller modifies these
	pub position: Vec3,
	pub look_at: Vec3,
	pub up: Vec3,
	aspect_ratio: f32,
	fovy: f32,
	znear: f32,
	zfar: f32,
	// used for culling aabb
	frustum_planes: [Plane; 4],
}

impl Camera {
	pub fn new(position: Vec3, look_at: Vec3, aspect_ratio: f32) -> Self {
		let mut out = Self {
			position,
			look_at,
			up: Vec3::Y,
			aspect_ratio,
			fovy: 45.0,
			znear: 0.1,
			zfar: 1000.0,
			frustum_planes: [Plane::default(); 4],
		};

		out.generate_frustum();

		out
	}

	fn fovx(&self) -> f32 {
		self.fovy * self.aspect_ratio
	}

	// must be called after changing camera position
	pub fn generate_frustum(&mut self) {
		let half_y_side = self.zfar * (self.fovy * 0.5).tan();
		let half_x_side = half_y_side * self.aspect_ratio;
		let forward_far = self.zfar * self.forward();
		let sideways = self.sideways();
		// this up is different than self.up
		let up = sideways.cross(self.forward());

		// right
		self.frustum_planes[0] = Plane::new(
			self.position,
			up.cross(forward_far + half_x_side * sideways).normalize(),
		);
		// left
		self.frustum_planes[1] = Plane::new(
			self.position,
			(forward_far - half_x_side * sideways).cross(up).normalize(),
		);
		// bottom
		self.frustum_planes[2] = Plane::new(
			self.position,
			sideways.cross(forward_far - half_y_side * up).normalize(),
		);
		// top
		self.frustum_planes[2] = Plane::new(
			self.position,
			(forward_far + half_y_side * up).cross(sideways).normalize(),
		);
	}

	pub fn get_camera_matrix(&self) -> Mat4 {
		// FIXME: these should not be opposite, but it seems like that is what works
		// probably because wgpu coordinates differ from game coordinates
		let view = Mat4::look_at_lh(self.look_at, self.position, self.up);
		let proj = Mat4::perspective_rh(self.fovy, self.aspect_ratio, self.znear, self.zfar);

		TO_GPU_MATRIX * proj * view
	}

	// gets a camera uniform which can be sent to the gpu
	pub fn get_camera_uniform(&self) -> CameraUniform {
		CameraUniform(self.get_camera_matrix().to_cols_array_2d())
	}

	pub fn get_position(&self) -> Position {
		Position::new(self.position.x, self.position.y, self.position.z)
	}

	pub fn forward(&self) -> Vec3 {
		self.look_at - self.position
	}

	// sideways is pointing right
	pub fn sideways(&self) -> Vec3 {
		self.forward().cross(self.up).normalize()
	}

	// returns true if any part of the axis aligned bounding box is vivisble in the camera
	pub fn bounding_box_visible(&self, aabb: Aabb) -> bool {
		// this might be cleaner with iter reduce, but i'm not sure if that would get as optimized
		aabb.inside_of_plane(self.frustum_planes[0])
			&& aabb.inside_of_plane(self.frustum_planes[1])
			&& aabb.inside_of_plane(self.frustum_planes[2])
			&& aabb.inside_of_plane(self.frustum_planes[3])
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform([[f32; 4]; 4]);
