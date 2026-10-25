use std::{
    io::{self, Read},
    mem::size_of,
};

use shared::math3;

use crate::mdx::{model::Model, track::Tracks, utils::read_value};

const KMTA: u32 = u32::from_le_bytes(*b"KMTA");
const KMTE: u32 = u32::from_le_bytes(*b"KMTE");
const KMTF: u32 = u32::from_le_bytes(*b"KMTF");
const LAYS: u32 = u32::from_le_bytes(*b"LAYS");

#[derive(Debug)]
pub enum GeoFlag {
    Unshared = 0x001,
    SphereEnvMap = 0x002,
    WrapWidth = 0x004,
    WrapHeight = 0x008,
    TwoSided = 0x010,
    Unfogged = 0x020,
    NoDepthTest = 0x040,
    NoDepthSet = 0x080,
    NoFallback = 0x100,
}

#[derive(Debug)]
pub struct Layer<'s> {
    pub model: &'s Model<'s>,
    pub blend_mode: shared::BlendMode,
    pub flags: u32,
    pub texture_id: u32,
    pub transform_id: u32,
    pub coord_id: u32,
    pub alpha: f32,

    pub alpha_tracks: Option<Tracks<'s, f32>>,
    pub flipbook_tracks: Option<Tracks<'s, u32>>,
}

#[repr(C)]
struct LayerData {
    blend_mode: u32,
    flags: u32,
    texture_id: u32,
    transform_id: u32,
    coord_id: u32,
    alpha: f32,
}

impl<'s> Layer<'s> {
    pub fn bind_texture_animation(&self, global_time: u32, frame: u32) -> math3::Matrix3 {
        let mut translation = math3::Vector3::default();
        let mut rotation = math3::Quaternion::default();
        let mut scale = math3::Vector3::from((1., 1., 1.));

        if let Some(animation) = self
            .model
            .texture_animations
            .get(self.transform_id as usize)
        {
            if let Some(value) = animation
                .translation
                .as_ref()
                .and_then(|track| track.get_value(global_time, frame))
            {
                translation = value;
            }
            if let Some(value) = animation
                .rotation
                .as_ref()
                .and_then(|track| track.get_value(global_time, frame))
            {
                rotation = value;
            }
            if let Some(value) = animation
                .scale
                .as_ref()
                .and_then(|track| track.get_value(global_time, frame))
            {
                scale = value;
            }
        }

        if !translation[0].is_finite() || !translation[1].is_finite() {
            translation = math3::Vector3::default();
        }
        if !rotation.z.is_finite() || !rotation.w.is_finite() {
            rotation = math3::Quaternion::default();
        }
        if !scale[0].is_finite() || !scale[1].is_finite() {
            scale = math3::Vector3::from((1., 1., 1.));
        }

        let c = rotation.w * rotation.w - rotation.z * rotation.z;
        let s = 2. * rotation.z * rotation.w;
        let tx = scale[0] * (c * (translation[0] - 0.5) - s * (translation[1] - 0.5)) + 0.5;
        let ty = scale[1] * (s * (translation[0] - 0.5) + c * (translation[1] - 0.5)) + 0.5;
        let mut matrix = math3::Matrix3::zero();
        matrix[0] = (scale[0] * c, scale[1] * s, 0.).into();
        matrix[1] = (-scale[0] * s, scale[1] * c, 0.).into();
        matrix[2] = (tx, ty, 1.).into();
        matrix
    }

    pub fn evaluate_texture_id(&self, global_time: u32, frame: u32) -> usize {
        self.flipbook_tracks
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
            .filter(|&texture_id| (texture_id as usize) < self.model.textures.len())
            .unwrap_or(self.texture_id) as usize
    }

    pub fn is_blended(&self) -> bool {
        matches!(
            self.blend_mode,
            shared::BlendMode::Blend
                | shared::BlendMode::Add
                | shared::BlendMode::AddAlpha
                | shared::BlendMode::Modulate
                | shared::BlendMode::Modulate2X
        )
    }

