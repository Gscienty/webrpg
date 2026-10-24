use super::{Box3, Plane3, Sphere3, Triangle3, Vector3};

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Line3 {
    a: Vector3,
    b: Vector3,
}

impl From<(Vector3, Vector3)> for Line3 {
    fn from((a, b): (Vector3, Vector3)) -> Self {
        Self { a, b }
    }
}

impl Line3 {
    pub const fn start(&self) -> Vector3 {
        self.a
    }

    pub const fn end(&self) -> Vector3 {
        self.b
    }
}

pub trait Line3Intersect<T> {
    fn intersect(&self, obj: T) -> Option<Vector3>;
}

impl Line3Intersect<Sphere3> for Line3 {
    fn intersect(&self, obj: Sphere3) -> Option<Vector3> {
        let direction = self.b - self.a;
        let offset = self.a - obj.center;
        let a = direction.dot(direction);
        if a == 0. {
            return ((offset.dot(offset) - obj.radius * obj.radius).abs() <= f32::EPSILON)
                .then_some(self.a);
        }

        let b = 2. * offset.dot(direction);
        let discriminant = b * b - 4. * a * (offset.dot(offset) - obj.radius * obj.radius);
        if discriminant < 0. {
            return None;
        }

        let root = discriminant.sqrt();
        [(-b - root) / (2. * a), (-b + root) / (2. * a)]
            .into_iter()
            .find(|t| (0. ..=1.).contains(t))
            .map(|t| self.a + direction * t)
    }
}

impl Line3Intersect<Plane3> for Line3 {
    fn intersect(&self, obj: Plane3) -> Option<Vector3> {
        let direction = self.b - self.a;
        let denominator = obj.normal.dot(direction);
        if denominator.abs() <= f32::EPSILON {
            return None;
        }

        let t = -(obj * self.a) / denominator;
        (0. ..=1.).contains(&t).then_some(self.a + direction * t)
    }
}

impl Line3Intersect<Triangle3> for Line3 {
    fn intersect(&self, obj: Triangle3) -> Option<Vector3> {
        let direction = self.b - self.a;
        let edge_1 = obj.b - obj.a;
        let edge_2 = obj.c - obj.a;
        let p = direction.cross(edge_2);
        let determinant = edge_1.dot(p);
        if determinant.abs() <= f32::EPSILON {
            return None;
        }

        let inverse = 1. / determinant;
        let offset = self.a - obj.a;
        let u = offset.dot(p) * inverse;
        if !(0. ..=1.).contains(&u) {
            return None;
        }
        let q = offset.cross(edge_1);
        let v = direction.dot(q) * inverse;
        if v < 0. || u + v > 1. {
            return None;
        }
        let t = edge_2.dot(q) * inverse;
        (0. ..=1.).contains(&t).then_some(self.a + direction * t)
    }
}

impl Line3Intersect<Box3> for Line3 {
    fn intersect(&self, obj: Box3) -> Option<Vector3> {
        let direction = self.b - self.a;
        let mut enter = f32::NEG_INFINITY;
        let mut exit = f32::INFINITY;

        for axis in 0..3 {
            if direction[axis].abs() <= f32::EPSILON {
                if self.a[axis] < obj.min[axis] || self.a[axis] > obj.max[axis] {
                    return None;
                }
                continue;
            }

            let mut near = (obj.min[axis] - self.a[axis]) / direction[axis];
            let mut far = (obj.max[axis] - self.a[axis]) / direction[axis];
            if near > far {
                std::mem::swap(&mut near, &mut far);
            }
            enter = enter.max(near);
            exit = exit.min(far);
            if enter > exit {
                return None;
            }
        }

        let t = if enter >= 0. { enter } else { exit };
        (0. ..=1.).contains(&t).then_some(self.a + direction * t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3::from((x, y, z))
    }

    #[test]
    fn intersects_supported_shapes_on_the_segment() {
        let line = Line3::from((v(-2., 0., 0.), v(2., 0., 0.)));

        assert_eq!(
            line.intersect(Sphere3 {
                center: v(0., 0., 0.),
                radius: 1.,
            }),
            Some(v(-1., 0., 0.))
        );
        assert_eq!(
            line.intersect(Plane3 {
                normal: v(1., 0., 0.),
                distance: 0.,
            }),
            Some(v(0., 0., 0.))
        );
        assert_eq!(
            Line3::from((v(0., 0., 1.), v(0., 0., -1.))).intersect(Triangle3::from((
                v(-1., -1., 0.),
                v(1., -1., 0.),
                v(0., 1., 0.),
            ))),
            Some(v(0., 0., 0.))
        );
        assert_eq!(
            line.intersect(Box3::from((v(-1., -1., -1.), v(1., 1., 1.)))),
            Some(v(-1., 0., 0.))
        );
    }

    #[test]
    fn ignores_intersections_outside_the_segment() {
        let line = Line3::from((v(2., 0., 0.), v(3., 0., 0.)));

        assert_eq!(
            line.intersect(Sphere3 {
                center: v(0., 0., 0.),
                radius: 1.,
            }),
            None
        );
        assert_eq!(
            line.intersect(Plane3 {
                normal: v(1., 0., 0.),
                distance: 0.,
            }),
            None
        );
        assert_eq!(
            line.intersect(Box3::from((v(-1., -1., -1.), v(1., 1., 1.)))),
            None
        );
    }
}
