use std::ops::{Mul, Index, IndexMut};

use glam::{Vec3, IVec3};
use derive_more::{Deref, DerefMut, Add, Sub, Mul, Div};

use super::{CHUNK_SIZE, BlockFace};

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Index<Axis> for Vec3 {
    type Output = f32;

    fn index(&self, index: Axis) -> &Self::Output {
        &self[index as usize]
    }
}

impl IndexMut<Axis> for Vec3 {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl Index<Axis> for IVec3 {
    type Output = i32;

    fn index(&self, index: Axis) -> &Self::Output {
        &self[index as usize]
    }
}

impl IndexMut<Axis> for IVec3 {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

macro_rules! only_methods {
    ($vec:ident, $zero:expr) => {
        impl $vec {
            pub fn xonly(&self) -> Self {
                Self::new(self.x, $zero, $zero)
            }

            pub fn yonly(&self) -> Self {
                Self::new($zero, self.y, $zero)
            }

            pub fn zonly(&self) -> Self {
                Self::new($zero, $zero, self.z)
            }

            pub fn xyonly(&self) -> Self {
                Self::new(self.x, self.y, $zero)
            }

            pub fn yzonly(&self) -> Self {
                Self::new($zero, self.y, self.z)
            }

            pub fn xzonly(&self) -> Self {
                Self::new(self.x, $zero, self.z)
            }

            pub fn axis_only(&self, axis: Axis) -> Self {
                let mut out = Self::new($zero, $zero, $zero);
                out[axis] = self[axis];
                out
            }

            pub fn all_but_axis(&self, axis: Axis) -> Self {
                let mut out = *self;
                out[axis] = $zero;
                out
            }
        }
    };
}

/// Position of a chunk in chunk coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Add, Sub, Mul, Div)]
pub struct ChunkPos(pub IVec3);

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        ChunkPos(IVec3::new(x, y, z))
    }

    pub fn as_block_pos(self) -> BlockPos {
        self.into()
    }

    pub fn as_position(self) -> Position {
        self.into()
    }

    pub fn length(&self) -> f32 {
        self.0.as_vec3().length()
    }
}

only_methods!(ChunkPos, 0);

impl Mul<ChunkPos> for i32 {
    type Output = ChunkPos;

    fn mul(self, rhs: ChunkPos) -> Self::Output {
        rhs * self
    }
}

impl From<BlockPos> for ChunkPos {
    fn from(block_pos: BlockPos) -> Self {
        ChunkPos(IVec3::from_array(block_pos.0.to_array().map(|elem| {
            if elem >= 0 {
                elem / CHUNK_SIZE as i32
            } else {
                (elem - (CHUNK_SIZE as i32 - 1)) / CHUNK_SIZE as i32
            }
        })))
    }
}

impl From<Position> for ChunkPos {
    fn from(position: Position) -> Self {
        let block_pos: BlockPos = position.into();
        block_pos.into()
    }
}

/// Position of a block in chunk space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut, Add, Sub, Mul, Div)]
pub struct BlockPos(pub IVec3);

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        BlockPos(IVec3::new(x, y, z))
    }

    pub fn is_chunk_local(&self) -> bool {
        const CSIZE: i32 = CHUNK_SIZE as i32;
        self.x >= 0 && self.x < CSIZE
            && self.y >= 0 && self.y < CSIZE
            && self.z >= 0 && self.z < CSIZE
    }

    pub fn as_chunk_pos(self) -> ChunkPos {
        self.into()
    }

    pub fn as_position(self) -> Position {
        self.into()
    }

    pub fn as_chunk_local(&self) -> Self {
        BlockPos(IVec3::from_array(self.0.to_array().map(|elem| {
            if elem >= 0 {
                elem % CHUNK_SIZE as i32
            } else {
                CHUNK_SIZE as i32 + ((elem + 1) % CHUNK_SIZE as i32) - 1
            }
        })))
    }

    pub fn as_chunk_block_pos(self) -> (ChunkPos, BlockPos) {
        (self.into(), self.as_chunk_local())
    }

    pub fn as_indicies(&self) -> Option<(usize, usize, usize)> {
        let x = self.x.try_into().ok()?;
		let y = self.y.try_into().ok()?;
		let z = self.z.try_into().ok()?;
        Some((x, y, z))
    }

    pub fn get_face_component(&self, face: BlockFace) -> i32 {
        match face {
			BlockFace::XPos | BlockFace::XNeg => self.x,
			BlockFace::YPos | BlockFace::YNeg => self.y,
			BlockFace::ZPos | BlockFace::ZNeg => self.z,
		}
    }

    pub fn length(&self) -> f32 {
        self.0.as_vec3().length()
    }
}

only_methods!(BlockPos, 0);

impl Mul<BlockPos> for i32 {
    type Output = BlockPos;

    fn mul(self, rhs: BlockPos) -> Self::Output {
        rhs * self
    }
}

impl From<ChunkPos> for BlockPos {
    fn from(chunk_pos: ChunkPos) -> Self {
        BlockPos(chunk_pos.0 * CHUNK_SIZE as i32)
    }
}

impl From<Position> for BlockPos {
    fn from(position: Position) -> Self {
        BlockPos(position.0.floor().as_ivec3())
    }
}

/// Position of an entity or anything that does not use whole number positions
#[derive(Debug, Clone, Copy, PartialEq, Deref, DerefMut, Add, Sub, Mul, Div)]
pub struct Position(pub Vec3);

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Position(Vec3::new(x, y, z))
    }

    pub fn as_block_pos(self) -> BlockPos {
        self.into()
    }

    pub fn as_chunk_pos(self) -> ChunkPos {
        self.into()
    }
}

only_methods!(Position, 0.0);

impl Mul<Position> for f32 {
    type Output = Position;

    fn mul(self, rhs: Position) -> Self::Output {
        rhs * self
    }
}

impl From<ChunkPos> for Position {
    fn from(chunk_pos: ChunkPos) -> Self {
        Position(chunk_pos.as_block_pos().0.as_vec3())
    }
}

impl From<BlockPos> for Position {
    fn from(block_pos: BlockPos) -> Self {
        Position(block_pos.0.as_vec3())
    }
}