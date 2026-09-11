use super::Vector3;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Triangle3 {
    pub(crate) a: Vector3,
    pub(crate) b: Vector3,
    pub(crate) c: Vector3,
}

impl From<(Vector3, Vector3, Vector3)> for Triangle3 {
    fn from((a, b, c): (Vector3, Vector3, Vector3)) -> Self {
        Self { a, b, c }
    }
}

impl Triangle3 {
    pub fn normal(&self) -> Vector3 {
        let side_1 = self.b - self.a;
        let side_2 = self.c - self.a;

        side_1.cross(side_2)
    }
}
