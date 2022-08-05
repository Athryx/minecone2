use crate::prelude::*;
use crate::game::block::{Block, BlockTrait, Air, Grass, Dirt, RockyDirt, Stone};

#[derive(Debug, Clone, Copy)]
pub struct BiomeNoiseData {
	// there will be 16 different heat and humidity levels used to determine the biome type
	pub heat: u8,
	pub humidity: u8,
}

// the first index is heat, the second is humidity
const BIOME_MAP: [[SurfaceBiome; 16]; 16] = {
	use SurfaceBiome::*;
	// colder
	[
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Arctic, Arctic, Arctic],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Arctic, Arctic, Arctic],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Arctic, Arctic, Arctic],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, LushGrasslands, LushGrasslands, LushGrasslands, FloodedGrasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, LushGrasslands, LushGrasslands, LushGrasslands, FloodedGrasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, LushGrasslands, LushGrasslands, LushGrasslands, FloodedGrasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands],
		[Desert, Desert, Desert, XericShrubland, XericShrubland, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Jungle, Jungle, Jungle, Swamp, Swamp],
		[Desert, Desert, Desert, XericShrubland, XericShrubland, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Jungle, Jungle, Jungle, Swamp, Swamp],
		[Desert, Desert, Desert, XericShrubland, XericShrubland, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Grasslands, Jungle, Jungle, Jungle, Swamp, Swamp],
		// drier																																												wetter
	]
	// hotter
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceBiome {
	// oak an birch trees grow here
	Grasslands,
	// grasslands with far more frequent vegetation and shrubs
	LushGrasslands,
	// coniferous tree is a tree like a pine tree
	ConiferousForest,
	// forest of other trees
	BroadleafForest,
	// very moist forest
	Jungle,
	Swamp,
	// like a swamp but with no trees
	FloodedGrasslands,
	// dry, cold, little vegetation
	Tundra,
	// ice and snow evrywhere
	Arctic,
	// a cold forest
	Taiga,
	Desert,
	// like a desert but with more shrubs growing and vegetation
	XericShrubland,
	// savanna with very few trees, mostly grass
	SavannaGrassland,
	// savanna with much more frequent trees
	SavannaWoodland,
}

impl SurfaceBiome {
	pub fn new(biome_noise: BiomeNoiseData) -> Self {
		BIOME_MAP[biome_noise.heat as usize][biome_noise.humidity as usize]
	}

	pub fn height_amplitude(&self) -> f64 {
		match self {
			Self::Grasslands => 4.0,
			_ => 1.0,
		}
	}

	// depth is negative for blocks below the surface, and 0 at the surface
	// returns none if the depth is too deep and it is not apart of this biome
	pub fn get_block_at_depth(&self, depth: i32) -> Block {
		if depth > 0 {
			Air::new().into()
		} else {
			match self {
				Self::Grasslands => {
					// TODO: make a macro for this
					if depth == 0 {
						Grass::new().into()
					} else if depth > -3 {
						Dirt::new().into()
					} else if depth > -6 {
						RockyDirt::new().into()
					} else {
						Stone::new().into()
					}
				},
				_ => Stone::new().into(),
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountainBiome {
	SnowyPeaks,
	BarrenPeaks,
	MontaneForest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeaBiome {
	Sea,
	FrozenSea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndergroundBiome {
	SolidGround,
	Caverns,
	LushCaverns,
	UndergroundLake,
	FloodedCaverns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderworldBiome {
}
