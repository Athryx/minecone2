use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use array_init::array_init;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::block::{Block, BlockTrait, BlockFaceMesh, BlockFace, OcclusionCorners};
use super::entity::Entity;
use super::world::World;
use crate::prelude::*;

pub const CHUNK_SIZE: usize = 32;

// says all blocks that have been visited for the greedy meshing algorithm in a given layer
pub struct VisitedBlockMap {
	face: BlockFace,
	coord3: i32,
}

impl VisitedBlockMap {
	// the face and coord3 are used to return the correct number in the unused coordinate of the 3rd vector
	// face and coord3 should be set according to the current slice we are iterating over
	pub fn new() -> Self {
		VisitedBlockMap {
			face: BlockFace::XPos,
			coord3: 0,
		}
	}

	fn get_index(&self, position: BlockPos) -> (usize, usize) {
		let (x, y) = match self.face {
			BlockFace::XPos | BlockFace::XNeg => (position.y, position.z),
			BlockFace::YPos | BlockFace::YNeg => (position.x, position.z),
			BlockFace::ZPos | BlockFace::ZNeg => (position.x, position.y),
		};
		(x.try_into().unwrap(), y.try_into().unwrap())
	}

	fn get_block_pos(&self, x: i32, y: i32) -> BlockPos {
		match self.face {
			BlockFace::XPos | BlockFace::XNeg => BlockPos::new(self.coord3, x, y),
			BlockFace::YPos | BlockFace::YNeg => BlockPos::new(x, self.coord3, y),
			BlockFace::ZPos | BlockFace::ZNeg => BlockPos::new(x, y, self.coord3),
		}
	}

	fn get_block_pos_offset(&self, block: BlockPos, x_offset: i32, y_offset: i32) -> BlockPos {
		match self.face {
			BlockFace::XPos | BlockFace::XNeg => BlockPos::new(self.coord3, block.y + x_offset, block.z + y_offset),
			BlockFace::YPos | BlockFace::YNeg => BlockPos::new(block.x + x_offset, self.coord3, block.z + y_offset),
			BlockFace::ZPos | BlockFace::ZNeg => BlockPos::new(block.x + x_offset, block.y + y_offset, self.coord3),
		}
	}

	fn set_face_coord(&mut self, face: BlockFace, coord3: i32) {
		self.face = face;
		self.coord3 = coord3;
	}
}

type BlockArray = Box<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>;

pub struct ChunkBlockRef<'a> {
	_block_lock: RwLockReadGuard<'a, BlockArray>,
	block: *const Block,
}

impl Deref for ChunkBlockRef<'_> {
	type Target = Block;

	fn deref(&self) -> &Self::Target {
		// safety: read lock will ensure this block is still alive
		unsafe { self.block.as_ref().unwrap() }
	}
}

pub struct ChunkBlockRefMut<'a> {
	_block_lock: RwLockWriteGuard<'a, BlockArray>,
	block: *mut Block,
}

impl Deref for ChunkBlockRefMut<'_> {
	type Target = Block;

	fn deref(&self) -> &Self::Target {
		// safety: write lock will ensure this block is still alive
		unsafe { self.block.as_ref().unwrap() }
	}
}

impl DerefMut for ChunkBlockRefMut<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		// safety: write lock will ensure this block is still alive
		unsafe { self.block.as_mut().unwrap() }
	}
}

pub struct Chunk {
	world: Arc<World>,
	// position of back bottom left corner of chunk in block coordinates
	// increases in incraments of 32
	position: Position,
	// coordinates of chunk, increases in incraments of 1
	chunk_position: ChunkPos,
	// coordinates of bottom left back block in world space
	block_position: BlockPos,
	// store them on heap to avoid stack overflow
	blocks: RwLock<BlockArray>,
	//chunk_mesh: HashMap<BlockPos, Vec<BlockFaceMesh>>,
	chunk_mesh: RwLock<Box<[[Vec<BlockFaceMesh>; CHUNK_SIZE]; 6]>>,
}

