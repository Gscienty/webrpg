use std::{
    io::{self, Read},
    mem::size_of,
};

use crate::{
    context::{Context, RenderPhase},
    model::mdx::{Model, Tracks, utils::read_value},
    shader::ModelState,
};
use web_sys::WebGl2RenderingContext;

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
pub struct Layer {
    pub blend_mode: shared::BlendMode,
    pub flags: u32,
    pub texture_id: u32,
    pub transform_id: u32,
    pub coord_id: u32,
    pub alpha: f32,

    pub alpha_tracks: Option<Tracks<f32>>,
    pub flipbook_tracks: Option<Tracks<u32>>,
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

impl Layer {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
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
                KMTA => result.alpha_tracks = Some(Tracks::read(&mut payload)?),
                KMTF => result.flipbook_tracks = Some(Tracks::read(&mut payload)?),
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

    pub const fn is_blended(&self) -> bool {
        matches!(
            self.blend_mode,
            shared::BlendMode::Blend
                | shared::BlendMode::Add
                | shared::BlendMode::AddAlpha
                | shared::BlendMode::Modulate
                | shared::BlendMode::Modulate2X
        )
    }

    pub fn set_layer_blend(
        &self,
        context: &Context,
        state: &mut ModelState,
        layer_id: usize,
    ) -> bool {
        context.set_alpha_key_state(false);

        if context.render_phase == RenderPhase::Lights && self.is_blended() {
            return false;
        }

        let gl = &context.gl;
        match self.blend_mode {
            shared::BlendMode::None => {
                gl.disable(WebGl2RenderingContext::BLEND);
                if layer_id == 0 {
                    gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);
                } else {
                    gl.blend_func(
                        WebGl2RenderingContext::SRC_ALPHA,
                        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
                    );
                }
                gl.depth_mask(true);
            }
            shared::BlendMode::AlphaKey => {
                state.alpha_key = true;
                context.set_alpha_key_state(true);
            }
            shared::BlendMode::Blend => {
                gl.enable(WebGl2RenderingContext::BLEND);
                gl.blend_func(
                    WebGl2RenderingContext::SRC_ALPHA,
                    WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
                );
                gl.depth_mask(false);
            }
            shared::BlendMode::Add => {
                gl.enable(WebGl2RenderingContext::BLEND);
                gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);
                gl.depth_mask(false);
            }
            shared::BlendMode::AddAlpha => {
                gl.enable(WebGl2RenderingContext::BLEND);
                gl.blend_func(
                    WebGl2RenderingContext::SRC_ALPHA,
                    WebGl2RenderingContext::ONE,
                );
                gl.depth_mask(false);
            }
            shared::BlendMode::Modulate => {
                gl.enable(WebGl2RenderingContext::BLEND);
                gl.blend_func(
                    WebGl2RenderingContext::DST_COLOR,
                    WebGl2RenderingContext::ZERO,
                );
                gl.depth_mask(false);
            }
            shared::BlendMode::Modulate2X => {
                gl.enable(WebGl2RenderingContext::BLEND);
                gl.blend_func(
                    WebGl2RenderingContext::DST_COLOR,
                    WebGl2RenderingContext::SRC_COLOR,
                );
                gl.depth_mask(false);
            }
        }

        true
    }

    pub fn apply_flags(&self, context: &Context) {
        let gl = &context.gl;
        if self.flags & GeoFlag::TwoSided as u32 != 0 {
            gl.disable(WebGl2RenderingContext::CULL_FACE);
        }
        if self.flags & GeoFlag::NoDepthTest as u32 != 0 {
            gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        }
        if self.flags & GeoFlag::NoDepthSet as u32 != 0 {
            gl.depth_mask(false);
        }
    }

    pub fn evaluate_layer_alpha(
        &self,
        material: &Material,
        model: &Model,
        global_time: u32,
        frame: u32,
    ) -> f32 {
        let mut alpha = if self.alpha.is_finite() {
            self.alpha
        } else {
            1.
        };

        if let Some(value) = self
            .alpha_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
        {
            alpha = value;
        }
        if let Some(material_alpha) = material
            .alpha_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
        {
            alpha *= material_alpha;
        }

        if !alpha.is_finite() {
            alpha = 1.;
        }
        alpha.clamp(0., 1.)
    }

    pub fn evaluate_layer_texture_id(&self, model: &Model, global_time: u32, frame: u32) -> usize {
        self.flipbook_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
            .filter(|&texture_id| (texture_id as usize) < model.textures.len())
            .unwrap_or(self.texture_id) as usize
    }

    pub fn bind_texture_animation(
        &self,
        model: &Model,
        global_time: u32,
        frame: u32,
    ) -> shared::math3::Matrix3 {
        let mut translation = shared::math3::Vector3::default();
        let mut rotation = shared::math3::Quaternion::default();
        let mut scale = shared::math3::Vector3::from((1., 1., 1.));

        if let Some(animation) = model.texture_animations.get(self.transform_id as usize) {
            if let Some(value) = animation
                .translation
                .as_ref()
                .and_then(|track| track.get_value(model, global_time, frame))
            {
                translation = value;
            }
            if let Some(value) = animation
                .rotation
                .as_ref()
                .and_then(|track| track.get_value(model, global_time, frame))
            {
                rotation = value;
            }
            if let Some(value) = animation
                .scale
                .as_ref()
                .and_then(|track| track.get_value(model, global_time, frame))
            {
                scale = value;
            }
        }

        if !translation[0].is_finite() || !translation[1].is_finite() {
            translation = shared::math3::Vector3::default();
        }
        if !rotation.z.is_finite() || !rotation.w.is_finite() {
            rotation = shared::math3::Quaternion::default();
        }
        if !scale[0].is_finite() || !scale[1].is_finite() {
            scale = shared::math3::Vector3::from((1., 1., 1.));
        }

        let c = rotation.w * rotation.w - rotation.z * rotation.z;
        let s = 2. * rotation.z * rotation.w;
        let tx = scale[0] * (c * (translation[0] - 0.5) - s * (translation[1] - 0.5)) + 0.5;
        let ty = scale[1] * (s * (translation[0] - 0.5) + c * (translation[1] - 0.5)) + 0.5;
        let mut matrix = shared::math3::Matrix3::zero();
        matrix[0] = (scale[0] * c, scale[1] * s, 0.).into();
        matrix[1] = (-scale[0] * s, scale[1] * c, 0.).into();
        matrix[2] = (tx, ty, 1.).into();
        matrix
    }
}

