use std::sync::Arc;
use std::time::Duration;
use std::cell::RefCell;

use winit::{
	window::Window,
	event::*,
	dpi::PhysicalSize
};
use rustc_hash::FxHashMap;

use crate::prelude::*;
use crate::render::{Renderer, Aabb};
use crate::render::model::{Mesh, Material};
use camera_controller::CameraController;
use super::player::PlayerId;
use super::world::World;
use super::block::{generate_texture_array, BlockFaceMesh, Air};
use super::render_zone::{UpdatedRenderZones, render_zone_aabb};
use super::ui::MineConeUi;

mod camera_controller;

pub struct Client {
	world: Arc<World>,
	world_mesh: RefCell<FxHashMap<ChunkPos, Mesh>>,
	block_textures: Material,
	player_id: PlayerId,
	camera_controller: CameraController,
	ui: MineConeUi,
	renderer: Renderer,
	window: Window,
	// destroy block on the next physics frame
	destroy_block: bool,
	// this is a set of all the render zones that need their frame updated
	updated_render_zones: UpdatedRenderZones,
}

impl Client {
	pub fn new(window: Window, world: Arc<World>) -> Self {
		let renderer = pollster::block_on(Renderer::new(&window));

		let texture_array = generate_texture_array().expect("could not load texture map");
		let block_textures = Material::array_from_images(&texture_array, String::from("texture map"), renderer.context());

		let player_id = world.connect();

		Self {
			world,
			world_mesh: RefCell::new(FxHashMap::default()),
			block_textures,
			player_id,
			camera_controller: CameraController::new(7.0, 20.0, 2.0),
			ui: MineConeUi::new(&window, &renderer),
			renderer,
			window,
			destroy_block: false,
			updated_render_zones: UpdatedRenderZones::new(),
		}
	}

	fn generate_mesh(&self, render_zone: ChunkPos) {
		let mut vertexes = Vec::new();
		let mut indexes = Vec::new();

		let mut current_index = 0;
		for block_face in self.world.render_zone_mesh(render_zone) {
			vertexes.extend(block_face.0);
			indexes.extend(BlockFaceMesh::indicies().iter().map(|elem| elem + current_index));
			current_index += 4;
		}

		// TODO: write to the underlying buffer
		self.world_mesh.borrow_mut().insert(render_zone, Mesh::new(
			"world mesh",
			&vertexes,
			&indexes,
			0,
			Some(render_zone_aabb(render_zone)),
			self.renderer.context(),
		));
	}

	fn render(&mut self) {
		let world_mesh = self.world_mesh.borrow();
		let models = world_mesh.values().map(|mesh| (mesh, &self.block_textures)).collect::<Vec<_>>();

		let mut tri_count = 0;
		for (mesh, _) in models.iter() {
			tri_count += mesh.triangle_count() as i64;
		}
		debug_display("Triangle Count", &tri_count);

		self.renderer.start_render_pass();		

		self.renderer.render(&models);
		self.ui.frame_update(&self.window, &self.renderer);

		self.renderer.finish_render_pass();
	}

	// TODO: merge this with input
	pub fn handle_event(&mut self, event: &Event<()>) {
		self.ui.handle_event(event);
	}

	pub fn input(&mut self, event: &WindowEvent) {
		self.ui.input(event);
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
		self.render();
	}

	pub fn physics_update(&mut self, delta: Duration) {
		let camera = self.renderer.get_camera_mut();
		self.camera_controller.update_camera(camera, delta);
		let camera_position = camera.get_position();

		if self.destroy_block {
			if let Some(block) = self.world.block_raycast(camera_position, camera.forward(), 15.0) {
				self.world.set_block(block, Air::new().into());
				self.world.mesh_update_adjacent(block, &mut self.updated_render_zones);
			}

			self.destroy_block = false;
		}

		self.world.set_player_position(self.player_id, camera_position);

		self.world.poll_completed_tasks(&mut self.updated_render_zones);
		for render_zone in self.updated_render_zones.iter() {
			self.generate_mesh(*render_zone);
		}
		self.updated_render_zones.clear();

		debug_display("Physics Updates per Second", &((1.0 / delta.as_secs_f64()) as i64));

		self.render();
	}
}
