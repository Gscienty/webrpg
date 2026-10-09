use std::{
    collections::BTreeMap,
    io::{self, Read},
    mem::size_of,
    sync,
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
    GeosetAnimations(Vec<sync::Arc<GeosetAnimation>>),
    GlobalSequences(Vec<u32>),
    Helpers(Vec<sync::Arc<Node>>),
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
    pub geoset_animations: Vec<sync::Arc<GeosetAnimation>>,
    pub bones: Vec<Bone>,
    pub helpers: Vec<sync::Arc<Node>>,
    pub attachment: Vec<Attachment>,
    pub pivots: Vec<shared::math3::Vector3>,
    pub emitters: Vec<ParticleEmitter>,
    pub emitters2: Vec<ParticleEmitter2>,
    pub ribbons: Vec<RibbonEmitter>,
    pub events: Vec<Event>,
    pub collision_shapes: Vec<CollisionShape>,
    pub lights: Vec<Light>,
    pub cameras: Vec<Camera>,

    pub nodes: BTreeMap<u32, sync::Arc<Node>>,
    pub bounds: Bounds,
}

impl Model {
    pub fn parse(
        reader: &mut impl Read,
        context: &mut renderer::context::Context,
        mpq: Option<&mut webrpg_mpq::Archive>,
    ) -> io::Result<Self> {
        let mut model = Self::read(reader)?;
        let Some(mpq) = mpq else {
            return Ok(model);
        };

        for texture in &mut model.textures {
            if !texture.path.is_empty() {
                texture.texture_id = Some(
                    context
                        .textures
                        .load_texture(&context.gl, mpq, &texture.path)
                        .map_err(io::Error::other)?,
                );
            }
        }
        Ok(model)
    }

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

        for animation in &geoset_animations {
            let geoset = geosets
                .get_mut(animation.geoset_id as usize)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid geoset animation geoset id",
                    )
                })?;
            geoset.anim = Some(sync::Arc::clone(animation));
        }

        let nodes = bones
            .iter()
            .map(|value| &value.node)
            .chain(helpers.iter())
            .chain(attachment.iter().map(|value| &value.node))
            .chain(emitters.iter().map(|value| &value.node))
            .chain(emitters2.iter().map(|value| &value.node))
            .chain(ribbons.iter().map(|value| &value.node))
            .chain(events.iter().map(|value| &value.node))
            .chain(collision_shapes.iter().map(|value| &value.node))
            .chain(lights.iter().map(|value| &value.node))
            .map(|node| (node.object_id, sync::Arc::clone(node)))
            .collect();

        let mut min = [0_f32; 3];
        let mut max = [0_f32; 3];
        for vertex in geosets.iter().flat_map(|geoset| &geoset.vertices) {
            for axis in 0..3 {
                min[axis] = min[axis].min(vertex[axis]);
                max[axis] = max[axis].max(vertex[axis]);
            }
        }
        let bounds = Bounds {
            radius: (0..3).map(|axis| max[axis] - min[axis]).fold(0., f32::max),
            box_: (
                (min[0], min[1], min[2]).into(),
                (max[0], max[1], max[2]).into(),
            )
                .into(),
        };

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
            nodes,
            bounds,
        })
    }

    pub fn find_sequence_at_time(&self, time: u32) -> Option<&Sequence> {
        self.sequences
            .iter()
            .find(|&seq| seq.interval[0] <= time && time < seq.interval[1])
    }

    pub fn build_buffers(
        gl: &web_sys::WebGl2RenderingContext,
        geosets: &mut [Geoset],
    ) -> Result<renderer::buffer::RenderBuffer, renderer::shader::Error> {
        let (vertices, indices) = Self::pack_geometry(geosets);
        let buffer = vertices.new_indexed_array_object(gl, &indices)?;
        let ibo = buffer.ibo.as_ref().unwrap();
        let mut base = 0;
        for geoset in geosets {
            geoset.buffer = Some(renderer::buffer::Vertices::create_shared_array_object(
                gl,
                &buffer.vbo,
                ibo,
                base,
            )?);
            base += geoset.vertices.len();
        }
        Ok(buffer)
    }

    fn pack_geometry(geosets: &[Geoset]) -> (renderer::buffer::Vertices, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for geoset in geosets {
            let (geoset_vertices, geoset_indices) = geoset.pack_geometry();
            vertices.extend(geoset_vertices);
            indices.extend(geoset_indices);
        }
        (vertices.into(), indices)
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
        bytes.extend(b"VERS");
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(800_u32.to_le_bytes());
        bytes.extend(b"MODL");
        bytes.extend(372_u32.to_le_bytes());
        bytes.extend([0; 372]);
        bytes.extend(b"GEOA");
        bytes.extend(28_u32.to_le_bytes());
        bytes.extend(28_u32.to_le_bytes());
        bytes.extend(0.5_f32.to_le_bytes());
        bytes.extend(1_u32.to_le_bytes());
        for value in [1_f32, 2., 3.] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(b"GEOS");
        bytes.extend(160_u32.to_le_bytes());
        bytes.extend(160_u32.to_le_bytes());
        for tag in [b"PTYP", b"GNDX", b"NRMS", b"PVTX", b"MTGC", b"PCNT"] {
            bytes.extend(tag);
            bytes.extend(0_u32.to_le_bytes());
        }
        bytes.extend(b"VRTX");
        bytes.extend(2_u32.to_le_bytes());
        for value in [-2_f32, 1., 3., 4., -5., 2.] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(b"UVAS");
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(b"UVBS");
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(0.25_f32.to_le_bytes());
        bytes.extend(0.75_f32.to_le_bytes());
        bytes.extend(b"MATS");
        bytes.extend(0_u32.to_le_bytes());
        for value in [1_u32, 2, 3] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend([0; 28]);
        bytes.extend(0_u32.to_le_bytes());
        bytes.extend(b"HELP");
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend(96_u32.to_le_bytes());
        bytes.extend([0; 80]);
        for value in [7_u32, u32::MAX, 0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(b"GLBS");
        bytes.extend(8_u32.to_le_bytes());
        bytes.extend(100_u32.to_le_bytes());
        bytes.extend(200_u32.to_le_bytes());

        let model = Model::read(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(model.version, 800);
        assert_eq!(model.global_sequences, [100, 200]);
        assert!(model.sequences.is_empty());
        assert!(sync::Arc::ptr_eq(&model.helpers[0], &model.nodes[&7]));
        assert!(sync::Arc::ptr_eq(
            model.geosets[0].anim.as_ref().unwrap(),
            &model.geoset_animations[0]
        ));
        assert_eq!(model.bounds.box_.min, (-2., -5., 0.).into());
        assert_eq!(model.bounds.box_.max, (4., 1., 3.).into());
        assert_eq!(model.bounds.radius, 6.);
    }
}
