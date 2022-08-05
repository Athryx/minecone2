use super::*;
use texmanip::*;

#[derive(Debug, Clone)]
pub struct Grass {}

impl Grass {
	pub fn new() -> Grass {
		Grass {}
	}

	pub fn get_texture() -> Result<DynamicImage> {
		let dirt = loader().load_image("textures/dirt.png")?;
		let grass_top = loader().load_image("textures/grass-top.png")?;
		let grass_side = loader().load_image("textures/grass-side.png")?;

		let mut tiled_dirt = tile_from_side(&dirt);
		let tiled_grass = stitch_texture(TextureStitchFaces {
			front: TextureFace::BlockAligned(&grass_side),
			back: TextureFace::BlockAligned(&grass_side),
			left: TextureFace::BlockAligned(&grass_side),
			right: TextureFace::BlockAligned(&grass_side),
			top: TextureFace::BlockAligned(&grass_top),
			bottom: TextureFace::Empty,
		})?;
		
		overlay(&mut tiled_dirt, &tiled_grass, 0, 0);
		Ok(tiled_dirt)
	}
}

impl BlockTrait for Grass {
	fn name(&self) -> &str {
		"grass"
	}

	fn is_translucent(&self) -> bool {
		false
	}
}
