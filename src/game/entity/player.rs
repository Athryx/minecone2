use super::*;
use crate::prelude::*;

pub struct Player {
	position: Position,
}

impl Player {
	pub fn new(position: Position) -> Box<dyn Entity> {
		Box::new(Player {
			position
		})
	}
}

impl Entity for Player {
}
