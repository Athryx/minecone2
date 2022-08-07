use std::hash::BuildHasherDefault;
use glam::{Vec3, IVec3};

pub use anyhow::Result;
use rustc_hash::FxHasher;
use dashmap::DashMap;

pub use crate::game::{CHUNK_SIZE, types::*, debug_string, debug_display};

pub type FxDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;

pub trait VecExt<T> {
    fn map<F: FnMut(T) -> T>(&self, f: F) -> Self;

    fn axis(axis: Axis) -> Self;
}

impl VecExt<f32> for Vec3 {
    fn map<F: FnMut(f32) -> f32>(&self, mut f: F) -> Self {
        Self::new(f(self.x), f(self.y), f(self.z))
    }

    fn axis(axis: Axis) -> Self {
        match axis {
            Axis::X => Self::X,
            Axis::Y => Self::Y,
            Axis::Z => Self::Z,
        }
    }
}

impl VecExt<i32> for IVec3 {
    fn map<F: FnMut(i32) -> i32>(&self, mut f: F) -> Self {
        Self::new(f(self.x), f(self.y), f(self.z))
    }

    fn axis(axis: Axis) -> Self {
        match axis {
            Axis::X => Self::X,
            Axis::Y => Self::Y,
            Axis::Z => Self::Z,
        }
    }
}

#[macro_export]
macro_rules! vec3_map {
    ($f:expr, $out_vec:ident, $( $vecs:ident ),+) => {
        $out_vec::new($f(
            $(
                $vecs.x,
            )*
        ),
        $f(
            $(
                $vecs.y,
            )*
        ),
        $f(
            $(
                $vecs.z,
            )*
        ))
    };
}