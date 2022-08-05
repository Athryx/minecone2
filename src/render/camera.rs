//use nalgebra::{Vector3, Matrix4, Point3};
use glam::{Mat4, Vec3, Vec4};

use crate::prelude::*;

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
}

impl Camera {
	pub fn new(position: Vec3, look_at: Vec3, aspect_ratio: f32) -> Self {
		Self {
			position,
			look_at,
			up: Vec3::Y,
			aspect_ratio,
			fovy: 45.0,
			znear: 0.1,
			zfar: 1000.0,
		}
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
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform([[f32; 4]; 4]);
