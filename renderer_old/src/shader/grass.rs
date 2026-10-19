pub struct ModelGrass {
    pub camera: shared::math3::Vector2,
    pub fade: shared::math3::Vector2,
    pub height: shared::math3::Vector2,
    pub wind: shared::math3::Vector3,
    pub phase: shared::math3::Vector4,
    pub time: f32,
    pub enabled: bool,
}

impl ModelGrass {
    pub fn pack(&self) -> shared::math3::Matrix4 {
        let mut packed = shared::math3::Matrix4::zero();
        packed[0] = (self.camera[0], self.camera[1], self.fade[0], self.fade[1]).into();
        packed[1] = (self.time, self.wind[0], self.wind[1], self.wind[2]).into();
        packed[2] = self.phase;
        packed[3] = (
            self.height[0],
            self.height[1],
            if self.enabled { 1. } else { 0. },
            0.,
        )
            .into();
        packed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_grass_parameters_by_matrix_column() {
        let grass = ModelGrass {
            camera: (1., 2.).into(),
            fade: (3., 4.).into(),
            time: 5.,
            wind: (6., 7., 8.).into(),
            phase: (9., 10., 11., 12.).into(),
            height: (13., 14.).into(),
            enabled: true,
        };

        let packed = grass.pack();
        assert_eq!(packed[0], (1., 2., 3., 4.).into());
        assert_eq!(packed[1], (5., 6., 7., 8.).into());
        assert_eq!(packed[2], (9., 10., 11., 12.).into());
        assert_eq!(packed[3], (13., 14., 1., 0.).into());
    }
}
