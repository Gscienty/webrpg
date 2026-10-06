use std::{
    io::{self, Read},
    mem::size_of,
};

use utils::{read_value, read_values};

mod utils;

mod bounds;
mod node;
mod track;

mod attachment;
mod bone;
mod camera;
mod collision_shape;
mod event;
mod geoset;
mod geoset_animation;
mod info;
mod light;
mod material;
mod particle_emitter;
mod particle_emitter2;
mod ribbon_emitter;
mod sequence;
mod texture;
mod texture_animation;

pub use bone::Bone;
pub use bounds::Bounds;
pub use track::{Frame, TrackType, Tracks};

pub use attachment::Attachment;
pub use camera::Camera;
pub use collision_shape::{CollisionShape, CollisionShapeType};
pub use event::{Event, EventTrack};
pub use geoset::Geoset;
pub use geoset_animation::GeosetAnimation;
pub use info::Info;
pub use light::{Light, LightType};
pub use material::{GeoFlag, Layer, Material};
pub use node::Node;
pub use particle_emitter::ParticleEmitter;
pub use particle_emitter2::ParticleEmitter2;
pub use ribbon_emitter::RibbonEmitter;
pub use sequence::Sequence;
pub use texture::Texture;
pub use texture_animation::TextureAnimation;

const ATCH: u32 = u32::from_le_bytes(*b"ATCH");
const BONE: u32 = u32::from_le_bytes(*b"BONE");
const CAMS: u32 = u32::from_le_bytes(*b"CAMS");
const CLID: u32 = u32::from_le_bytes(*b"CLID");
const EVTS: u32 = u32::from_le_bytes(*b"EVTS");
const GEOA: u32 = u32::from_le_bytes(*b"GEOA");
const GEOS: u32 = u32::from_le_bytes(*b"GEOS");
const GLBS: u32 = u32::from_le_bytes(*b"GLBS");
const HELP: u32 = u32::from_le_bytes(*b"HELP");
const LITE: u32 = u32::from_le_bytes(*b"LITE");
const MDLX: u32 = u32::from_le_bytes(*b"MDLX");
const MODL: u32 = u32::from_le_bytes(*b"MODL");
const MTLS: u32 = u32::from_le_bytes(*b"MTLS");
const PIVT: u32 = u32::from_le_bytes(*b"PIVT");
const PREM: u32 = u32::from_le_bytes(*b"PREM");
const PRE2: u32 = u32::from_le_bytes(*b"PRE2");
const RIBB: u32 = u32::from_le_bytes(*b"RIBB");
const SEQS: u32 = u32::from_le_bytes(*b"SEQS");
const TEXS: u32 = u32::from_le_bytes(*b"TEXS");
const TXAN: u32 = u32::from_le_bytes(*b"TXAN");
const VERS: u32 = u32::from_le_bytes(*b"VERS");

pub enum ModelChunk {
    Attachments(Vec<Attachment>),
    Bones(Vec<Bone>),
    Cameras(Vec<Camera>),
    CollisionShapes(Vec<CollisionShape>),
    Events(Vec<Event>),
    Geosets(Vec<Geoset>),
    GeosetAnimations(Vec<GeosetAnimation>),
    GlobalSequences(Vec<u32>),
    Helpers(Vec<Node>),
    Info(Info),
    Lights(Vec<Light>),
    Materials(Vec<Material>),
    PivotPoints(Vec<shared::math3::Vector3>),
    ParticleEmitters(Vec<ParticleEmitter>),
    ParticleEmitters2(Vec<ParticleEmitter2>),
    RibbonEmitters(Vec<RibbonEmitter>),
    Sequences(Vec<Sequence>),
    Textures(Vec<Texture>),
    TextureAnimations(Vec<TextureAnimation>),
    Version(u32),
}

