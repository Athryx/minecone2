use std::{
	fs::{File, OpenOptions},
	path::Path,
	sync::{Arc, Weak},
};

use rustc_hash::FxHashMap;
use glam::{UVec3, IVec3};
use anyhow::Result;
use parking_lot::RwLock;
use glam::Vec3;
use rustc_hash::FxHashSet;

use super::{
	chunk::{Chunk, LoadedChunk, ChunkData, VisitedBlockMap},
	entity::Entity,
	block::{BlockFaceMesh, BlockFace, Block, BlockTrait},
	worldgen::WorldGenerator,
	player::{Player, PlayerId},
	parallel::{Task, run_task, pull_completed_task},
};
use crate::prelude::*;
use super::render_zone::{RENDER_ZONE_SIZE, UpdatedRenderZones};
use crate::vec3_map;

#[derive(Debug)]
pub struct ChunkMeshFaceData {
	min_chunk: ChunkPos,
	max_chunk: ChunkPos,
	face: BlockFace,
}

impl ChunkMeshFaceData {
	fn into_task(&self) -> Task {
		Task::ChunkMeshFace {
			min_chunk: self.min_chunk,
			max_chunk: self.max_chunk,
			face: self.face,
		}
	}
}

#[derive(Debug)]
struct ChunkLoadJob {
	min_chunk: ChunkPos,
	max_chunk: ChunkPos,
	remaining_chunks: u64,
	// data to run the mesh facing task after the chunk is done loading in
	// TODO: handle when there is more than 1 face to mesh
	mesh_face_task: Option<ChunkMeshFaceData>,
}

impl ChunkLoadJob {
	fn contains_chunk(&self, chunk: ChunkPos) -> bool {
		chunk.x >= self.min_chunk.x
			&& chunk.y >= self.min_chunk.y
			&& chunk.z >= self.min_chunk.z
			&& chunk.x < self.max_chunk.x
			&& chunk.y < self.max_chunk.y
			&& chunk.z < self.max_chunk.z
	}
}

// max size of world in chunks
// 16,384 meters in each x and y direction
// 2,048 meters in z direction
pub const WORLD_MAX_SIZE: UVec3 = UVec3::new(512, 64, 512);

pub struct World {
	self_weak: Weak<Self>,
	players: RwLock<FxHashMap<PlayerId, Player>>,
	entities: RwLock<Vec<Box<dyn Entity>>>,
	pub chunks: FxDashMap<ChunkPos, LoadedChunk>,
	cached_chunks: RwLock<FxHashMap<ChunkPos, ChunkData>>,
	chunk_load_jobs: RwLock<Vec<ChunkLoadJob>>,
	chunk_unload_jobs: RwLock<Vec<ChunkLoadJob>>,
	pub(super) world_generator: WorldGenerator,
	// backing file of the world
	file: File,
}

impl World {
	pub fn load_from_file<T: AsRef<Path>>(file_name: T) -> Result<Arc<Self>> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open(file_name)?;

