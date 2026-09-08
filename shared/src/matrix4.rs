use crate::Vector4;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Matrix4 {
    pub(crate) v: [f32; 16],
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl From<[f32; 16]> for Matrix4 {
    fn from(v: [f32; 16]) -> Self {
        Self { v }
    }
}

impl Matrix4 {
    pub const fn identity() -> Self {
        Self {
            v: [
                1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
            ],
        }
    }

    pub fn column(self, index: usize) -> Vector4 {
        let i = index * 4;
        Vector4::from((self.v[i], self.v[i + 1], self.v[i + 2], self.v[i + 3]))
    }
}