    pub fn evaluate_alpha(&self, material: &Material, global_time: u32, frame: u32) -> f32 {
        let mut alpha = if self.alpha.is_finite() {
            self.alpha
        } else {
            1.
        };

        if let Some(value) = self
            .alpha_tracks
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
        {
            alpha = value;
        }
        if let Some(material_alpha) = material
            .alpha_tracks
            .as_ref()
            .and_then(|track| track.get_value(global_time, frame))
        {
            alpha *= material_alpha;
        }

        if !alpha.is_finite() {
            alpha = 1.;
        }
        alpha.clamp(0., 1.)
    }

    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid layer size"))?;
        if payload_size < size_of::<LayerData>() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid layer size",
            ));
        }

        let mut payload = reader.take(payload_size as u64);
        let data: LayerData = read_value(&mut payload)?;
        let blend_mode = match data.blend_mode {
            0 => shared::BlendMode::None,
            1 => shared::BlendMode::AlphaKey,
            2 => shared::BlendMode::Blend,
            3 => shared::BlendMode::Add,
            4 => shared::BlendMode::AddAlpha,
            5 => shared::BlendMode::Modulate,
            6 => shared::BlendMode::Modulate2X,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid layer blend mode",
                ));
            }
        };
        let mut result = Self {
            model,
            blend_mode,
            flags: data.flags,
            texture_id: data.texture_id,
            transform_id: data.transform_id,
            coord_id: data.coord_id,
            alpha: data.alpha,
            alpha_tracks: None,
            flipbook_tracks: None,
        };

        while payload.limit() > 0 {
            let tag: u32 = read_value(&mut payload)?;
            match tag {
                KMTA => result.alpha_tracks = Some(Tracks::read(model, &mut payload)?),
                KMTF => result.flipbook_tracks = Some(Tracks::read(model, &mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported layer track",
                    ));
                }
            }
        }

        Ok(result)
    }

    pub fn read_list(
        model: &'s Model,
        reader: &mut impl Read,
        size: usize,
    ) -> io::Result<Vec<Self>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(model, &mut payload)?);
        }
        Ok(values)
    }
}

#[derive(Debug)]
pub struct Material<'s> {
    pub priority: u32,
    pub flags: u32,
    pub layers: Vec<Layer<'s>>,

    pub emission_tracks: Option<Tracks<'s, f32>>,
    pub alpha_tracks: Option<Tracks<'s, f32>>,
    pub flipbook_tracks: Option<Tracks<'s, f32>>,
}

impl<'s> Material<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid material size"))?;
        let mut payload = reader.take(payload_size as u64);
        let [priority, flags] = read_value(&mut payload)?;
        let mut layers = None;
        let mut emission_tracks = None;
        let mut alpha_tracks = None;
        let mut flipbook_tracks = None;

        while payload.limit() > 0 {
            let tag: u32 = read_value(&mut payload)?;
            match tag {
                LAYS => {
                    let layer_count: u32 = read_value(&mut payload)?;
                    let mut values = Vec::with_capacity(layer_count as usize);
                    for _ in 0..layer_count {
                        values.push(Layer::read(model, &mut payload)?);
                    }
                    layers = Some(values);
                }
                KMTE => emission_tracks = Some(Tracks::read(model, &mut payload)?),
                KMTA => alpha_tracks = Some(Tracks::read(model, &mut payload)?),
                KMTF => flipbook_tracks = Some(Tracks::read(model, &mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported material data",
                    ));
                }
            }
        }

        let Some(layers) = layers else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "missing material layers",
            ));
        };

        Ok(Self {
            priority,
            flags,
            layers,
            emission_tracks,
            alpha_tracks,
            flipbook_tracks,
        })
    }

    pub fn read_list(
        model: &'s Model,
        reader: &mut impl Read,
        size: usize,
    ) -> io::Result<Vec<Self>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(model, &mut payload)?);
        }
        Ok(values)
    }
}
