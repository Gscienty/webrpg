use super::Vector3;

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Sphere3 {
    pub center: Vector3,
    pub radius: f32,
}
