mod player;
pub use player::*;

pub trait Entity: Send + Sync {}
