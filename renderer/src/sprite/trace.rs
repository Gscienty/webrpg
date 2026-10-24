use shared::math3::{self, Line3Intersect};

use crate::sprite::{Sprite, utils};

impl<'s> Sprite<'s> {
    pub fn trace(&self, param: &TraceParam) -> Option<math3::Vector3> {
        let model: &'s fileformat::mdx::Model<'s> =
            unsafe { &*(self.model.as_ref().get_ref() as *const _) };
        if model.collision_shapes.is_empty() {
            return self.trace_mesh(param);
        }

        let transform = utils::evaluate_sprite_transform(param.angle, param.origin, param.scale);
        let inverse = transform.inverse();
        let local_line =
            math3::Line3::from((inverse * param.line.start(), inverse * param.line.end()));

        model
            .collision_shapes
            .iter()
            .filter_map(|shape| {
                let point = match shape.type_ {
                    fileformat::mdx::CollisionShapeType::Box_ => {
                        let (&min, &max) = (shape.vertices.first()?, shape.vertices.get(1)?);
                        let local_point = local_line.intersect(math3::Box3::from((min, max)))?;
                        transform * local_point
                    }
                    fileformat::mdx::CollisionShapeType::Sphere => {
                        let center = transform * *shape.vertices.first()?;
                        let radius = shape.radius? * param.scale.abs();
                        param.line.intersect(math3::Sphere3 { center, radius })?
                    }
                    _ => return None,
                };
                Some((param.line.start().distance(point), point))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, point)| point)
    }

    pub fn trace_mesh(&self, param: &TraceParam) -> Option<math3::Vector3> {
        let model: &'s fileformat::mdx::Model<'s> =
            unsafe { &*(self.model.as_ref().get_ref() as *const _) };

        let transform = utils::evaluate_sprite_transform(param.angle, param.origin, param.scale);
        let inverse = transform.inverse();
        let local_line =
            math3::Line3::from((inverse * param.line.start(), inverse * param.line.end()));

        let mut best = None;
        for geoset in &model.geosets {
            if geoset.selectable & 4 != 0
                || !geoset.visible(param.global_time, param.frame)
                || local_line.intersect(geoset.default_bounds.cube).is_none()
            {
                continue;
            }

            if let Some(local_point) = geoset.trace(&local_line) {
                let point = transform * local_point;
                let distance = param.line.start().distance(point);
                if best.is_none_or(|(best_distance, _)| distance < best_distance) {
                    best = Some((distance, point));
                }
            }
        }

        best.map(|(_, point)| point)
    }
}

pub struct TraceParam {
    pub origin: math3::Vector3,
    pub angle: f32,
    pub scale: f32,

    pub global_time: u32,
    pub frame: u32,

    pub line: math3::Line3,
}
