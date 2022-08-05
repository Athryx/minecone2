use super::*;

#[derive(Debug, Clone)]
pub struct Dirt {}

impl Dirt {
	pub fn new() -> Dirt {
		Dirt {}
	}

	pub fn get_texture() -> Result<DynamicImage> {
		Ok(texmanip::tile_from_side(&loader().load_image("textures/dirt.png")?))
	}
}

impl BlockTrait for Dirt {
	fn name(&self) -> &str {
		"dirt"
	}

	fn is_translucent(&self) -> bool {
		false
	}
}