impl Chunk {
	pub fn new<F: FnMut(BlockPos) -> Block>(world: Arc<World>, position: ChunkPos, mut block_fn: F) -> Self {
		let block_position = position.as_block_pos();

		let blocks = Box::new(array_init(|x| {
			array_init(|y| {
				array_init(|z| {
					let block = BlockPos::new(x as i32, y as i32, z as i32) + block_position;
					block_fn(block)
				})
			})
		}));

		let x = (position.x * CHUNK_SIZE as i32) as f32;
		let y = (position.y * CHUNK_SIZE as i32) as f32;
		let z = (position.z * CHUNK_SIZE as i32) as f32;
		
		Self {
			world,
			position: Position::new(x, y, z),
			chunk_position: position,
			block_position,
			blocks: RwLock::new(blocks),
			chunk_mesh: RwLock::new(Box::new(array_init(|_| array_init(|_| Vec::new())))),
		}
	}

	// calls the function on the given block position
	// the block may be from another chunk
	#[inline]
	fn with_block<T, F>(&self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&Block) -> T {
		if block.is_chunk_local() {
			Some(f(&self.get_block(block)))
		} else {
			let chunk_position = block.as_chunk_pos() + self.chunk_position;

			Some(f(&self.world
				.chunks.get(&chunk_position)?
				.chunk.get_block(block.as_chunk_local())))
		}
	}

	// calls the function on the given block position
	// the block may be from another chunk
	// FIXME: figure out how to make this work without potentialy deadlocking (it might not even be needed though, so maybe remove)
	/*#[inline]
	fn with_block_mut<T, F>(&self, block: BlockPos, f: F) -> Option<T>
		where F: FnOnce(&mut dyn Block) -> T {
		if block.is_chunk_local() {
			Some(f(&mut *self.get_block_mut(block)))
		} else {
			let chunk_position = block.as_chunk_pos() + self.chunk_position;

			Some(f(&mut *self.world
				.chunks.read().get(&chunk_position)?
				.chunk.get_block_mut(block.as_chunk_local())))
		}
	}*/

	#[inline]
	pub fn get_block(&self, block: BlockPos) -> ChunkBlockRef {
		assert!(block.is_chunk_local());
		let (x, y, z) = block.as_indicies().unwrap();

		let block_lock = self.blocks.read();
		let block = &block_lock[x][y][z] as *const Block;
		ChunkBlockRef {
			_block_lock: block_lock,
			block,
		}
	}

	#[inline]
	pub fn get_block_mut(&self, block: BlockPos) -> ChunkBlockRefMut {
		assert!(block.is_chunk_local());
		let (x, y, z) = block.as_indicies().unwrap();

		let mut block_lock = self.blocks.write();
		let block = &mut block_lock[x][y][z] as *mut Block;
		ChunkBlockRefMut {
			_block_lock: block_lock,
			block,
		}
	}

	#[inline]
	pub fn set_block(&self, block_pos: BlockPos, block: Block) {
		assert!(block_pos.is_chunk_local());
		let (x, y, z) = block_pos.as_indicies().unwrap();

		self.blocks.write()[x][y][z] = block;
	}