impl ModelChunk {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let tag = read_value(reader)?;
        Self::read_tag(reader, tag)
    }

    fn read_next(reader: &mut impl Read) -> io::Result<Option<Self>> {
        let mut tag = [0; 4];
        if reader.read(&mut tag[..1])? == 0 {
            return Ok(None);
        }
        reader.read_exact(&mut tag[1..])?;
        Ok(Some(Self::read_tag(reader, u32::from_le_bytes(tag))?))
    }

    fn read_tag(reader: &mut impl Read, tag: u32) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let size = size as usize;

        let result = match tag {
            VERS => {
                if size != size_of::<u32>() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid version size",
                    ));
                }
                Self::Version(read_value(reader)?)
            }
            MODL => Self::Info(Info::read(reader, size)?),
            SEQS => Self::Sequences(Sequence::read_list(reader, size)?),
            GLBS => Self::GlobalSequences(read_values(reader, size / size_of::<u32>())?),
            MTLS => Self::Materials(Material::read_list(reader, size)?),
            TEXS => Self::Textures(Texture::read_list(reader, size)?),
            TXAN => Self::TextureAnimations(TextureAnimation::read_list(reader, size)?),
            GEOS => Self::Geosets(Geoset::read_list(reader, size)?),
            GEOA => Self::GeosetAnimations(GeosetAnimation::read_list(reader, size)?),
            BONE => Self::Bones(Bone::read_list(reader, size)?),
            HELP => Self::Helpers(Node::read_list(reader, size)?),
            ATCH => Self::Attachments(Attachment::read_list(reader, size)?),
            PIVT => Self::PivotPoints(read_values(
                reader,
                size / size_of::<shared::math3::Vector3>(),
            )?),
            PREM => Self::ParticleEmitters(ParticleEmitter::read_list(reader, size)?),
            PRE2 => Self::ParticleEmitters2(ParticleEmitter2::read_list(reader, size)?),
            RIBB => Self::RibbonEmitters(RibbonEmitter::read_list(reader, size)?),
            EVTS => Self::Events(Event::read_list(reader, size)?),
            CLID => Self::CollisionShapes(CollisionShape::read_list(reader, size)?),
            LITE => Self::Lights(Light::read_list(reader, size)?),
            CAMS => Self::Cameras(Camera::read_list(reader, size)?),

            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unsupported model chunk",
                ));
            }
        };

        Ok(result)
    }
}

#[derive(Debug)]
pub struct Model {
    pub version: u32,
    pub info: Info,

    pub sequences: Vec<Sequence>,
    pub global_sequences: Vec<u32>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub texture_animations: Vec<TextureAnimation>,
    pub geosets: Vec<Geoset>,
    pub geoset_animations: Vec<GeosetAnimation>,
    pub bones: Vec<Bone>,
    pub helpers: Vec<Node>,
    pub attachment: Vec<Attachment>,
    pub pivots: Vec<shared::math3::Vector3>,
    pub emitters: Vec<ParticleEmitter>,
    pub emitters2: Vec<ParticleEmitter2>,
    pub ribbons: Vec<RibbonEmitter>,
    pub events: Vec<Event>,
    pub collision_shapes: Vec<CollisionShape>,
    pub lights: Vec<Light>,
    pub cameras: Vec<Camera>,
}

