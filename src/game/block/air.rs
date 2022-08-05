use super::*;

#[derive(Debug, Clone)]
pub struct Air {}

impl Air {
	pub fn new() -> Air {
		Air {}
	}
}

impl BlockTrait for Air {
	fn name(&self) -> &str {
	    "air"
	}

	fn is_translucent(&self) -> bool {
		true
	}
}
