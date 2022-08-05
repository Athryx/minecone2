use std::sync::Arc;
use std::time::Duration;

use winit::{
	window::Window,
	event::*,
	dpi::PhysicalSize
};

use crate::prelude::*;
use crate::render::Renderer;
use crate::render::model::{Mesh, Material, ModelVertex};
use camera_controller::CameraController;
use super::player::PlayerId;
use super::world::World;
use super::block::{generate_texture_array, BlockFaceMesh, Air};

mod camera_controller;

pub struct Client {
	world: Arc<World>,
	world_mesh: Mesh,
	block_textures: Material,
	player_id: PlayerId,
	camera_controller: CameraController,
	renderer: Renderer,
	// destroy block on the next physics frame
	destroy_block: bool,
	// the number of physics updates that will happen before we update again
	// reset evrytime world tells us to update the mesh
	// the reason we wait a bit is because when loading new chunks world will tell us to update many times
	// in rapid succession, and waiting a bit will improve performance
	mesh_update_countdown: Option<u64>,
}

impl Client {
	const MESH_UPDATE_FRAME_DELAY: u64 = 5;

	pub fn new(window: &Window, world: Arc<World>) -> Self {
		let renderer = pollster::block_on(Renderer::new(window));

		let texture_array = generate_texture_array().expect("could not load texture map");
		let block_textures = Material::array_from_images(&texture_array, String::from("texture map"), renderer.context());

		let player_id = world.connect();

		let mut vertexes = Vec::new();
		let mut indexes = Vec::new();

		let mut current_index = 0;
		for block_face in world.world_mesh() {
			vertexes.extend(block_face.0);
			indexes.extend(BlockFaceMesh::indicies().iter().map(|elem| elem + current_index));
			current_index += 4;
		}

		let mesh = Mesh::new(
			"world mesh",
			&vertexes,
			&indexes,
			0,
			renderer.context()
		);

		Self {
			world,
			world_mesh: mesh,
			block_textures,
			player_id,
			camera_controller: CameraController::new(7.0, 20.0, 2.0),
			renderer,
			destroy_block: false,
			mesh_update_countdown: None,
		}
	}

	pub fn generate_mesh(&mut self) {
		let mut vertexes = Vec::new();
		let mut indexes = Vec::new();

		let mut current_index = 0;
		for block_face in self.world.world_mesh() {
			vertexes.extend(block_face.0);
			indexes.extend(BlockFaceMesh::indicies().iter().map(|elem| elem + current_index));
			current_index += 4;
		}

		// TODO: write to the underlying buffer
		self.world_mesh = Mesh::new(
			"world mesh",
			&vertexes,
			&indexes,
			0,
			self.renderer.context()
		);
	}

	pub fn input(&mut self, event: &WindowEvent) {
		self.camera_controller.process_event(event);
		if let WindowEvent::KeyboardInput {
			input: KeyboardInput {
				state: ElementState::Pressed,
				virtual_keycode: Some(VirtualKeyCode::Return),
				..
			},
			..
		} = event {
			self.destroy_block = true;
		}
	}

	pub fn frame_update(&mut self, new_window_size: Option<PhysicalSize<u32>>) {
		if let Some(new_window_size) = new_window_size {
			self.renderer.resize(new_window_size);
		}
		self.renderer.render(&[(&self.world_mesh, &self.block_textures)]);
	}

	pub fn physics_update(&mut self, delta: Duration) {
		let camera = self.renderer.get_camera_mut();
		self.camera_controller.update_camera(camera, delta);
		let camera_position = camera.get_position();

		let mut generate_mesh = false;

		if self.destroy_block {
			if let Some(block) = self.world.block_raycast(camera_position, camera.forward(), 15.0) {
				self.world.set_block(block, Air::new().into());
				self.world.mesh_update_adjacent(block);
				generate_mesh = true;
			}

			self.destroy_block = false;
		}

		if let Some(result) = self.world.set_player_position(self.player_id, camera_position) {
			if result {
				generate_mesh = true;
			}
		}

		if self.world.poll_completed_tasks() {
			generate_mesh = true;
		}

		if generate_mesh {
			self.mesh_update_countdown = Some(Self::MESH_UPDATE_FRAME_DELAY);
		} else if let Some(ref mut countdown) = self.mesh_update_countdown {
			*countdown -= 1;
			if *countdown == 0 {
				self.mesh_update_countdown = None;
				self.generate_mesh();
			}
		}

		self.renderer.render(&[(&self.world_mesh, &self.block_textures)]);
	}
}