		Ok(Arc::new_cyclic(|weak| Self {
			self_weak: weak.clone(),
			players: RwLock::new(FxHashMap::default()),
			entities: RwLock::new(Vec::new()),
			chunks: FxDashMap::default(),
			cached_chunks: RwLock::new(FxHashMap::default()),
			chunk_load_jobs: RwLock::new(Vec::new()),
			chunk_unload_jobs: RwLock::new(Vec::new()),
			world_generator: WorldGenerator::new(0),
			file,
		}))
	}

	// TEMP
	pub fn new_test() -> Result<Arc<Self>> {
		let file = OpenOptions::new()
			.read(true)
			.write(true)
			.open("test-world")?;

		let out = Arc::new_cyclic(|weak| Self {
			self_weak: weak.clone(),
			players: RwLock::new(FxHashMap::default()),
			entities: RwLock::new(Vec::new()),
			chunks: FxDashMap::default(),
			cached_chunks: RwLock::new(FxHashMap::default()),
			chunk_load_jobs: RwLock::new(Vec::new()),
			chunk_unload_jobs: RwLock::new(Vec::new()),
			world_generator: WorldGenerator::new(0),
			file,
		});

		Ok(out)
	}

	// TODO: refresh meshes of adjacent chunks when loading is finished
	// TODO: handle unloading of chunks before they finish loading
	// loads all chunks between min_chunk and max_chunk not including max_chunk,
	// or incraments the load count if they are already loaded
	pub fn load_chunks(&self, min_chunk: ChunkPos, max_chunk: ChunkPos, mesh_face_task: Option<ChunkMeshFaceData>) {
		self.chunk_load_jobs.write().push(ChunkLoadJob {
			min_chunk,
			max_chunk,
			remaining_chunks: ((max_chunk.x - min_chunk.x) * (max_chunk.y - min_chunk.y) * (max_chunk.z - min_chunk.z))
				.try_into().unwrap(),
			mesh_face_task,
		});

		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let position = ChunkPos::new(x, y, z);

					run_task(Task::GenerateChunk(position));
				}
			}
		}
	}

	// decraments the load counter of all chunks between min and max chunk, not including max
	// and unloads them if the count reaches 0
	// TODO: refresh meshes of adjacent chunks when unloading is finished
	// TODO: handle unloading before loading is finished
	pub fn unload_chunks(&self, min_chunk: ChunkPos, max_chunk: ChunkPos, mesh_face_task: Option<ChunkMeshFaceData>) {
		self.chunk_load_jobs.write().push(ChunkLoadJob {
			min_chunk,
			max_chunk,
			remaining_chunks: 1,
			mesh_face_task,
		});

		run_task(Task::UnloadChunks {
			min_chunk,
			max_chunk,
		});
	}

	// performs mesh updates on the passed in block as well as all adjacent blocks
	// NOTE: this is not multithreaded, it blocks the current thread until done
	// FIXME: this doesn't update everything it needs to with ambient occlusion on chunk boundaries
	pub fn mesh_update_adjacent(&self, block: BlockPos, updated_render_zones: &mut UpdatedRenderZones) {
		let block_chunk_local = block.as_chunk_local();
		let mut visit_map = VisitedBlockMap::new();

		if let Some(chunk) = self.chunks.get(&block.as_chunk_pos()) {
			chunk.chunk.mesh_update_inner(BlockFace::XPos, block_chunk_local.x as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::XNeg, block_chunk_local.x as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::YPos, block_chunk_local.y as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::YNeg, block_chunk_local.y as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::ZPos, block_chunk_local.z as usize, &mut visit_map);
			chunk.chunk.mesh_update_inner(BlockFace::ZNeg, block_chunk_local.z as usize, &mut visit_map);
			updated_render_zones.mark_block(block);
		}

		for face in BlockFace::iter() {
			// subtract to get opposite as normal offest
			let offset_block = block - face.block_pos_offset();
			if let Some(chunk) = self.chunks.get(&offset_block.as_chunk_pos()) {
				updated_render_zones.mark_block(offset_block);
				chunk.chunk.mesh_update_inner(
					face,
					offset_block.as_chunk_local().get_face_component(face) as usize,
					&mut visit_map
				);
			}
		}
	}

	pub fn chunk_mesh_update(&self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
		for x in min_chunk.x..max_chunk.x {
			for y in min_chunk.y..max_chunk.y {
				for z in min_chunk.z..max_chunk.z {
					let chunk_pos = ChunkPos::new(x, y, z);
					run_task(Task::ChunkMesh(chunk_pos));
				}
			}
		}
	}

	#[inline]
	fn with_block<T, F>(&self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&Block) -> T {
		let (chunk_position, block) = block.as_chunk_block_pos();

		Some(f(&*self.chunks.get(&chunk_position)?
			.chunk.get_block(block.as_chunk_local())))
	}

	// calls the function on the given block position
	// the block may be from another chunk
	#[inline]
	fn with_block_mut<T, F>(&mut self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&mut Block) -> T {
		let (chunk_position, block) = block.as_chunk_block_pos();

		Some(f(&mut *self.chunks.get(&chunk_position)?
			.chunk.get_block_mut(block.as_chunk_local())))
	}

	// sets the block at BlockPos, returns bool on success
	pub fn set_block(&self, block_pos: BlockPos, block: Block) -> bool {
		let (chunk_pos, block_pos) = block_pos.as_chunk_block_pos();

		if let Some(chunk) = self.chunks.get(&chunk_pos) {
			chunk.chunk.set_block(block_pos, block);
			true
		} else {
			false
		}
	}

	// casts a ray starting at ray_start up to a length of max_length
	// if a block other than air is found, the coordinates are returned, otherwise None is returned
	// if the ray ever intersects with an empty chunk, None is returned
	pub fn block_raycast(&self, ray_start: Position, ray: Vec3, max_length: f32) -> Option<BlockPos> {
		let ray = ray.normalize();
		let block_start_pos = ray_start.as_block_pos();
		let mut block_pos = block_start_pos;

		let direction = ray.signum().as_ivec3();

		let intercept_time_interval = ray.map(|elem| {
			if elem != 0.0 {
				(1.0 / elem).abs()
			} else {
				f32::INFINITY
			}
		});

		let ray_offset = ray_start.map(|elem| {
			if elem > 0.0 {
				elem % 1.0
			} else {
				1.0 + (elem % 1.0)
			}
		});

		let mut next_intercept_time = vec3_map!(|ray: f32, ray_offset: f32| {
			if ray > 0.0 {
				(1.0 - ray_offset) / ray
			} else if ray < 0.0 {
				-ray_offset / ray
			} else {
				f32::INFINITY
			}
		}, Vec3, ray, ray_offset);

		let mut incrament_axis = |next_intercept_time: &mut Vec3, axis: Axis| {
			block_pos[axis] += direction[axis];
			if (block_pos - block_start_pos).length() > max_length {
				return Some(None);
			}

			if !self.with_block(block_pos, |b| b.is_air())? {
				return Some(Some(block_pos));
			}

			next_intercept_time[axis] += intercept_time_interval[axis];
			None
		};

		loop {
			if let Some(result) = 
				if next_intercept_time.x < next_intercept_time.y && next_intercept_time.x < next_intercept_time.z {
					incrament_axis(&mut next_intercept_time, Axis::X)
				} else if next_intercept_time.y < next_intercept_time.z {
					incrament_axis(&mut next_intercept_time, Axis::Y)
				} else {
					incrament_axis(&mut next_intercept_time, Axis::Z)
				}
			{
				return result;
			}
		}
	}

	// called by the client to force the world to recieve task completion notices
	// returns true if the mesh should be updated by the client
	pub fn poll_completed_tasks(&self, updated_render_zones: &mut UpdatedRenderZones) {
		while let Some(task) = pull_completed_task() {
			match task {
				Task::ChunkMesh(chunk) => {
					updated_render_zones.mark_chunk(chunk);
				},
				Task::ChunkMeshFace { min_chunk, max_chunk, .. } => {
					updated_render_zones.mark_chunk_zone(min_chunk, max_chunk);
				},
				Task::GenerateChunk(chunk) => {
					let mut load_jobs = self.chunk_load_jobs.write();

					let mut drain_iter = load_jobs.drain_filter(|job| {
						// find out if the chunk is part of this job
						if job.contains_chunk(chunk) {
							job.remaining_chunks -= 1;
							// remove the job if there are no more remaining chunks to temove
							job.remaining_chunks == 0
						} else {
							false
						}
					});

					if let Some(finished_job) = drain_iter.next() {
						self.chunk_mesh_update(finished_job.min_chunk, finished_job.max_chunk);
						if let Some(mesh_face_task) = finished_job.mesh_face_task {
							run_task(mesh_face_task.into_task());
						}
					}
				},
				Task::UnloadChunks { min_chunk, max_chunk } => {
					// recreate mesh because chunks have been removed, but we don't actually have to generate their meshes
					updated_render_zones.mark_chunk_zone(min_chunk, max_chunk);

					let mut unload_jobs = self.chunk_unload_jobs.write();

					let mut drain_iter = unload_jobs.drain_filter(|job| {
						// find out if the chunk is part of this job
						job.min_chunk == min_chunk && job.max_chunk == max_chunk
					});

					if let Some(finished_job) = drain_iter.next() {
						if let Some(mesh_face_task) = finished_job.mesh_face_task {
							run_task(mesh_face_task.into_task());
						}
					}
				}
			}
		}
	}
}

