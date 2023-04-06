use std::sync::LazyLock;

use array_init::array_init;

use crate::prelude::*;
use crate::game::block::*;

use super::biome::BiomeNoiseData;

#[derive(Debug)]
pub struct SurfaceLayer {
	block: Block,
	thickness: u64,
}

#[derive(Debug)]
pub struct SurfaceBiome {
	pub name: String,
	// the amplitude of the height noise
	pub height_amplitude: f64,
	// these are the layers on top of the surface biome
	pub layers: Vec<SurfaceLayer>,
	// this is the filler block down to the bottom of the surface layer
	pub filler: Block,
	pub heat_point: u8,
	pub humidity_point: u8,
}

impl SurfaceBiome {
	pub fn get_block_at_depth(&self, depth: i32) -> Block {
		if depth > 0 {
			return Air::new().into();
		}

		let mut current_block_depth: i32 = 0;
		for layer in self.layers.iter() {
			let thickness: i32 = layer.thickness.try_into().unwrap();
			current_block_depth -= thickness;
			if depth > current_block_depth {
				return layer.block.clone();
			}
		}

		self.filler.clone()
	}
}

static BIOMES: LazyLock<[SurfaceBiome; 3]> = LazyLock::new(|| [
	SurfaceBiome {
		name: "grasslands".to_owned(),
		height_amplitude: 4.0,
		layers: vec![
			SurfaceLayer {
				block: Grass::new().into(),
				thickness: 1,
			},
			SurfaceLayer {
				block: Dirt::new().into(),
				thickness: 3,
			},
			SurfaceLayer {
				block: RockyDirt::new().into(),
				thickness: 3,
			},
		],
		filler: Stone::new().into(),
		heat_point: 28,
		humidity_point: 18,
	},
	SurfaceBiome {
		name: "lush grasslands".to_owned(),
		height_amplitude: 4.0,
		layers: vec![
			SurfaceLayer {
				block: Dirt::new().into(),
				thickness: 1,
			},
			SurfaceLayer {
				block: Dirt::new().into(),
				thickness: 3,
			},
			SurfaceLayer {
				block: RockyDirt::new().into(),
				thickness: 3,
			},
		],
		filler: Stone::new().into(),
		heat_point: 28,
		humidity_point: 25,
	},
	SurfaceBiome {
		name: "coniferous forest".to_owned(),
		height_amplitude: 50.0,
		layers: vec![
			SurfaceLayer {
				block: Stone::new().into(),
				thickness: 1,
			},
			SurfaceLayer {
				block: Dirt::new().into(),
				thickness: 3,
			},
			SurfaceLayer {
				block: RockyDirt::new().into(),
				thickness: 3,
			},
		],
		filler: Stone::new().into(),
		heat_point: 13,
		humidity_point: 35,
	},
]);

pub const BIOME_MAP_SIZE: usize = 50;

#[derive(Debug)]
pub struct SurfaceBiomeMap {
	// put big array in box to avoid stack overflow
	map: Box<[[&'static SurfaceBiome; BIOME_MAP_SIZE]; BIOME_MAP_SIZE]>,
}

impl SurfaceBiomeMap {
	// makes a varioni diagram using the heat and humidity points of all the biomes
	pub fn new() -> Self {
		// this is a really slow, lazy way to do it but this is onle being run once so it doesn't matter
		SurfaceBiomeMap {
			map: Box::new(array_init(|heat| array_init(|humidity| {
				let mut min_distance = f64::INFINITY;
				let mut closest_biome = None;
				for biome in BIOMES.iter() {
					let heat_diff = biome.heat_point as f64 - heat as f64;
					let humidity_diff = biome.humidity_point as f64 - humidity as f64;
					let distance = (heat_diff * heat_diff + humidity_diff * humidity_diff).sqrt();

					if distance < min_distance {
						min_distance = distance;
						closest_biome = Some(biome);
					}
				}

				closest_biome.unwrap()
			})))
		}
	}

	pub fn get_biome(&self, noise: BiomeNoiseData) -> &'static SurfaceBiome {
		self.map[noise.heat as usize][noise.humidity as usize]
	}

	pub fn print_diagram(&self) {
		let mut out_str = String::from("");
		for heat in 0..50 {
			for humidity in 0..50 {
				out_str.push_str(&self.map[heat][humidity].name[0..1]);
			}
			out_str.push('\n');
		}
		println!("{}", out_str);
		println!();
		println!();
		println!();
		println!();
		println!();
	}
}
