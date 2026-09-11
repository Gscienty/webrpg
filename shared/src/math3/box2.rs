use super::Vector2;

#[derive(PartialEq, Clone, Copy, Default, Debug)]
pub struct Box2 {
    min: Vector2,
    max: Vector2,
}

impl From<(Vector2, Vector2)> for Box2 {
    fn from((min, max): (Vector2, Vector2)) -> Self {
        Self { min, max }
    }
}

impl Box2 {
    pub fn center(&self) -> Vector2 {
        (self.min + self.max) * 0.5
    }

    pub fn move_to(&self, next_center: Vector2) -> Self {
        let center = self.center();

        Self {
            min: self.min + next_center - center,
            max: self.max + next_center - center,
        }
    }

    pub fn contains(&self, point: Vector2) -> bool {
        self.min.x <= point.x
            && point.x < self.max.x
            && self.min.y <= point.y
            && point.y < self.max.y
    }
}
