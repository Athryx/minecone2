use super::*;

#[derive(Debug, Clone)]
pub struct RockyDirt {}

impl RockyDirt {
	pub fn new() -> RockyDirt {
		RockyDirt {}
	}

	pub fn get_texture() -> Result<DynamicImage> {
		Ok(texmanip::tile_from_side(&loader().load_image("textures/rocky-dirt.png")?))
	}
}

impl BlockTrait for RockyDirt {
	fn name(&self) -> &str {
		"rocky dirt"
	}

	fn is_translucent(&self) -> bool {
		false
	}
}
