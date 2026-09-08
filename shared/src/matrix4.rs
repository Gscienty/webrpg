use std::ops::{Index, IndexMut, Mul};

use crate::{Quaternion, RotationOrder, Vector3, Vector4};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Matrix4 {
    pub(crate) columns: [Vector4; 4],
}

impl Default for Matrix4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Index<usize> for Matrix4 {
    type Output = Vector4;

    fn index(&self, index: usize) -> &Self::Output {
        &self.columns[index]
    }
}

impl IndexMut<usize> for Matrix4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.columns[index]
    }
}

impl Mul<Matrix4> for Matrix4 {
    type Output = Matrix4;

    fn mul(self, rhs: Matrix4) -> Self::Output {
        let mut result = Self {
            columns: [Vector4::default(); 4],
        };
        for column in 0..4 {
            for row in 0..4 {
                result[column][row] = (0..4).map(|i| self[i][row] * rhs[column][i]).sum();
            }
        }
        result
    }
}

impl Mul<Vector3> for Matrix4 {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Self::Output {
        let s = 1. / (self[0][3] * rhs[0] + self[1][3] * rhs[1] + self[2][3] * rhs[2] + self[3][3]);
        Vector3::from((
            (self[0][0] * rhs[0] + self[1][0] * rhs[1] + self[2][0] * rhs[2] + self[3][0]) * s,
            (self[0][1] * rhs[0] + self[1][1] * rhs[1] + self[2][1] * rhs[2] + self[3][1]) * s,
            (self[0][2] * rhs[0] + self[1][2] * rhs[1] + self[2][2] * rhs[2] + self[3][2]) * s,
        ))
    }
}

impl Matrix4 {
    pub fn identity() -> Self {
        Self {
            columns: [
                Vector4::from((1., 0., 0., 0.)),
                Vector4::from((0., 1., 0., 0.)),
                Vector4::from((0., 0., 1., 0.)),
                Vector4::from((0., 0., 0., 1.)),
            ],
        }
    }

    pub fn from_rotation_origin(rotation: Quaternion, origin: Vector3) -> Self {
        Self::identity()
            .translate(origin)
            .rotate_quat(rotation)
            .translate(-origin)
    }

    pub fn from_rotation_translation_scale_origin(
        q: Quaternion,
        v: Vector3,
        s: Vector3,
        o: Vector3,
    ) -> Self {
        Self::identity()
            .translate(v)
            .translate(o)
            .rotate_quat(q)
            .scale(s)
            .translate(-o)
    }

    pub fn from_translation(v: Vector3) -> Self {
        Self::identity().translate(v)
    }

    pub fn translate(mut self, v: Vector3) -> Self {
        self[3] = self[0] * v.x + self[1] * v.y + self[2] * v.z + self[3];

        self
    }

    pub fn scale(mut self, v: Vector3) -> Self {
        self[0] *= v[0];
        self[1] *= v[1];
        self[2] *= v[2];

        self
    }

    pub fn transpose(mut self) -> Self {
        for column in 0..4 {
            for row in 0..column {
                let value = self[column][row];
                self[column][row] = self[row][column];
                self[row][column] = value;
            }
        }
        self
    }

    pub fn inverse(self) -> Self {
        let mut rows = [[0.; 8]; 4];
        for row in 0..4 {
            for column in 0..4 {
                rows[row][column] = self[column][row];
            }
            rows[row][row + 4] = 1.;
        }

        for column in 0..4 {
            let pivot = (column..4)
                .max_by(|&a, &b| rows[a][column].abs().total_cmp(&rows[b][column].abs()))
                .unwrap();
            assert!(rows[pivot][column] != 0., "matrix is singular");
            rows.swap(column, pivot);

            let divisor = rows[column][column];
            for value in &mut rows[column] {
                *value /= divisor;
            }
            let pivot_row = rows[column];
            (0..4).for_each(|row| {
                if row == column {
                    return;
                }
                let factor = rows[row][column];
                for i in 0..8 {
                    rows[row][i] -= factor * pivot_row[i];
                }
            });
        }

        let mut result = Self::identity();
        for column in 0..4 {
            for row in 0..4 {
                result[column][row] = rows[row][column + 4];
            }
        }
        result
    }

