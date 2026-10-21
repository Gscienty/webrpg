use std::{
    collections::BTreeMap,
    io::{self, Read},
    marker::PhantomPinned,
    pin::Pin,
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

#[derive(Clone, Copy, Debug)]
enum NodeIndex {
    Bone(usize),
    Helper(usize),
    Attachment(usize),
    Emitter(usize),
    Emitter2(usize),
    Ribbon(usize),
    Event(usize),
    CollisionShape(usize),
    Light(usize),
}

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

    nodes: BTreeMap<u32, NodeIndex>,
    pub bounds: Bounds,

    _pin: PhantomPinned,
}

impl<'s> Model<'s> {
    pub fn read(reader: &mut impl Read) -> Result<Pin<Box<Self>>, io::Error> {
        let tag: u32 = read_value(reader)?;
        if tag != MDLX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid model tag",
            ));
        }
        let mut model = Box::pin(Self::default());
        // Tracks borrow the model they belong to, so parsing must happen after its address is fixed.
        let model_ptr = unsafe { Pin::as_mut(&mut model).get_unchecked_mut() as *mut Self };

        while let Some(chunk) = ModelChunk::read_next(unsafe { &*model_ptr }, reader)? {
            let model = unsafe { &mut *model_ptr };
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
        unsafe { Self::assign_geoset_animations(model_ptr)? };

        let parsed = unsafe { &*model_ptr };
        let nodes = parsed.indexed_nodes()?;
        let bounds = parsed.calculate_bounds();
        unsafe { std::ptr::addr_of_mut!((*model_ptr).nodes).replace(nodes) };
        unsafe { std::ptr::addr_of_mut!((*model_ptr).bounds).replace(bounds) };

        Ok(model)
    }

    unsafe fn assign_geoset_animations(model_ptr: *mut Self) -> io::Result<()> {
        let animation_count = unsafe { (*model_ptr).geoset_animations.len() };
        for animation_index in 0..animation_count {
            let animation_ptr =
                unsafe { (*model_ptr).geoset_animations.as_ptr().add(animation_index) };
            let geoset_index = unsafe { (*animation_ptr).geoset_id as usize };
            let geoset_count = unsafe { (*model_ptr).geosets.len() };
            if geoset_index >= geoset_count {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid geoset animation geoset id",
                ));
            }

            let geoset_ptr = unsafe { (*model_ptr).geosets.as_mut_ptr().add(geoset_index) };
            unsafe {
                std::ptr::addr_of_mut!((*geoset_ptr).anim).replace(Some(&*animation_ptr));
            }
        }
        Ok(())
    }

    fn indexed_nodes(&self) -> io::Result<BTreeMap<u32, NodeIndex>> {
        let indexed_nodes = self
            .bones
            .iter()
            .enumerate()
            .map(|(index, value)| (NodeIndex::Bone(index), &value.node))
            .chain(
                self.helpers
                    .iter()
                    .enumerate()
                    .map(|(index, node)| (NodeIndex::Helper(index), node)),
            )
            .chain(
                self.attachment
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (NodeIndex::Attachment(index), &value.node)),
            )
            .chain(
                self.emitters
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (NodeIndex::Emitter(index), &value.node)),
            )
            .chain(
                self.emitters2
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (NodeIndex::Emitter2(index), &value.node)),
            )
            .chain(
                self.ribbons
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (NodeIndex::Ribbon(index), &value.node)),
            )
            .chain(
                self.events
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (NodeIndex::Event(index), &value.node)),
            )
            .chain(
                self.collision_shapes
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (NodeIndex::CollisionShape(index), &value.node)),
            )
            .chain(
                self.lights
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (NodeIndex::Light(index), &value.node)),
            );
        let mut nodes = BTreeMap::new();
        for (index, node) in indexed_nodes {
            if nodes.insert(node.object_id, index).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "duplicate model node id",
                ));
            }
        }
        Ok(nodes)
    }

    fn calculate_bounds(&self) -> Bounds {
        let mut min = [0_f32; 3];
        let mut max = [0_f32; 3];
        for vertex in self.geosets.iter().flat_map(|geoset| &geoset.vertices) {
            for axis in 0..3 {
                min[axis] = min[axis].min(vertex[axis]);
                max[axis] = max[axis].max(vertex[axis]);
            }
        }
        Bounds {
            radius: (0..3).map(|axis| max[axis] - min[axis]).fold(0., f32::max),
            cube: (
                (min[0], min[1], min[2]).into(),
                (max[0], max[1], max[2]).into(),
            )
                .into(),
        }
    }

    pub fn node(&self, object_id: u32) -> Option<&Node<'s>> {
        self.nodes
            .get(&object_id)
            .copied()
            .map(|index| self.node_at(index))
    }

    pub fn nodes(&self) -> impl Iterator<Item = (u32, &Node<'s>)> + '_ {
        self.nodes
            .iter()
            .map(|(&object_id, &index)| (object_id, self.node_at(index)))
    }

    fn node_at(&self, index: NodeIndex) -> &Node<'s> {
        match index {
            NodeIndex::Bone(index) => &self.bones[index].node,
            NodeIndex::Helper(index) => &self.helpers[index],
            NodeIndex::Attachment(index) => &self.attachment[index].node,
            NodeIndex::Emitter(index) => &self.emitters[index].node,
            NodeIndex::Emitter2(index) => &self.emitters2[index].node,
            NodeIndex::Ribbon(index) => &self.ribbons[index].node,
            NodeIndex::Event(index) => &self.events[index].node,
            NodeIndex::CollisionShape(index) => &self.collision_shapes[index].node,
            NodeIndex::Light(index) => &self.lights[index].node,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn push_node(bytes: &mut Vec<u8>, name: &str, object_id: u32) {
        bytes.extend_from_slice(&96_u32.to_le_bytes());
        let mut node_name = [0; 80];
        node_name[..name.len()].copy_from_slice(name.as_bytes());
        bytes.extend_from_slice(&node_name);
        bytes.extend_from_slice(&object_id.to_le_bytes());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
    }

    #[test]
    fn reads_model_without_moving_it() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"MDLX");
        bytes.extend_from_slice(b"VERS");
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&800_u32.to_le_bytes());
        bytes.extend_from_slice(b"GLBS");
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&100_u32.to_le_bytes());

        let model = Model::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(model.version, 800);
        assert_eq!(model.global_sequences, [100]);
    }

    #[test]
    fn tracks_keep_using_the_pinned_model() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"MDLX");
        bytes.extend_from_slice(b"GLBS");
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&100_u32.to_le_bytes());
        bytes.extend_from_slice(b"TXAN");
        bytes.extend_from_slice(&36_u32.to_le_bytes());
        bytes.extend_from_slice(&36_u32.to_le_bytes());
        bytes.extend_from_slice(b"KTAT");
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&0_i32.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&1_f32.to_le_bytes());
        bytes.extend_from_slice(&2_f32.to_le_bytes());
        bytes.extend_from_slice(&3_f32.to_le_bytes());

        let model = Model::read(&mut Cursor::new(bytes)).unwrap();
        let track = model.texture_animations[0].translation.as_ref().unwrap();
        assert_eq!(
            track.get_value(50, 0),
            Some(math3::Vector3::from((1.0, 2.0, 3.0)))
        );
    }

    #[test]
    fn indexes_nodes_without_storing_internal_references() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"MDLX");
        bytes.extend_from_slice(b"HELP");
        bytes.extend_from_slice(&192_u32.to_le_bytes());
        push_node(&mut bytes, "First", 3);
        push_node(&mut bytes, "Second", 7);

        let model = Model::read(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(&*model.node(7).unwrap().name, "Second");
        assert_eq!(
            model
                .nodes()
                .map(|(object_id, _)| object_id)
                .collect::<Vec<_>>(),
            [3, 7]
        );
    }

    #[test]
    fn rejects_duplicate_node_ids() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"MDLX");
        bytes.extend_from_slice(b"HELP");
        bytes.extend_from_slice(&192_u32.to_le_bytes());
        push_node(&mut bytes, "First", 3);
        push_node(&mut bytes, "Duplicate", 3);

        let error = Model::read(&mut Cursor::new(bytes)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn assigns_geoset_animation() {
        let mut model = Box::pin(Model::default());
        let model_ptr = {
            let model = unsafe { Pin::as_mut(&mut model).get_unchecked_mut() };
            model.geosets.push(Geoset {
                vertices: Vec::new(),
                normals: Vec::new(),
                texcoord: Vec::new(),
                primitive_types: Vec::new(),
                primitive_counts: Vec::new(),
                triangles: Vec::new(),
                vertex_groups: Vec::new(),
                matrix_group_sizes: Vec::new(),
                matrices: Vec::new(),
                material_id: 0,
                group: 0,
                selectable: 0,
                default_bounds: Bounds::default(),
                bounds: Vec::new(),
                anim: None,
            });
            model.geoset_animations.push(GeosetAnimation {
                alpha: 1.0,
                flags: 0,
                color: (1.0, 1.0, 1.0).into(),
                geoset_id: 0,
                geoset_alphas: None,
                geoset_colors: None,
            });
            model as *mut Model
        };

        unsafe { Model::assign_geoset_animations(model_ptr).unwrap() };

        assert!(std::ptr::eq(
            model.geosets[0].anim.unwrap(),
            &model.geoset_animations[0]
        ));
    }
}
