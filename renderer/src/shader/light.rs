#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightType {
    #[default]
    Omni,
    Direct,
    Ambient,
}

#[derive(Default)]
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

#[derive(Default)]
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