	// the visit map is passed in seperately to avoid having to reallocat the memory for the visit map every time	
	pub fn mesh_update_inner(&self, face: BlockFace, index: usize, visit_map: &mut VisitedBlockMap) {
		visit_map.set_face_coord(face, index as i32);
		let mut chunk_mesh = self.chunk_mesh.write();
		chunk_mesh[Into::<usize>::into(face)][index].clear();

		let face_offset = face.block_pos_offset();

		let is_occluded_by = |block_pos| {
			if let Some(is_translucent) = self.with_block(block_pos + face_offset, |block| block.is_translucent()) {
				if is_translucent {
					0
				} else {
					1
				}
			} else {
				0
			}
		};

		// FIXME: inconsistance arguments
		let vertex_occlusion_level = |x: i32, y: i32| {
			let tl_occludes = is_occluded_by(visit_map.get_block_pos(x - 1, y - 1));
			let tr_occludes = is_occluded_by(visit_map.get_block_pos(x, y - 1));
			let bl_occludes = is_occluded_by(visit_map.get_block_pos(x - 1, y));
			let br_occludes = is_occluded_by(visit_map.get_block_pos(x, y));

			let mut occlusion_level = tl_occludes + tr_occludes + bl_occludes + br_occludes;
			// if the vertex is in a corner formed by only 2 blocks, the occlusion level needs to be 3
			if (tl_occludes == 1 && br_occludes == 1) || (tr_occludes == 1 && bl_occludes == 1) {
				occlusion_level = 3;
			}

			return occlusion_level;
		};

		let face_occlusion_data = |block_pos: BlockPos| {
			let (x, y) = visit_map.get_index(block_pos);
			let x = x as i32;
			let y = y as i32;
			OcclusionCorners {
				tl: vertex_occlusion_level(x, y + 1),
				tr: vertex_occlusion_level(x + 1, y + 1),
				bl: vertex_occlusion_level(x, y),
				br: vertex_occlusion_level(x + 1, y),
			}
		};

		for x in 0..CHUNK_SIZE as i32 {
			let mut y = 0;
			while y < CHUNK_SIZE as i32 {
				let block_pos = visit_map.get_block_pos(x, y);

				let block = self.get_block(block_pos);
				if block.is_air() {
					y += 1;
					continue;
				} else if let Some(is_translucent) = self.with_block(block_pos + face_offset, |block| block.is_translucent()) {
					if !is_translucent {
						y += 1;
						continue;
					}
				} else {
					y += 1;
					continue;
				}

				let block_type = block.block_type();

				// width and height of the greedy mesh region
				let mut width = 1;

				let occlusion_corners = face_occlusion_data(block_pos);

				// to be growable, the ambient occlusion level of the vertext on each respective x level has to be the same
				let growable = occlusion_corners.tl == occlusion_corners.bl && occlusion_corners.tr == occlusion_corners.br;

				if growable {
					loop {
						let current_block_pos = visit_map.get_block_pos_offset(block_pos, 0, width);
						// get_block_pos_offset could put current block out of bounds
						if !current_block_pos.is_chunk_local() {
							break;
						}

						if let Some(is_translucent) = self.with_block(block_pos + face_offset, |block| block.is_translucent()) {
							if is_translucent && self.get_block(current_block_pos).block_type() == block_type {
								// TODO: don't need to calculate all occlusion corners, only 2
								let occlusion_corners_new = face_occlusion_data(current_block_pos);
								if occlusion_corners_new.tl == occlusion_corners.tl && occlusion_corners_new.tr == occlusion_corners.tr {
									width += 1;
								} else {
									break;
								}
							} else {
								break;
							}
						} else {
							break;
						}
					}
				}

				let block_face_mesh = BlockFaceMesh::from_cube_corners(
					face,
					block.texture_index().unwrap(),
					block_pos + self.block_position,
					visit_map.get_block_pos_offset(block_pos, 0, width - 1) + self.block_position,
					occlusion_corners,
				);
	
				chunk_mesh[Into::<usize>::into(face)][index].push(block_face_mesh);

				y += width;
			}
		}
	}

	// updates the mesh for the entire chunk
	pub fn chunk_mesh_update(&self) {
		let mut visit_map = VisitedBlockMap::new();

		for face in BlockFace::iter() {
			for i in 0..CHUNK_SIZE {
				self.mesh_update_inner(face, i, &mut visit_map);
			}
		}
	}

	// returns None if the mesh is currently locked, which means it is being generated,
	// so we wouldn't have to display it anyways
	pub fn get_chunk_mesh(&self) -> Option<Vec<BlockFaceMesh>> {
		Some(self.chunk_mesh.try_read()?.iter()
			.flatten()
			.flatten()
			.copied()
			.collect::<Vec<_>>())
	}
}

pub struct LoadedChunk {
	pub chunk: Chunk,
	pub load_count: AtomicU64,
}

impl LoadedChunk {
	pub fn new(chunk: Chunk) -> LoadedChunk {
		//let chunk_mesh = chunk.generate_block_faces();
		LoadedChunk {
			chunk,
			load_count: AtomicU64::new(0),
		}
	}

	pub fn inc_load_count(&self) {
		self.load_count.fetch_add(1, Ordering::AcqRel);
	}

	pub fn dec_load_count(&self) -> u64 {
		self.load_count.fetch_sub(1, Ordering::AcqRel) - 1
	}

	pub fn get_load_count(&self) -> u64 {
		self.load_count.load(Ordering::Acquire)
	}
}

// the entire saved state of the chunk, which is all blocks and entities
// TODO: maybe save chunk mesh to load faster
pub struct ChunkData {
	chunk: Chunk,
	entities: Vec<Box<dyn Entity>>,
}
