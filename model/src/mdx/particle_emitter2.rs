use std::{
    io::{self, Read},
    mem::size_of,
    sync,
};

use crate::mdx::{Model, Node, Tracks, utils::read_value};

const KP2E: u32 = u32::from_le_bytes(*b"KP2E");
const KP2G: u32 = u32::from_le_bytes(*b"KP2G");
const KP2L: u32 = u32::from_le_bytes(*b"KP2L");
const KP2N: u32 = u32::from_le_bytes(*b"KP2N");
const KP2R: u32 = u32::from_le_bytes(*b"KP2R");
const KP2S: u32 = u32::from_le_bytes(*b"KP2S");
const KP2V: u32 = u32::from_le_bytes(*b"KP2V");
const KP2W: u32 = u32::from_le_bytes(*b"KP2W");

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ParticleAnimation {
    pub start: u32,
    pub end: u32,
    pub repeat: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct ParticleEmitter2Data {
    speed: f32,
    variation: f32,
    latitude: f32,
    gravity: f32,
    lifespan: f32,
    emission_rate: f32,
    length: f32,
    width: f32,
    filter_mode: u32,
    rows: u32,
    columns: u32,
    frame_flags: u32,
    tail_length: f32,
    time: f32,
    segment_color: [shared::math3::Vector3; 3],
    alpha: [u8; 3],
    particle_scaling: [f32; 3],
    lifespan_uv_anim: ParticleAnimation,
    decay_uv_anim: ParticleAnimation,
    tail_uv_anim: ParticleAnimation,
    tail_decay_uv_anim: ParticleAnimation,
    texture_id: u32,
    squirt: u32,
    priority_plane: u32,
    replaceable_id: u32,
}

#[derive(Debug)]
pub struct ParticleEmitter2 {
    pub node: sync::Arc<Node>,
    pub speed: f32,
    pub variation: f32,
    pub latitude: f32,
    pub gravity: f32,
    pub lifespan: f32,
    pub emission_rate: f32,
    pub length: f32,
    pub width: f32,
    pub filter_mode: u32,
    pub rows: u32,
    pub columns: u32,
    pub frame_flags: u32,
    pub tail_length: f32,
    pub time: f32,
    pub segment_color: [shared::math3::Vector3; 3],
    pub alpha: [u8; 3],
    pub particle_scaling: [f32; 3],
    pub lifespan_uv_anim: ParticleAnimation,
    pub decay_uv_anim: ParticleAnimation,
    pub tail_uv_anim: ParticleAnimation,
    pub tail_decay_uv_anim: ParticleAnimation,
    pub texture_id: u32,
    pub squirt: u32,
    pub priority_plane: u32,
    pub replaceable_id: u32,

    pub emission_rate_tracks: Option<Tracks<f32>>,
    pub width_tracks: Option<Tracks<f32>>,
    pub length_tracks: Option<Tracks<f32>>,
    pub speed_tracks: Option<Tracks<f32>>,
    pub latitude_tracks: Option<Tracks<f32>>,
    pub gravity_tracks: Option<Tracks<f32>>,
    pub variation_tracks: Option<Tracks<f32>>,
    pub visibility: Option<Tracks<f32>>,
}

impl ParticleEmitter2 {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid particle emitter 2 size",
                )
            })?;
        let mut payload = reader.take(payload_size as u64);
        let node = Node::read(&mut payload)?;
        let data: ParticleEmitter2Data = read_value(&mut payload)?;
        let mut result = Self {
            node,
            speed: data.speed,
            variation: data.variation,
            latitude: data.latitude,
            gravity: data.gravity,
            lifespan: data.lifespan,
            emission_rate: data.emission_rate,
            length: data.length,
            width: data.width,
            filter_mode: data.filter_mode,
            rows: data.rows,
            columns: data.columns,
            frame_flags: data.frame_flags,
            tail_length: data.tail_length,
            time: data.time,
            segment_color: data.segment_color,
            alpha: data.alpha,
            particle_scaling: data.particle_scaling,
            lifespan_uv_anim: data.lifespan_uv_anim,
            decay_uv_anim: data.decay_uv_anim,
            tail_uv_anim: data.tail_uv_anim,
            tail_decay_uv_anim: data.tail_decay_uv_anim,
            texture_id: data.texture_id,
            squirt: data.squirt,
            priority_plane: data.priority_plane,
            replaceable_id: data.replaceable_id,
            emission_rate_tracks: None,
            width_tracks: None,
            length_tracks: None,
            speed_tracks: None,
            latitude_tracks: None,
            gravity_tracks: None,
            variation_tracks: None,
            visibility: None,
        };

        while payload.limit() > 0 {
            match read_value(&mut payload)? {
                KP2V => result.visibility = Some(Tracks::read(&mut payload)?),
                KP2E => result.emission_rate_tracks = Some(Tracks::read(&mut payload)?),
                KP2W => result.width_tracks = Some(Tracks::read(&mut payload)?),
                KP2N => result.length_tracks = Some(Tracks::read(&mut payload)?),
                KP2S => result.speed_tracks = Some(Tracks::read(&mut payload)?),
                KP2L => result.latitude_tracks = Some(Tracks::read(&mut payload)?),
                KP2G => result.gravity_tracks = Some(Tracks::read(&mut payload)?),
                KP2R => result.variation_tracks = Some(Tracks::read(&mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported particle emitter 2 track",
                    ));
                }
            }
        }

        Ok(result)
    }

    pub fn read_list(reader: &mut impl Read, size: usize) -> io::Result<Vec<Self>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(&mut payload)?);
        }
        Ok(values)
    }

    pub fn color(&self, seg: usize) -> shared::color::Color32 {
        self.segment_color[seg].convert_to_color(self.alpha[seg])
    }

    pub fn get_emission_rate(&self, model: &Model, global_time: u32, frame: u32) -> f32 {
        self.emission_rate_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
            .unwrap_or(self.emission_rate)
    }

    pub fn get_speed(&self, model: &Model, global_time: u32, frame: u32) -> f32 {
        self.speed_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
            .unwrap_or(self.speed)
    }

    pub fn get_variation(&self, model: &Model, global_time: u32, frame: u32) -> f32 {
        self.variation_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
            .unwrap_or(self.variation)
    }

    pub fn get_latitude(&self, model: &Model, global_time: u32, frame: u32) -> f32 {
        self.latitude_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
            .unwrap_or(self.latitude)
    }

    pub fn get_gravity(&self, model: &Model, global_time: u32, frame: u32) -> f32 {
        self.gravity_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
            .unwrap_or(self.gravity)
    }

    pub fn get_width(&self, model: &Model, global_time: u32, frame: u32) -> f32 {
        self.width_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
            .unwrap_or(self.width)
    }

    pub fn get_length(&self, model: &Model, global_time: u32, frame: u32) -> f32 {
        self.length_tracks
            .as_ref()
            .and_then(|track| track.get_value(model, global_time, frame))
            .unwrap_or(self.length)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::mdx::TrackType;

    #[test]
    fn reads_particle_emitter_2_tracks() {
        assert_eq!(size_of::<ParticleEmitter2Data>(), 171);

        let mut bytes = 463_u32.to_le_bytes().to_vec();
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 80]);
        for value in [1_u32, 2, 3] {
            bytes.extend(value.to_le_bytes());
        }
        for value in 1..=8 {
            bytes.extend((value as f32).to_le_bytes());
        }
        for value in 9_u32..=12 {
            bytes.extend(value.to_le_bytes());
        }
        for value in [13_f32, 14.] {
            bytes.extend(value.to_le_bytes());
        }
        for value in 15..=23 {
            bytes.extend((value as f32).to_le_bytes());
        }
        bytes.extend([24, 25, 26]);
        for value in 27..=29 {
            bytes.extend((value as f32).to_le_bytes());
        }
        for value in 30_u32..=45 {
            bytes.extend(value.to_le_bytes());
        }
        for (tag, value) in [
            (b"KP2V", 46_f32),
            (b"KP2E", 47.),
            (b"KP2W", 48.),
            (b"KP2N", 49.),
            (b"KP2S", 50.),
            (b"KP2L", 51.),
            (b"KP2G", 52.),
            (b"KP2R", 53.),
        ] {
            bytes.extend(tag);
            for data in [1_u32, TrackType::NoInterp as u32, 0, 60] {
                bytes.extend(data.to_le_bytes());
            }
            bytes.extend(value.to_le_bytes());
        }

        let values = ParticleEmitter2::read_list(&mut Cursor::new(bytes), 463).unwrap();

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].speed, 1.);
        assert_eq!(values[0].alpha, [24, 25, 26]);
        assert_eq!(values[0].visibility.as_ref().unwrap().values[0].data, 46.);
        assert_eq!(
            values[0].variation_tracks.as_ref().unwrap().values[0].data,
            53.
        );
    }
}
