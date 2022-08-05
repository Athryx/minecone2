use std::sync::Arc;

use noise::{Seedable, NoiseFn, OpenSimplex};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use glam::{IVec2, Vec3Swizzles};
use statrs::function::erf::erf;

use crate::prelude::*;
use biome::{SurfaceBiome, BiomeNoiseData};
use surface_biome::SurfaceBiomeMap;
use super::chunk::{Chunk, LoadedChunk};
use super::world::World;
use super::block::*;

mod biome;
mod surface_biome;

type Cache2D = FxHashMap<IVec2, f64>;
type Cache3D = FxHashMap<BlockPos, f64>;

#[derive(Debug, Default)]
struct NoiseCache {
	height_noise: Cache2D,
	biome_height_noise: Cache2D,
	biome_heat_noise: Cache2D,
	biome_humidity_noise: Cache2D,
}

struct CachedNoise2D {
	noise: OpenSimplex,
	scale: f64,
	amplitude_fn: fn(f64) -> f64,
}

impl CachedNoise2D {
	fn new(seed: u32, scale: f64) -> Self {
		Self {
			noise: OpenSimplex::new().set_seed(seed),
			scale,
			amplitude_fn: |value| value,
		}
	}

	fn new_amplitude_scaled(seed: u32, scale: f64, amplitude_fn: fn(f64) -> f64) -> Self {
		Self {
			noise: OpenSimplex::new().set_seed(seed),
			scale,
			amplitude_fn,
		}
	}

	fn get_block_pos(&self, block: BlockPos, cache: &mut Cache2D) -> f64 {
		*cache.entry(block.0.xz()).or_insert_with(||
			(self.amplitude_fn)(self.noise.get([block.x as f64 * self.scale, block.z as f64 * self.scale])))
	}
}

struct CachedNoise3D {
	noise: OpenSimplex,
	scale: f64,
	amplitude_fn: fn(f64) -> f64,
}

impl CachedNoise3D {
	fn new(seed: u32, scale: f64) -> Self {
		Self {
			noise: OpenSimplex::new().set_seed(seed),
			scale,
			amplitude_fn: |value| value,
		}
	}

	fn new_amplitude_scaled(seed: u32, scale: f64, amplitude_fn: fn(f64) -> f64) -> Self {
		Self {
			noise: OpenSimplex::new().set_seed(seed),
			scale,
			amplitude_fn,
		}
	}

	fn get_block_pos(&self, block: BlockPos, cache: &mut Cache3D) -> f64 {
		*cache.entry(block).or_insert_with(||
			(self.amplitude_fn)(self.noise.get([block.x as f64 * self.scale, block.y as f64 * self.scale, block.z as f64 * self.scale])))
	}
}

pub struct WorldGenerator {
	height_noise: CachedNoise2D,
	biome_height_noise: CachedNoise2D,
	biome_heat_noise: CachedNoise2D,
	biome_humidity_noise: CachedNoise2D,
	surface_biome_map: SurfaceBiomeMap,
}

impl WorldGenerator {
	pub fn new(seed: u32) -> Self {
		// TODO: this doesn't make it completely uniform, could be better
		let biome_make_uniform = |value: f64| {
			// the varience of opensimplex is about this
			let varience: f64 = 0.0463;
			let uniform = erf(value / (2.0 * varience).sqrt());
			(25.0 + 25.0 * uniform).clamp(0.0, 49.0)
			/*(25.0 + 55.0 * value).clamp(0.0, 49.0) as u8*/
		};

		WorldGenerator {
			height_noise: CachedNoise2D::new(seed, 0.05),
			biome_height_noise: CachedNoise2D::new(seed + 1, 0.002),
			biome_heat_noise: CachedNoise2D::new_amplitude_scaled(seed + 2, 0.002, biome_make_uniform),
			biome_humidity_noise: CachedNoise2D::new_amplitude_scaled(seed + 3, 0.002, biome_make_uniform),
			surface_biome_map: SurfaceBiomeMap::new(),
		}
	}

	fn get_height_noise(&self, block: BlockPos, amplitude: f64, cache: &mut NoiseCache) -> i32 {
		(amplitude * self.height_noise.get_block_pos(block, &mut cache.height_noise)) as i32
	}

	fn get_biome_height_noise(&self, block: BlockPos, cache: &mut NoiseCache) -> i32 {
		let noise = 6.0 * self.biome_height_noise.get_block_pos(block, &mut cache.biome_height_noise);
		(noise * noise * noise) as i32
	}

	fn get_biome_noise(&self, block: BlockPos, cache: &mut NoiseCache) -> BiomeNoiseData {
		// TODO: this doesn't make it completely uniform, could be better
		let make_uniform = |value: f64| {
			// the varience of opensimplex is about this
			let varience: f64 = 0.0463;
			let uniform = erf(value / (2.0 * varience).sqrt());
			(25.0 + 25.0 * uniform).clamp(0.0, 49.0) as u8
			/*(25.0 + 55.0 * value).clamp(0.0, 49.0) as u8*/
		};
		let heat = self.biome_heat_noise.get_block_pos(block, &mut cache.biome_heat_noise) as u8;
		let humidity = self.biome_humidity_noise.get_block_pos(block, &mut cache.biome_humidity_noise) as u8;
		BiomeNoiseData {
			heat,
			humidity,
		}
	}

	pub fn generate_chunk(&self, world: Arc<World>, position: ChunkPos) -> LoadedChunk {
		let mut cache = NoiseCache::default();
		LoadedChunk::new(Chunk::new(world, position, |block| {
			let biome_height = self.get_biome_height_noise(block, &mut cache);
			let biome_noise = self.get_biome_noise(block, &mut cache);

			let biome = self.surface_biome_map.get_biome(biome_noise);

			let height = self.get_height_noise(block, biome.height_amplitude, &mut cache);

			biome.get_block_at_depth(block.y - height)
		}))
	}
}