#[derive(Debug)]
pub struct Material {
    pub priority: u32,
    pub flags: u32,
    pub layers: Vec<Layer>,

    pub emission_tracks: Option<Tracks<f32>>,
    pub alpha_tracks: Option<Tracks<f32>>,
    pub flipbook_tracks: Option<Tracks<f32>>,
}

impl Material {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
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
                        values.push(Layer::read(&mut payload)?);
                    }
                    layers = Some(values);
                }
                KMTE => emission_tracks = Some(Tracks::read(&mut payload)?),
                KMTA => alpha_tracks = Some(Tracks::read(&mut payload)?),
                KMTF => flipbook_tracks = Some(Tracks::read(&mut payload)?),
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

    pub fn read_list(reader: &mut impl Read, size: usize) -> io::Result<Vec<Self>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(&mut payload)?);
        }
        Ok(values)
    }

    pub fn has_pass(&self, blended_pass: bool) -> bool {
        self.layers
            .iter()
            .any(|layer| layer.is_blended() == blended_pass)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::model::mdx::TrackType;

    #[test]
    fn reads_layer_tracks() {
        let mut bytes = 76_u32.to_le_bytes().to_vec();
        for value in [2_u32, 3, 4, 5, 6] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(0.5_f32.to_le_bytes());

        bytes.extend(b"KMTA");
        for value in [1_u32, TrackType::NoInterp as u32, 7, 8] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(0.75_f32.to_le_bytes());

        bytes.extend(b"KMTF");
        for value in [1_u32, TrackType::NoInterp as u32, 9, 10, 11] {
            bytes.extend(value.to_le_bytes());
        }

        let layer = Layer::read(&mut Cursor::new(bytes)).unwrap();

        assert!(matches!(layer.blend_mode, shared::BlendMode::Blend));
        assert_eq!(layer.alpha_tracks.unwrap().values[0].data, 0.75);
        assert_eq!(layer.flipbook_tracks.unwrap().values[0].data, 11);
    }

    #[test]
    fn reads_material_list() {
        let mut bytes = 120_u32.to_le_bytes().to_vec();
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(2_u32.to_le_bytes());
        bytes.extend(b"LAYS");
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(28_u32.to_le_bytes());
        for value in [3_u32, 4, 5, 6, 7] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(0.5_f32.to_le_bytes());

        for (tag, value) in [(b"KMTE", 0.25_f32), (b"KMTA", 0.75), (b"KMTF", 1.0)] {
            bytes.extend(tag);
            for data in [1_u32, TrackType::NoInterp as u32, 0, 10] {
                bytes.extend(data.to_le_bytes());
            }
            bytes.extend(value.to_le_bytes());
        }

        let materials = Material::read_list(&mut Cursor::new(bytes), 120).unwrap();

        assert_eq!(materials.len(), 1);
        assert_eq!(materials[0].priority, 1);
        assert_eq!(materials[0].flags, 2);
        assert_eq!(materials[0].layers.len(), 1);
        assert!(matches!(
            materials[0].layers[0].blend_mode,
            shared::BlendMode::Add
        ));
        assert_eq!(
            materials[0].emission_tracks.as_ref().unwrap().values[0].data,
            0.25
        );
        assert_eq!(
            materials[0].alpha_tracks.as_ref().unwrap().values[0].data,
            0.75
        );
        assert_eq!(
            materials[0].flipbook_tracks.as_ref().unwrap().values[0].data,
            1.0
        );
    }
}
