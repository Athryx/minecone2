use std::collections::hash_set::Iter;

use rustc_hash::FxHashSet;

use crate::prelude::*;

pub const RENDER_ZONE_SIZE: i32 = 8;

pub struct UpdatedRenderZones(FxHashSet<ChunkPos>);

impl UpdatedRenderZones {
    pub fn new() -> Self {
        UpdatedRenderZones(FxHashSet::default())
    }

    fn get_render_zone_of_chunk(chunk: ChunkPos) -> ChunkPos {
		ChunkPos(RENDER_ZONE_SIZE * chunk.map(|elem| {
			if elem >= 0 {
				elem / RENDER_ZONE_SIZE
			} else {
				(elem - RENDER_ZONE_SIZE + 1) / RENDER_ZONE_SIZE
			}
		}))
	}

    pub fn mark_block(&mut self, block: BlockPos) {
        self.mark_chunk(block.into());
    }

    pub fn mark_chunk(&mut self, chunk: ChunkPos) {
        self.0.insert(Self::get_render_zone_of_chunk(chunk));
    }

    pub fn mark_chunk_zone(&mut self, min_chunk: ChunkPos, max_chunk: ChunkPos) {
        let min_render_zone = Self::get_render_zone_of_chunk(min_chunk);
		let max_render_zone = Self::get_render_zone_of_chunk(max_chunk - ChunkPos::splat(1));

		for x in (min_render_zone.x..=max_render_zone.x).step_by(RENDER_ZONE_SIZE as usize) {
			for y in (min_render_zone.y..=max_render_zone.y).step_by(RENDER_ZONE_SIZE as usize) {
				for z in (min_render_zone.z..=max_render_zone.z).step_by(RENDER_ZONE_SIZE as usize) {
					self.0.insert(ChunkPos::new(x, y, z));
				}
			}
		}
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> Iter<ChunkPos> {
        self.0.iter()
    }
}