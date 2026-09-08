use crate::Vector3;

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Matrix3 {
    pub(crate) v: [f32; 9],
}

impl From<[f32; 9]> for Matrix3 {
    fn from(v: [f32; 9]) -> Self {
        Self { v }
    }
}

impl Matrix3 {
    pub fn column(self, index: usize) -> Vector3 {
        let offset = index * 3;
        Vector3::from((self.v[offset], self.v[offset + 1], self.v[offset + 2]))
    }
}
