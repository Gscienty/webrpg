use super::{Box3, Matrix4, Plane3, Sphere3, Vector3};

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Frustum3 {
    pub left: Plane3,
    pub right: Plane3,
    pub bottom: Plane3,
    pub top: Plane3,
    pub front: Plane3,
    pub back: Plane3,
}

pub trait FrustumContains<T> {
    fn contains(&self, obj: T) -> bool;
}

impl Frustum3 {
    pub fn calculate(mat: Matrix4) -> Self {
        let plane = |axis: usize, sign: f32| {
            Plane3 {
                normal: Vector3::from((
                    mat[0][3] + sign * mat[0][axis],
                    mat[1][3] + sign * mat[1][axis],
                    mat[2][3] + sign * mat[2][axis],
                )),
                distance: mat[3][3] + sign * mat[3][axis],
            }
            .normalize()
        };

        Self {
            left: plane(0, 1.),
            right: plane(0, -1.),
            bottom: plane(1, 1.),
            top: plane(1, -1.),
            front: plane(2, 1.),
            back: plane(2, -1.),
        }
    }

    fn planes(&self) -> [Plane3; 6] {
        [
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.front,
            self.back,
        ]
    }
}

impl FrustumContains<Vector3> for Frustum3 {
    fn contains(&self, obj: Vector3) -> bool {
        self.planes().into_iter().all(|plane| plane * obj >= 0.)
    }
}

impl FrustumContains<Sphere3> for Frustum3 {
    fn contains(&self, obj: Sphere3) -> bool {
        self.planes()
            .into_iter()
            .all(|plane| plane * obj.center >= -obj.radius)
    }
}

impl FrustumContains<Box3> for Frustum3 {
    fn contains(&self, obj: Box3) -> bool {
        self.planes().into_iter().all(|plane| {
            let point = Vector3::from((
                if plane.normal.x >= 0. {
                    obj.max.x
                } else {
                    obj.min.x
                },
                if plane.normal.y >= 0. {
                    obj.max.y
                } else {
                    obj.min.y
                },
                if plane.normal.z >= 0. {
                    obj.max.z
                } else {
                    obj.min.z
                },
            ));
            plane * point >= 0.
        })
    }
}

impl FrustumContains<(Box3, Matrix4)> for Frustum3 {
    fn contains(&self, (box_, mat): (Box3, Matrix4)) -> bool {
        let corners = [
            Vector3::from((box_.min.x, box_.min.y, box_.min.z)),
            Vector3::from((box_.min.x, box_.min.y, box_.max.z)),
            Vector3::from((box_.min.x, box_.max.y, box_.min.z)),
            Vector3::from((box_.min.x, box_.max.y, box_.max.z)),
            Vector3::from((box_.max.x, box_.min.y, box_.min.z)),
            Vector3::from((box_.max.x, box_.min.y, box_.max.z)),
            Vector3::from((box_.max.x, box_.max.y, box_.min.z)),
            Vector3::from((box_.max.x, box_.max.y, box_.max.z)),
        ]
        .map(|point| mat * point);

        self.planes()
            .into_iter()
            .all(|plane| corners.into_iter().any(|point| plane * point >= 0.))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3::from((x, y, z))
    }

    #[test]
    fn calculates_and_culls_points_spheres_and_boxes() {
        let frustum = Frustum3::calculate(Matrix4::identity());

        assert!(frustum.contains(v(0., 0., 0.)));
        assert!(frustum.contains(v(1., 1., 1.)));
        assert!(!frustum.contains(v(1.1, 0., 0.)));
        assert!(frustum.contains(Sphere3 {
            center: v(1.5, 0., 0.),
            radius: 0.5,
        }));
        assert!(!frustum.contains(Sphere3 {
            center: v(2., 0., 0.),
            radius: 0.5,
        }));

        let box_ = Box3::from((v(-0.5, -0.5, -0.5), v(0.5, 0.5, 0.5)));
        assert!(frustum.contains(box_));
        assert!(frustum.contains((box_, Matrix4::from_translation(v(1., 0., 0.)))));
        assert!(!frustum.contains((box_, Matrix4::from_translation(v(2., 0., 0.)))));
    }
}
