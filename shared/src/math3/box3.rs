use super::Vector3;

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Box3 {
    pub(crate) min: Vector3,
    pub(crate) max: Vector3,
}

impl From<(Vector3, Vector3)> for Box3 {
    fn from((min, max): (Vector3, Vector3)) -> Self {
        Self { min, max }
    }
}

impl Box3 {
    pub fn center(&self) -> Vector3 {
        (self.min + self.max) * 0.5
    }
}
