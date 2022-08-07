use glam::Vec3;

use crate::math::Plane;

/// Axis aligned bounding boss
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub position: Vec3,
    pub size: Vec3,
}

impl Aabb {
    pub fn new(position: Vec3, size: Vec3) -> Self {
        Aabb {
            position,
            size,
        }
    }

    pub fn end(&self) -> Vec3 {
        self.position + self.size
    }

    /// Returns true if any part of the boundinf box lies inside of the plane (on the side that the normal is pointong to)
    pub fn inside_of_plane(&self, plane: Plane) -> bool {
        let rel_pos = self.position - plane.position;

        // find which corner of the bounding box to use
        // it will be the corner that the plane's normal vector is pointing towards
        // use vector functions to do this because if vec3a is used it will use simd and be very fast
        let mask = plane.normal.cmpge(Vec3::ZERO);
        let corner_offset = Vec3::select(mask, self.size, Vec3::ZERO);
        let corner = rel_pos + corner_offset;

        corner.dot(plane.normal) >= 0.0
    }
}