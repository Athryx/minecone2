use std::time::Duration;

use winit::event::*;
//use nalgebra::{Unit, Matrix, Vector4};
use glam::{Mat4, Vec4, Vec4Swizzles};

use crate::render::camera::Camera;

#[derive(Debug)]
pub struct CameraController {
	// speed and fast_speed in meters / second
	speed: f32,
	fast_speed: f32,
	// radians / second
	rotation_speed: f32,
	forward_pressed: bool,
	backward_pressed: bool,
	left_pressed: bool,
	right_pressed: bool,
	up_pressed: bool,
	down_pressed: bool,
	rotate_up_pressed: bool,
	rotate_down_pressed: bool,
	rotate_left_pressed: bool,
	rotate_right_pressed: bool,
	sprint_pressed: bool,
}

impl CameraController {
	pub fn new(speed: f32, fast_speed: f32, rotation_speed: f32) -> Self {
		Self {
			speed,
			fast_speed,
			rotation_speed,
			forward_pressed: false,
			backward_pressed: false,
			left_pressed: false,
			right_pressed: false,
			up_pressed: false,
			down_pressed: false,
			rotate_up_pressed: false,
			rotate_down_pressed: false,
			rotate_left_pressed: false,
			rotate_right_pressed: false,
			sprint_pressed: false,
		}
	}

	pub fn process_event(&mut self, event: &WindowEvent) -> bool {
		match event {
			WindowEvent::KeyboardInput {
				input: KeyboardInput {
					state,
					virtual_keycode: Some(keycode),
					..
				},
				..
			} => {
				let is_pressed = *state == ElementState::Pressed;
				match keycode {
					VirtualKeyCode::W => {
						self.forward_pressed = is_pressed;
						true
					},
					VirtualKeyCode::S => {
						self.backward_pressed = is_pressed;
						true
					},
					VirtualKeyCode::A => {
						self.left_pressed = is_pressed;
						true
					},
					VirtualKeyCode::D => {
						self.right_pressed = is_pressed;
						true
					},
					VirtualKeyCode::Space => {
						self.up_pressed = is_pressed;
						true
					},
					VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => {
						self.down_pressed = is_pressed;
						true
					},
					VirtualKeyCode::Up => {
						self.rotate_up_pressed = is_pressed;
						true
					},
					VirtualKeyCode::Down => {
						self.rotate_down_pressed = is_pressed;
						true
					},
					VirtualKeyCode::Left => {
						self.rotate_left_pressed = is_pressed;
						true
					},
					VirtualKeyCode::Right => {
						self.rotate_right_pressed = is_pressed;
						true
					},
					VirtualKeyCode::LShift | VirtualKeyCode::RShift => {
						self.sprint_pressed = is_pressed;
						true
					},
					_ => false,
				}
			}
			_ => false,
		}
	}

	pub fn update_camera(&self, camera: &mut Camera, time_delta: Duration) {
		let forward = camera.look_at - camera.position;
		let up = camera.up;
		// sideways is pointing right
		let right = forward.cross(up);
		// up from the perspective of the camera
		let camera_up = right.cross(forward);

		let forward_norm = forward.normalize();
		let right_norm = right.normalize();
		let camera_up_norm = camera_up.normalize();


		let distance_moved = time_delta.as_millis() as f32 * 
			if self.sprint_pressed {
				self.fast_speed
			} else {
				self.speed
			} / 1000.0;

		if self.forward_pressed {
			camera.position += forward_norm * distance_moved;
		}
		if self.backward_pressed {
			camera.position -= forward_norm * distance_moved;
		}
		if self.left_pressed {
			camera.position -= right_norm * distance_moved;
		}
		if self.right_pressed {
			camera.position += right_norm * distance_moved;
		}
		if self.up_pressed {
			camera.position += camera_up_norm * distance_moved;
		}
		if self.down_pressed {
			camera.position -= camera_up_norm * distance_moved;
		}


		let angle_rotated = time_delta.as_millis() as f32 * self.rotation_speed / 1000.0;

		let mut forward4 = Vec4::new(forward.x, forward.y, forward.z, 0.0);

		if self.rotate_up_pressed {
			let verticle_rotation = Mat4::from_axis_angle(right_norm, angle_rotated);
			let forward_temp = verticle_rotation * forward4;
			if forward_temp.xyz().normalize().dot(up) < 0.98 {
				forward4 = forward_temp;
			}
		}
		if self.rotate_down_pressed {
			let verticle_rotation = Mat4::from_axis_angle(right_norm, -angle_rotated);
			let forward_temp = verticle_rotation * forward4;
			if forward_temp.xyz().normalize().dot(up) > -0.98 {
				forward4 = forward_temp;
			}
		}

		if self.rotate_left_pressed {
			let horizantal_rotation = Mat4::from_axis_angle(up, angle_rotated);
			forward4 = horizantal_rotation * forward4;
		}
		if self.rotate_right_pressed {
			let horizantal_rotation = Mat4::from_axis_angle(up, -angle_rotated);
			forward4 = horizantal_rotation * forward4;
		}

		let forward = forward4.xyz();
		camera.look_at = camera.position + forward;
	}
}
