use std::sync::atomic::{AtomicU64, Ordering};

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerId(u64);

static NEXT_PLAYER_ID: AtomicU64 = AtomicU64::new(0);

impl PlayerId {
	// returns a unique player id
	pub fn new() -> PlayerId {
		PlayerId(NEXT_PLAYER_ID.fetch_add(1, Ordering::Relaxed))
	}
}

pub struct Player {
	id: PlayerId,
	pub position: Position,
	// render distance in x, y, and z direction
	render_distance: ChunkPos,
}

impl Player {
	pub fn new() -> Player {
		Player {
			id: PlayerId::new(),
			position: Position::new(0.0, 0.0, 0.0),
			render_distance: ChunkPos::new(10, 5, 10),
			//render_distance: ChunkPos::new(2, 2, 2),
			//render_distance: ChunkPos::new(5, 5, 3),
		}
	}

	pub fn id(&self) -> PlayerId {
		self.id
	}

	pub fn chunk_position(&self) -> ChunkPos {
		self.position.into()
	}

	pub fn render_distance(&self) -> ChunkPos {
		self.render_distance
	}
}
