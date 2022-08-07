use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Plane {
    pub position: Vec3,
    pub normal: Vec3,
}

impl Plane {
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Plane {
            position,
            normal,
        }
    }
}