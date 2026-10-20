use std::io::{self, Read};

use shared::math3;

use crate::mdx::{
    attachment::Attachment,
    bone::Bone,
    camera::Camera,
    collision_shape::CollisionShape,
    event::Event,
    geoset::Geoset,
    geoset_animation::GeosetAnimation,
    info::Info,
    light::Light,
    material::Material,
    model::Model,
    node::Node,
    particle_emitter::ParticleEmitter,
    particle_emitter2::ParticleEmitter2,
    ribbon_emitter::RibbonEmitter,
    sequence::Sequence,
    texture::Texture,
    texture_animation::TextureAnimation,
    utils::{read_value, read_values},
};

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

pub enum ModelChunk<'s> {
    Attachments(Vec<Attachment<'s>>),
    Bones(Vec<Bone<'s>>),
    Cameras(Vec<Camera<'s>>),
    CollisionShapes(Vec<CollisionShape<'s>>),
    Events(Vec<Event<'s>>),
    Geosets(Vec<Geoset<'s>>),
    GeosetAnimations(Vec<GeosetAnimation<'s>>),
    GlobalSequences(Vec<u32>),
    Helpers(Vec<Node<'s>>),
    Info(Info),
    Lights(Vec<Light<'s>>),
    Materials(Vec<Material<'s>>),
    PivotPoints(Vec<math3::Vector3>),
    ParticleEmitters(Vec<ParticleEmitter<'s>>),
    ParticleEmitters2(Vec<ParticleEmitter2<'s>>),
    RibbonEmitters(Vec<RibbonEmitter<'s>>),
    Sequences(Vec<Sequence>),
    Textures(Vec<Texture>),
    TextureAnimations(Vec<TextureAnimation<'s>>),
    Version(u32),
}

impl<'s> ModelChunk<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let tag = read_value(reader)?;
        Self::read_tag(model, reader, tag)
    }

    pub(super) fn read_next(model: &'s Model, reader: &mut impl Read) -> io::Result<Option<Self>> {
        let mut tag = [0; 4];
        if reader.read(&mut tag[..1])? == 0 {
            return Ok(None);
        }
        reader.read_exact(&mut tag[1..])?;
        Ok(Some(Self::read_tag(
            model,
            reader,
            u32::from_le_bytes(tag),
        )?))
    }

    fn read_tag(model: &'s Model, reader: &mut impl Read, tag: u32) -> io::Result<Self> {
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
            MTLS => Self::Materials(Material::read_list(model, reader, size)?),
            TEXS => Self::Textures(Texture::read_list(reader, size)?),
            TXAN => Self::TextureAnimations(TextureAnimation::read_list(model, reader, size)?),
            GEOS => Self::Geosets(Geoset::read_list(reader, size)?),
            GEOA => Self::GeosetAnimations(GeosetAnimation::read_list(model, reader, size)?),
            BONE => Self::Bones(Bone::read_list(model, reader, size)?),
            HELP => Self::Helpers(Node::read_list(model, reader, size)?),
            ATCH => Self::Attachments(Attachment::read_list(model, reader, size)?),
            PIVT => Self::PivotPoints(read_values(
                reader,
                size / size_of::<shared::math3::Vector3>(),
            )?),
            PREM => Self::ParticleEmitters(ParticleEmitter::read_list(model, reader, size)?),
            PRE2 => Self::ParticleEmitters2(ParticleEmitter2::read_list(model, reader, size)?),
            RIBB => Self::RibbonEmitters(RibbonEmitter::read_list(model, reader, size)?),
            EVTS => Self::Events(Event::read_list(model, reader, size)?),
            CLID => Self::CollisionShapes(CollisionShape::read_list(model, reader, size)?),
            LITE => Self::Lights(Light::read_list(model, reader, size)?),
            CAMS => Self::Cameras(Camera::read_list(model, reader, size)?),

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