    pub fn rotate4(self, quat: Vector4) -> Self {
        let x2 = quat.x + quat.x;
        let y2 = quat.y + quat.y;
        let z2 = quat.z + quat.z;
        let xx = quat.x * x2;
        let xy = quat.x * y2;
        let xz = quat.x * z2;
        let yy = quat.y * y2;
        let yz = quat.y * z2;
        let zz = quat.z * z2;
        let wx = quat.w * x2;
        let wy = quat.w * y2;
        let wz = quat.w * z2;

        self * Self {
            columns: [
                Vector4::from((1. - yy - zz, xy + wz, xz - wy, 0.)),
                Vector4::from((xy - wz, 1. - xx - zz, yz + wx, 0.)),
                Vector4::from((xz + wy, yz - wx, 1. - xx - yy, 0.)),
                Vector4::from((0., 0., 0., 1.)),
            ],
        }
    }

    pub fn rotate(self, euler: Vector3, order: RotationOrder) -> Self {
        let (sx, cx) = (euler.x * 0.5).sin_cos();
        let (sy, cy) = (euler.y * 0.5).sin_cos();
        let (sz, cz) = (euler.z * 0.5).sin_cos();
        let x = Vector4::from((sx, 0., 0., cx));
        let y = Vector4::from((0., sy, 0., cy));
        let z = Vector4::from((0., 0., sz, cz));

        match order {
            RotationOrder::XYZ => self.rotate4(x).rotate4(y).rotate4(z),
            RotationOrder::XZY => self.rotate4(x).rotate4(z).rotate4(y),
            RotationOrder::YXZ => self.rotate4(y).rotate4(x).rotate4(z),
            RotationOrder::YZX => self.rotate4(y).rotate4(z).rotate4(x),
            RotationOrder::ZXY => self.rotate4(z).rotate4(x).rotate4(y),
            RotationOrder::ZYX => self.rotate4(z).rotate4(y).rotate4(x),
        }
    }

    pub fn rotate_quat(self, quat: Quaternion) -> Self {
        self.rotate4(Vector4::from((quat.x, quat.y, quat.z, quat.w)))
    }

    pub fn perspective(self, vertical_fov: f32, aspect: f32, znear: f32, zfar: f32) -> Self {
        let f = 1. / (vertical_fov * 0.5).tan();
        self * Self {
            columns: [
                Vector4::from((f / aspect, 0., 0., 0.)),
                Vector4::from((0., f, 0., 0.)),
                Vector4::from((0., 0., (zfar + znear) / (znear - zfar), -1.)),
                Vector4::from((0., 0., 2. * zfar * znear / (znear - zfar), 0.)),
            ],
        }
    }

    pub fn ortho(
        self,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        self * Self {
            columns: [
                Vector4::from((2. / (right - left), 0., 0., 0.)),
                Vector4::from((0., 2. / (top - bottom), 0., 0.)),
                Vector4::from((0., 0., -2. / (zfar - znear), 0.)),
                Vector4::from((
                    -(right + left) / (right - left),
                    -(top + bottom) / (top - bottom),
                    -(zfar + znear) / (zfar - znear),
                    1.,
                )),
            ],
        }
    }

    pub fn look_at(self, eye: Vector3, direction: Vector3, up: Vector3) -> Self {
        let forward = direction.normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward);

        self * Self {
            columns: [
                Vector4::from((right.x, up.x, -forward.x, 0.)),
                Vector4::from((right.y, up.y, -forward.y, 0.)),
                Vector4::from((right.z, up.z, -forward.z, 0.)),
                Vector4::from((-right.dot(eye), -up.dot(eye), forward.dot(eye), 1.)),
            ],
        }
    }
}
