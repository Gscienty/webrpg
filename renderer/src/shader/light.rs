#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightType {
    #[default]
    Omni,
    Direct,
    Ambient,
}

pub struct ModelLight {
    pub pos: shared::math3::Vector3,
    pub dir: shared::math3::Vector3,
    pub color: shared::math3::Vector3,
    pub ambient: shared::math3::Vector3,

    pub atten_start: f32,
    pub intensity: f32,
    pub ambient_intensity: f32,

    pub typ: LightType,
}

#[derive(Default, Clone, Copy)]
pub struct EnvironLight {
    pub dir: shared::math3::Vector3,
    pub color: shared::math3::Vector3,
    pub ambient: shared::math3::Vector3,
    pub intensity: f32,
    pub ambient_intensity: f32,
    pub typ: LightType,
    pub valid: bool,
}

pub struct ModelLighting {
    pub lights: Vec<ModelLight>,
    pub ambient: shared::math3::Vector3,
}

impl ModelLighting {
    pub fn pack(&self) -> Vec<shared::math3::Matrix4> {
        let mut packed: Vec<_> = self
            .lights
            .iter()
            .map(|light| {
                let mut matrix = shared::math3::Matrix4::zero();
                matrix[0] = shared::math3::Vector4::from((
                    light.pos[0],
                    light.pos[1],
                    light.pos[2],
                    match light.typ {
                        LightType::Omni => 0.,
                        LightType::Direct => 1.,
                        LightType::Ambient => 2.,
                    },
                ));
                matrix[1] = shared::math3::Vector4::from((
                    -light.dir[0],
                    -light.dir[1],
                    -light.dir[2],
                    light.atten_start,
                ));
                matrix[2] = shared::math3::Vector4::from((
                    light.color[0],
                    light.color[1],
                    light.color[2],
                    light.intensity,
                ));
                matrix[3] = shared::math3::Vector4::from((
                    light.ambient[0],
                    light.ambient[1],
                    light.ambient[2],
                    light.ambient_intensity,
                ));
                matrix
            })
            .collect();

        if let Some(first) = packed.first_mut() {
            for i in 0..3 {
                first[3][i] = first[3][i] * first[3][3] + self.ambient[i];
            }
            first[3][3] = 1.;
        }

        packed
    }
}

impl TryFrom<&EnvironLight> for ModelLighting {
    type Error = ();

    fn try_from(light: &EnvironLight) -> Result<Self, Self::Error> {
        if !light.valid {
            return Err(());
        }

        Ok(Self {
            lights: vec![ModelLight {
                pos: shared::math3::Vector3::default(),
                dir: light.dir,
                color: light.color,
                ambient: light.ambient,
                atten_start: 0.,
                intensity: light.intensity,
                ambient_intensity: light.ambient_intensity,
                typ: light.typ,
            }],
            ambient: shared::math3::Vector3::default(),
        })
    }
}

impl From<&ModelLighting> for EnvironLight {
    fn from(lighting: &ModelLighting) -> Self {
        let Some(light) = lighting.lights.first() else {
            return Self {
                dir: shared::math3::Vector3::default(),
                color: shared::math3::Vector3::default(),
                ambient: shared::math3::Vector3::default(),
                intensity: 0.,
                ambient_intensity: 0.,
                typ: LightType::Omni,
                valid: false,
            };
        };

        Self {
            dir: light.dir,
            color: light.color,
            ambient: light.ambient,
            intensity: light.intensity,
            ambient_intensity: light.ambient_intensity,
            typ: light.typ,
            valid: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_lights_and_merges_global_ambient_into_the_first() {
        let lighting = ModelLighting {
            lights: vec![ModelLight {
                pos: (1., 2., 3.).into(),
                dir: (4., 5., 6.).into(),
                color: (7., 8., 9.).into(),
                ambient: (10., 11., 12.).into(),
                atten_start: 13.,
                intensity: 14.,
                ambient_intensity: 2.,
                typ: LightType::Direct,
            }],
            ambient: (1., 2., 3.).into(),
        };

        let packed = lighting.pack();
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0][0], (1., 2., 3., 1.).into());
        assert_eq!(packed[0][1], (-4., -5., -6., 13.).into());
        assert_eq!(packed[0][2], (7., 8., 9., 14.).into());
        assert_eq!(packed[0][3], (21., 24., 27., 1.).into());
    }

    #[test]
    fn packs_empty_lighting_as_empty() {
        let lighting = ModelLighting {
            lights: Vec::new(),
            ambient: (1., 2., 3.).into(),
        };

        assert!(lighting.pack().is_empty());
    }

    #[test]
    fn converts_between_environment_and_model_lighting() {
        let environment = EnvironLight {
            dir: (1., 2., 3.).into(),
            color: (4., 5., 6.).into(),
            ambient: (7., 8., 9.).into(),
            intensity: 10.,
            ambient_intensity: 11.,
            typ: LightType::Direct,
            valid: true,
        };

        let lighting = ModelLighting::try_from(&environment).unwrap();
        assert_eq!(lighting.lights.len(), 1);
        assert_eq!(lighting.lights[0].dir, environment.dir);
        assert_eq!(lighting.lights[0].color, environment.color);
        assert_eq!(lighting.lights[0].ambient, environment.ambient);
        assert_eq!(lighting.lights[0].intensity, environment.intensity);
        assert_eq!(
            lighting.lights[0].ambient_intensity,
            environment.ambient_intensity
        );
        assert_eq!(lighting.lights[0].typ, environment.typ);

        let converted = EnvironLight::from(&lighting);
        assert!(converted.valid);
        assert_eq!(converted.dir, environment.dir);
        assert_eq!(converted.typ, environment.typ);

        let invalid = EnvironLight {
            valid: false,
            ..converted
        };
        assert!(ModelLighting::try_from(&invalid).is_err());

        let empty = ModelLighting {
            lights: Vec::new(),
            ambient: (1., 2., 3.).into(),
        };
        assert!(!EnvironLight::from(&empty).valid);
    }
}