impl Model {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let tag: u32 = read_value(reader)?;
        if tag != MDLX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid model tag",
            ));
        }

        let mut info = None;
        let mut version = None;
        let mut sequences = Vec::new();
        let mut global_sequences = Vec::new();
        let mut materials = Vec::new();
        let mut textures = Vec::new();
        let mut texture_animations = Vec::new();
        let mut geosets = Vec::new();
        let mut geoset_animations = Vec::new();
        let mut bones = Vec::new();
        let mut helpers = Vec::new();
        let mut attachment = Vec::new();
        let mut pivots = Vec::new();
        let mut emitters = Vec::new();
        let mut emitters2 = Vec::new();
        let mut ribbons = Vec::new();
        let mut events = Vec::new();
        let mut collision_shapes = Vec::new();
        let mut lights = Vec::new();
        let mut cameras = Vec::new();

        while let Some(chunk) = ModelChunk::read_next(reader)? {
            match chunk {
                ModelChunk::Version(value) => version = Some(value),
                ModelChunk::Info(value) => info = Some(value),
                ModelChunk::Sequences(value) => sequences = value,
                ModelChunk::GlobalSequences(value) => global_sequences = value,
                ModelChunk::Materials(value) => materials = value,
                ModelChunk::Textures(value) => textures = value,
                ModelChunk::TextureAnimations(value) => texture_animations = value,
                ModelChunk::Geosets(value) => geosets = value,
                ModelChunk::GeosetAnimations(value) => geoset_animations = value,
                ModelChunk::Bones(value) => bones = value,
                ModelChunk::Helpers(value) => helpers = value,
                ModelChunk::Attachments(value) => attachment = value,
                ModelChunk::PivotPoints(value) => pivots = value,
                ModelChunk::ParticleEmitters(value) => emitters = value,
                ModelChunk::ParticleEmitters2(value) => emitters2 = value,
                ModelChunk::RibbonEmitters(value) => ribbons = value,
                ModelChunk::Events(value) => events = value,
                ModelChunk::CollisionShapes(value) => collision_shapes = value,
                ModelChunk::Lights(value) => lights = value,
                ModelChunk::Cameras(value) => cameras = value,
            }
        }

        Ok(Self {
            version: version.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "mission model version")
            })?,
            info: info
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing model info"))?,
            sequences,
            global_sequences,
            materials,
            textures,
            texture_animations,
            geosets,
            geoset_animations,
            bones,
            helpers,
            attachment,
            pivots,
            emitters,
            emitters2,
            ribbons,
            events,
            collision_shapes,
            lights,
            cameras,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn reads_global_sequences() {
        let mut bytes = b"GLBS".to_vec();
        bytes.extend(8_u32.to_le_bytes());
        bytes.extend(100_u32.to_le_bytes());
        bytes.extend(200_u32.to_le_bytes());

        let ModelChunk::GlobalSequences(values) =
            ModelChunk::read(&mut Cursor::new(bytes)).unwrap()
        else {
            panic!("expected global sequences");
        };
        assert_eq!(values, [100, 200]);
    }

    #[test]
    fn reads_pivot_points() {
        let mut bytes = b"PIVT".to_vec();
        bytes.extend(24_u32.to_le_bytes());
        for value in [1_f32, 2., 3., 4., 5., 6.] {
            bytes.extend(value.to_le_bytes());
        }

        let ModelChunk::PivotPoints(values) = ModelChunk::read(&mut Cursor::new(bytes)).unwrap()
        else {
            panic!("expected pivot points");
        };
        assert_eq!(values, [(1., 2., 3.).into(), (4., 5., 6.).into()]);
    }

    #[test]
    fn reads_version() {
        let mut bytes = b"VERS".to_vec();
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(800_u32.to_le_bytes());

        let ModelChunk::Version(version) = ModelChunk::read(&mut Cursor::new(bytes)).unwrap()
        else {
            panic!("expected version");
        };
        assert_eq!(version, 800);
    }

    #[test]
    fn reads_model() {
        let mut bytes = b"MDLX".to_vec();
        bytes.extend(800_u32.to_le_bytes());
        bytes.extend(b"TestModel\0");
        bytes.extend(b"MODL");
        bytes.extend(372_u32.to_le_bytes());
        bytes.extend([0; 372]);
        bytes.extend(b"GLBS");
        bytes.extend(8_u32.to_le_bytes());
        bytes.extend(100_u32.to_le_bytes());
        bytes.extend(200_u32.to_le_bytes());

        let model = Model::read(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(model.version, 800);
        assert_eq!(model.global_sequences, [100, 200]);
        assert!(model.sequences.is_empty());
    }
}
