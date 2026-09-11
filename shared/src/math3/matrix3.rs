use std::ops::{Index, IndexMut};

use super::Vector3;

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Matrix3 {
    pub(crate) v: [Vector3; 3],
}

impl Index<usize> for Matrix3 {
    type Output = Vector3;

    fn index(&self, index: usize) -> &Self::Output {
        &self.v[index]
    }
}

impl IndexMut<usize> for Matrix3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.v[index]
    }
}
