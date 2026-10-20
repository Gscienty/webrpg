use std::{
    collections::BTreeMap,
    io::{self, Read},
};

use shared::math3;

use crate::mdx::{
    attachment::Attachment, bone::Bone, bounds::Bounds, camera::Camera, chunk::ModelChunk,
    collision_shape::CollisionShape, event::Event, geoset::Geoset,
    geoset_animation::GeosetAnimation, info::Info, light::Light, material::Material, node::Node,
    particle_emitter::ParticleEmitter, particle_emitter2::ParticleEmitter2,
    ribbon_emitter::RibbonEmitter, sequence::Sequence, texture::Texture,
    texture_animation::TextureAnimation, utils::read_value,
};

const MDLX: u32 = u32::from_le_bytes(*b"MDLX");

#[derive(Debug, Default)]
pub struct Model<'s> {
    pub version: u32,
    pub info: Info,

    pub attachment: Vec<Attachment<'s>>,
    pub sequences: Vec<Sequence>,
    pub global_sequences: Vec<u32>,
    pub materials: Vec<Material<'s>>,
    pub textures: Vec<Texture>,
    pub texture_animations: Vec<TextureAnimation<'s>>,
    pub geoset_animations: Vec<GeosetAnimation<'s>>,
    pub geosets: Vec<Geoset<'s>>,
    pub bones: Vec<Bone<'s>>,
    pub helpers: Vec<Node<'s>>,
    pub pivots: Vec<math3::Vector3>,
    pub emitters: Vec<ParticleEmitter<'s>>,
    pub emitters2: Vec<ParticleEmitter2<'s>>,
    pub ribbons: Vec<RibbonEmitter<'s>>,
    pub events: Vec<Event<'s>>,
    pub collision_shapes: Vec<CollisionShape<'s>>,
    pub lights: Vec<Light<'s>>,
    pub cameras: Vec<Camera<'s>>,
    pub nodes: BTreeMap<u32, &'s Node<'s>>,
    pub bounds: Bounds,
}

impl<'s> Model<'s> {
    pub fn read(reader: &mut impl Read) -> Result<Self, io::Error> {
        let tag: u32 = read_value(reader)?;
        if tag != MDLX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid model tag",
            ));
        }
        let mut model = Self::default();

        while let Some(chunk) = ModelChunk::read_next(&model, reader)? {
            match chunk {
                ModelChunk::Version(value) => model.version = value,
                ModelChunk::Info(value) => model.info = value,
                ModelChunk::Sequences(value) => model.sequences = value,
                ModelChunk::GlobalSequences(value) => model.global_sequences = value,
                ModelChunk::Materials(value) => model.materials = value,
                ModelChunk::Textures(value) => model.textures = value,
                ModelChunk::TextureAnimations(value) => model.texture_animations = value,
                ModelChunk::Geosets(value) => model.geosets = value,
                ModelChunk::GeosetAnimations(value) => model.geoset_animations = value,
                ModelChunk::Bones(value) => model.bones = value,
                ModelChunk::Helpers(value) => model.helpers = value,
                ModelChunk::Attachments(value) => model.attachment = value,
                ModelChunk::PivotPoints(value) => model.pivots = value,
                ModelChunk::ParticleEmitters(value) => model.emitters = value,
                ModelChunk::ParticleEmitters2(value) => model.emitters2 = value,
                ModelChunk::RibbonEmitters(value) => model.ribbons = value,
                ModelChunk::Events(value) => model.events = value,
                ModelChunk::CollisionShapes(value) => model.collision_shapes = value,
                ModelChunk::Lights(value) => model.lights = value,
                ModelChunk::Cameras(value) => model.cameras = value,
            }
        }

        Ok(model)
    }
}
