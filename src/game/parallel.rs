use std::sync::LazyLock;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam::{
	deque::{Injector, Steal},
	queue::SegQueue,
};

use crate::prelude::*;
use super::{world::World, BlockFace};
use super::chunk::{VisitedBlockMap, CHUNK_SIZE};

static TASK_QUEUE: LazyLock<Injector<Task>> = LazyLock::new(|| Injector::new());
static COMPLETED_TASKS: SegQueue<Task> = SegQueue::new();

// TODO: allow easy way of chaining tasks
#[derive(Debug, Clone)]
pub enum Task {
	// generate a mesh for the given chunk
	ChunkMesh(ChunkPos),
	ChunkMeshFace {
		min_chunk: ChunkPos,
		max_chunk: ChunkPos,
		face: BlockFace,
	},
	// use world generate to generate chunk
	GenerateChunk(ChunkPos),
	UnloadChunks {
		min_chunk: ChunkPos,
		max_chunk: ChunkPos,
	},
}

pub fn init(world: Arc<World>, num_tasks: usize) {
	info!("runing with {} task processing threads", num_tasks);
	for _ in 0..num_tasks {
		let thread_world = world.clone();
		thread::spawn(move || task_runner(thread_world));
	}
}

// appends the given task to the task queue
pub fn run_task(task: Task) {
	TASK_QUEUE.push(task);
}

pub fn pull_completed_task() -> Option<Task> {
	COMPLETED_TASKS.pop()
}

// waits for a task to apear, than runs it
fn task_runner(world: Arc<World>) {
	let sleep_duration = Duration::from_millis(2);

	loop {
		match TASK_QUEUE.steal() {
			Steal::Success(task) => execute_task(&world, task),
			Steal::Empty => thread::sleep(sleep_duration),
			Steal::Retry => continue,
		}
	}
}

// executes the given task
fn execute_task(world: &Arc<World>, task: Task) {
	match task {
		Task::ChunkMesh(chunk) => {
			world.chunks.get(&chunk).map(|chunk| chunk.value().chunk.chunk_mesh_update());
			COMPLETED_TASKS.push(task);
		},
		Task::ChunkMeshFace { face, min_chunk, max_chunk } => {
			let mut visit_map = VisitedBlockMap::new();

			for x in min_chunk.x..max_chunk.x {
				for y in min_chunk.y..max_chunk.y {
					for z in min_chunk.z..max_chunk.z {
						let chunk_pos = ChunkPos::new(x, y, z);
						if let Some(chunk) = world.chunks.get(&chunk_pos) {
							let index = if face.is_positive_face() {
								CHUNK_SIZE - 1
							} else {
								0
							};

							chunk.chunk.mesh_update_inner(face, index, &mut visit_map);
						}
					}
				}
			}

			COMPLETED_TASKS.push(task);
		},
		Task::GenerateChunk(chunk) => {
			let chunk = world.chunks.entry(chunk)
				.or_insert_with(|| world.world_generator
					.generate_chunk(world.clone(), chunk));

			// when first inserting load count starts at 0
			chunk.inc_load_count();

			COMPLETED_TASKS.push(task);
		},
		Task::UnloadChunks { min_chunk, max_chunk } => {
			for x in min_chunk.x..max_chunk.x {
				for y in min_chunk.y..max_chunk.y {
					for z in min_chunk.z..max_chunk.z {
						let position = ChunkPos::new(x, y, z);

						if let Some(loaded_chunk) = world.chunks.get(&position) {
							if loaded_chunk.dec_load_count() == 0 {
								drop(loaded_chunk);
								world.chunks.remove(&position);
							}
						}
					}
				}
			}

			COMPLETED_TASKS.push(task);
		},
	}
}