impl World {
	pub fn connect(&self) -> PlayerId {
		let player = Player::new();

		let min_load_chunk = player.chunk_position() - player.render_distance();
		let max_load_chunk = player.chunk_position() + player.render_distance();
		self.load_chunks(min_load_chunk, max_load_chunk, None);

		let id = player.id();
		self.players.write().insert(id, player);
		id
	}

	// TODO: allow changing from more than 1 chunk at at a time
	// TODO: when going along diaganols, sometimes chunks are loaded and immediately unloaded
	// TEMP: returns true if mesh has changed
	pub fn set_player_position(&self, player_id: PlayerId, position: Position) -> Option<bool> {
		let mut players = self.players.write();
		let player = players.get_mut(&player_id)?;

		let chunk_position = position.as_chunk_pos();

		let render_zone_corner = player.chunk_position() - player.render_distance();
		let render_zone_length = 2 * player.render_distance();

		let load_axis = |axis: Axis| {
			if chunk_position[axis] == player.chunk_position()[axis] {
				return;
			}

			let axis_vec = ChunkPos(IVec3::axis(axis));

			let pos_min_chunk = render_zone_corner + render_zone_length.axis_only(axis);
			let pos_max_chunk = render_zone_corner + render_zone_length + axis_vec;

			let neg_min_chunk = render_zone_corner - axis_vec;
			let neg_max_chunk = render_zone_corner + render_zone_length.all_but_axis(axis);

			if chunk_position[axis] == player.chunk_position()[axis] + 1 {
				let neg_min_chunk = neg_min_chunk + axis_vec;
				let neg_max_chunk = neg_max_chunk + axis_vec;

				self.unload_chunks(neg_min_chunk, neg_max_chunk, None);

				let load_face_job = ChunkMeshFaceData {
					face: BlockFace::from_axis(axis, true),
					min_chunk: pos_min_chunk - axis_vec,
					max_chunk: pos_max_chunk - axis_vec,
				};

				self.load_chunks(pos_min_chunk, pos_max_chunk, Some(load_face_job));
			} else if chunk_position[axis] == player.chunk_position()[axis] - 1 {
				let pos_min_chunk = pos_min_chunk - axis_vec;
				let pos_max_chunk = pos_max_chunk - axis_vec;

				self.unload_chunks(pos_min_chunk, pos_max_chunk, None);

				let load_face_job = ChunkMeshFaceData {
					face: BlockFace::from_axis(axis, false),
					min_chunk: neg_min_chunk + axis_vec,
					max_chunk: neg_max_chunk + axis_vec,
				};

				self.load_chunks(neg_min_chunk, neg_max_chunk, Some(load_face_job));
			} else {
				todo!("moved to far for current player moving code");
			}
		};

		load_axis(Axis::X);
		load_axis(Axis::Y);
		load_axis(Axis::Z);

		let out = chunk_position != player.chunk_position();

		player.position = position;
		Some(out)
	}

	pub fn render_zone_mesh(&self, render_zone: ChunkPos) -> Vec<BlockFaceMesh> {
		let render_zone_end = render_zone + ChunkPos::splat(RENDER_ZONE_SIZE);

		let mut out = Vec::new();

		for x in render_zone.x..render_zone_end.x {
			for y in render_zone.y..render_zone_end.y {
				for z in render_zone.z..render_zone_end.z {
					let chunk_pos = ChunkPos::new(x, y, z);

					if let Some(chunk) = self.chunks.get(&chunk_pos) {
						if let Some(mesh) = chunk.chunk.get_chunk_mesh() {
							out.extend(mesh);
						}
					}
				}
			}
		}

		out
	}
}

#[cfg(test)]
mod tests {
	extern crate test;

	use test::Bencher;
	use super::*;

	#[bench]
	fn mesh_generation_benchmark(b: &mut Bencher) {
		b.iter(|| {
			let world = World::new_test().unwrap();
			world.connect();
		})
	}
}
