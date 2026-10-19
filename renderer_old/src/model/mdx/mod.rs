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
pub use geoset::{Geoset, MDXGeoset};
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
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::{
    buffer,
    context::{self, Context},
    shader::{self, ModelState},
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
    Geosets(Vec<MDXGeoset>),
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
            GEOS => Self::Geosets(MDXGeoset::read_list(reader, size)?),
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

    pub vbo: WebGlBuffer,
    pub ibo: WebGlBuffer,
    pub uvbuffer: Option<WebGlBuffer>,
}

impl Model {
    pub fn parse(
        context: &mut context::Context,
        mpq: Option<&mut webrpg_mpq::Archive>,
        reader: &mut impl Read,
    ) -> Result<Self, io::Error> {
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
        let mut mdx_geosets = Vec::new();
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
                ModelChunk::Geosets(value) => mdx_geosets = value,
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
            let geoset = mdx_geosets
                .get_mut(animation.geoset_id as usize)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "invalid geoset animation geoset id",
                    )
                })?;
            geoset.anim = Some(sync::Arc::clone(animation));
        }

        if let Some(mpq) = mpq {
            for texture in &mut textures {
                if !texture.path.is_empty() {
                    texture.texture_id = Some(
                        context
                            .textures
                            .load_texture(&context.gl, mpq, &texture.path)
                            .map_err(io::Error::other)?,
                    );
                }
            }
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

        // calc bounds
        let mut min = [0_f32; 3];
        let mut max = [0_f32; 3];
        for vertex in mdx_geosets.iter().flat_map(|geoset| &geoset.vertices) {
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

        let uvcount = mdx_geosets.iter().fold(0, |acc, geoset| {
            if geoset.texcoord.is_empty() {
                return acc;
            }
            acc + ((geoset.texcoord.len() - 1) * geoset.vertices.len())
        });

        let vbo = context
            .gl
            .create_buffer()
            .ok_or(shader::Error::CreateBufferFailed)
            .map_err(io::Error::other)?;
        let ibo = context
            .gl
            .create_buffer()
            .ok_or(shader::Error::CreateBufferFailed)
            .map_err(io::Error::other)?;

        let uvbuffer = if uvcount != 0 {
            let uvbuffer = context
                .gl
                .create_buffer()
                .ok_or(shader::Error::CreateBufferFailed)
                .map_err(io::Error::other)?;

            Some(uvbuffer)
        } else {
            None
        };

        let mut index_base = 0;
        let mut vertex_base = 0;
        let mut uvbase = 0;
        let mut geosets = Vec::with_capacity(mdx_geosets.len());
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut uvs = Vec::with_capacity(uvcount);
        for geoset in mdx_geosets {
            uvs.extend(
                geoset
                    .texcoord
                    .iter()
                    .skip(1)
                    .flat_map(|channel| channel.iter().copied()),
            );
            let (geoset, vertices_chunk, indices_chunk) = Geoset::pack_geometry(
                &context.gl,
                &vbo,
                &ibo,
                uvbuffer.as_ref(),
                geoset,
                vertex_base,
                index_base,
                uvbase,
            )
            .map_err(io::Error::other)?;

            index_base += indices_chunk.len();
            vertex_base += vertices_chunk.len();
            if !geoset.property.texcoord.is_empty() {
                uvbase += (geoset.property.texcoord.len() - 1) * geoset.property.vertices.len();
            }

            geosets.push(geoset);

            vertices.extend(vertices_chunk);
            indices.extend(indices_chunk);
        }

        context
            .gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        context.gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            buffer::Vertices::from(vertices).as_bytes(),
            WebGl2RenderingContext::STATIC_DRAW,
        );

        if let Some(uvbuffer) = &uvbuffer {
            context
                .gl
                .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(uvbuffer));
            let uv_bytes = unsafe {
                std::slice::from_raw_parts(
                    uvs.as_ptr().cast::<u8>(),
                    std::mem::size_of_val(uvs.as_slice()),
                )
            };
            context.gl.buffer_data_with_u8_array(
                WebGl2RenderingContext::ARRAY_BUFFER,
                uv_bytes,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        context
            .gl
            .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ibo));
        let index_bytes = unsafe {
            std::slice::from_raw_parts(
                indices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(indices.as_slice()),
            )
        };
        context.gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            index_bytes,
            WebGl2RenderingContext::STATIC_DRAW,
        );

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
            vbo,
            ibo,
            uvbuffer,
        })
    }

    pub fn find_sequence_at_time(&self, time: u32) -> Option<&Sequence> {
        self.sequences
            .iter()
            .find(|&seq| seq.interval[0] <= time && time < seq.interval[1])
    }

    pub fn remap_animation(&self, frame: u32, suffix: &str) -> u32 {
        let Some(source) = self.find_sequence_at_time(frame) else {
            return frame;
        };
        let name = format!("{} {suffix}", source.name);
        let Some(target) = self
            .sequences
            .iter()
            .find(|sequence| sequence.name.eq_ignore_ascii_case(&name))
        else {
            return frame;
        };

        target.interval[0]
            + (frame - source.interval[0])
                .min((target.interval[1] - target.interval[0]).saturating_sub(1))
    }

    pub fn select_ui_sequence(&self, animation: &str) -> Option<&Sequence> {
        let sequence = animation
            .strip_prefix("#!")
            .or_else(|| animation.strip_prefix('#'))
            .unwrap_or(animation);

        let mut selected = if animation.starts_with('#') {
            sequence
                .split('@')
                .next()
                .and_then(|index| index.parse::<usize>().ok())
                .and_then(|index| self.sequences.get(index))
        } else if !animation.is_empty() {
            let name = animation.split('@').next().unwrap_or(animation);
            self.sequences
                .iter()
                .find(|sequence| sequence.name.eq_ignore_ascii_case(name))
        } else {
            None
        };

        if selected.is_none()
            && !self.cameras.is_empty()
            && matches!(animation, "Stand" | "Portrait" | "Portrait Talk")
        {
            let requested = if animation == "Stand" {
                "Portrait"
            } else {
                animation
            };

            for requested in [requested, "Portrait"] {
                selected = self.sequences.iter().find(|sequence| {
                    let Some(prefix) = sequence.name.get(..requested.len()) else {
                        return false;
                    };
                    if !prefix.eq_ignore_ascii_case(requested) {
                        return false;
                    }

                    let suffix = &sequence.name[requested.len()..];
                    suffix.is_empty()
                        || matches!(suffix.as_bytes()[0], b' ' | b'-')
                            && suffix
                                .bytes()
                                .all(|byte| matches!(byte, b' ' | b'-' | b'0'..=b'9'))
                });
                if selected.is_some() {
                    break;
                }
            }
        }

        selected.or_else(|| self.sequences.first())
    }

    pub fn collect_model_lights(
        &self,
        global_matrices: &BTreeMap<u32, shared::math3::Matrix4>,
        model_matrix: shared::math3::Matrix4,
        global_time: u32,
        frame: u32,
    ) -> Vec<shader::ModelLight> {
        self.lights
            .iter()
            .filter_map(|light| {
                light.evaluate_light(
                    self,
                    global_matrices,
                    model_matrix,
                    global_time,
                    frame,
                    true,
                )
            })
            .collect()
    }

    pub fn bone_matrices(
        &self,
        lerpfrac: f32,
        model_matrix: shared::math3::Matrix4,
        global_time: u32,
        frame1: u32,
        frame0: u32,
    ) -> BTreeMap<u32, shared::math3::Matrix4> {
        let local_matrices: BTreeMap<_, _> = self
            .nodes
            .iter()
            .map(|(&id, node)| {
                (
                    id,
                    self.calculate_node_matrix(lerpfrac, node, global_time, frame1, frame0),
                )
            })
            .collect();

        let mut global_matrices = BTreeMap::new();
        for node in self.nodes.values() {
            self.get_node_global_matrix(&local_matrices, &mut global_matrices, node, model_matrix);
        }
        global_matrices
    }

    fn get_node_global_matrix(
        &self,
        local_matrices: &BTreeMap<u32, shared::math3::Matrix4>,
        global_matrices: &mut BTreeMap<u32, shared::math3::Matrix4>,
        node: &Node,
        model_matrix: shared::math3::Matrix4,
    ) -> shared::math3::Matrix4 {
        if let Some(&matrix) = global_matrices.get(&node.object_id) {
            return matrix;
        }

        let local_matrix = local_matrices
            .get(&node.object_id)
            .copied()
            .unwrap_or_default();
        let parent = self.nodes.get(&node.parent_id);
        let mut global_matrix = if let Some(parent) = parent {
            self.get_node_global_matrix(local_matrices, global_matrices, parent, model_matrix)
                * local_matrix
        } else {
            local_matrix
        };

        if node.flags & 8 != 0 {
            let pivot = self
                .pivots
                .get(node.object_id as usize)
                .copied()
                .unwrap_or_default();
            let transformed_pivot = global_matrix * pivot;

            if let Some(parent) = parent {
                let parent_matrix = self.get_node_global_matrix(
                    local_matrices,
                    global_matrices,
                    parent,
                    model_matrix,
                );
                let rotation = shared::math3::Quaternion::from(parent_matrix);
                let inverse_rotation = -rotation;
                global_matrix = shared::math3::Matrix4::from_rotation_origin(
                    inverse_rotation,
                    transformed_pivot,
                ) * global_matrix;
            }

            let view_matrix = shared::math3::Matrix4::identity().rotate(
                shared::math3::Vector3::from((30_f32.to_radians(), 0., 90_f32.to_radians())),
                shared::math3::RotationOrder::XYZ,
            ) * model_matrix;
            let rotation = shared::math3::Quaternion::from(view_matrix);
            let inverse_rotation = -rotation;
            global_matrix =
                shared::math3::Matrix4::from_rotation_origin(inverse_rotation, transformed_pivot)
                    * global_matrix;
        }

        global_matrices.insert(node.object_id, global_matrix);
        global_matrix
    }

    fn calculate_node_matrix(
        &self,
        lerpfrac: f32,
        node: &Node,
        global_time: u32,
        frame1: u32,
        frame0: u32,
    ) -> shared::math3::Matrix4 {
        let mut translation = shared::math3::Vector3::default();
        let mut rotation = shared::math3::Quaternion::default();
        let mut scale = shared::math3::Vector3::from((1., 1., 1.));
        let pivot = self
            .pivots
            .get(node.object_id as usize)
            .copied()
            .unwrap_or_default();

        if frame0 != frame1 {
            if let Some(track) = &node.translation {
                let left = track
                    .get_value(self, global_time, frame0)
                    .unwrap_or_default();
                let right = track
                    .get_value(self, global_time, frame1)
                    .unwrap_or_default();
                translation = shared::math3::Vector3::lerp(left, right, lerpfrac);
            }
            if let Some(track) = &node.rotation {
                let left = track
                    .get_value(self, global_time, frame0)
                    .unwrap_or_default();
                let right = track
                    .get_value(self, global_time, frame1)
                    .unwrap_or_default();
                rotation = shared::math3::Quaternion::slerp(left, right, lerpfrac);
            }
            if let Some(track) = &node.scale {
                let left = track
                    .get_value(self, global_time, frame0)
                    .unwrap_or((1., 1., 1.).into());
                let right = track
                    .get_value(self, global_time, frame1)
                    .unwrap_or((1., 1., 1.).into());
                scale = shared::math3::Vector3::lerp(left, right, lerpfrac);
            }
        } else {
            translation = node
                .translation
                .as_ref()
                .and_then(|track| track.get_value(self, global_time, frame1))
                .unwrap_or_default();
            rotation = node
                .rotation
                .as_ref()
                .and_then(|track| track.get_value(self, global_time, frame1))
                .unwrap_or_default();
            scale = node
                .scale
                .as_ref()
                .and_then(|track| track.get_value(self, global_time, frame1))
                .unwrap_or((1., 1., 1.).into());
        }

        match (
            node.translation.is_some(),
            node.rotation.is_some(),
            node.scale.is_some(),
        ) {
            (false, false, false) => shared::math3::Matrix4::identity(),
            (true, false, false) => shared::math3::Matrix4::from_translation(translation),
            (false, true, false) => shared::math3::Matrix4::from_rotation_origin(rotation, pivot),
            _ => shared::math3::Matrix4::from_rotation_translation_scale_origin(
                rotation,
                translation,
                scale,
                pivot,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_geoset(
        &self,
        context: &Context,
        global_matrix: &BTreeMap<u32, shared::math3::Matrix4>,
        state: &mut ModelState,
        geoset: &Geoset,
        material: Option<&Material>,
        fog_enable: bool,
        team: u32,
        override_texture: Option<&'_ crate::texture::Texture>,
        override_slot: u32,
        force_unshaded: bool,
        global_time: u32,
        frame: u32,
        tint: shared::math3::Vector4,
        blended_pass: bool,
    ) {
        if let Some(material) = material
            && !material.has_pass(blended_pass)
        {
            return;
        }
        let geoset_color = geoset
            .property
            .evaluate_geoset_color(self, global_time, frame);
        state.geoset_color = (
            geoset_color[0] * tint[0],
            geoset_color[1] * tint[1],
            geoset_color[2] * tint[2],
            geoset_color[3] * tint[3],
        )
            .into();
        state.bind_geoset_matrix_palette(global_matrix, self, geoset);
        state.layer_alpha = 1.;

        if let Some(material) = material {
            for (id, layer) in material.layers.iter().enumerate() {
                if layer.is_blended() != blended_pass {
                    continue;
                }

                context.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
                state.alpha_key = false;

                if self.cameras.is_empty() {
                    context.gl.disable(WebGl2RenderingContext::CULL_FACE);
                } else {
                    context.gl.enable(WebGl2RenderingContext::CULL_FACE);
                    context.gl.cull_face(WebGl2RenderingContext::BACK);
                }
                context.gl.depth_mask(true);

                if !layer.set_layer_blend(context, state, id) {
                    continue;
                }

                if tint[3] < 1. - f32::EPSILON
                    && matches!(
                        layer.blend_mode,
                        shared::BlendMode::None | shared::BlendMode::AlphaKey
                    )
                {
                    state.alpha_key = false;
                    context.set_alpha_key_state(false);
                    context.gl.enable(WebGl2RenderingContext::BLEND);
                    context.gl.blend_func(
                        WebGl2RenderingContext::SRC_ALPHA,
                        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
                    );
                    context.gl.depth_mask(false);
                }

                layer.apply_flags(context);
                state.unshaded = force_unshaded || layer.flags & GeoFlag::Unshared as u32 != 0;
                state.fog_enable = fog_enable
                    && layer.flags & GeoFlag::Unfogged as u32 == 0
                    && matches!(
                        layer.blend_mode,
                        shared::BlendMode::None
                            | shared::BlendMode::AddAlpha
                            | shared::BlendMode::Blend
                    );
                let alpha = layer.evaluate_layer_alpha(material, self, global_time, frame);
                if alpha < f32::EPSILON {
                    continue;
                }
                state.layer_alpha = alpha;
                state.uv_matrix = layer.bind_texture_animation(self, global_time, frame);
                let texture_id = layer.evaluate_layer_texture_id(self, global_time, frame);
                let Some(model_texture) = self.textures.get(texture_id) else {
                    continue;
                };

                let Some(texture) = context.get_texture(
                    self,
                    team as usize,
                    texture_id,
                    model_texture.replaceable_id,
                    override_texture,
                    override_slot,
                ) else {
                    continue;
                };
                texture.bind(&context.gl, 0);

                let Some(vao) = geoset.vaos.get(layer.coord_id as usize) else {
                    continue;
                };
                context.gl.bind_vertex_array(Some(vao));
                context.render_stats.borrow_mut().draw(
                    WebGl2RenderingContext::TRIANGLES,
                    geoset.property.triangles.len(),
                    1,
                );
                if context
                    .shaders
                    .apply_model(&context.gl, false, state)
                    .is_err()
                {
                    continue;
                }
                context.gl.draw_elements_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    geoset.property.triangles.len() as i32,
                    WebGl2RenderingContext::UNSIGNED_INT,
                    geoset.index_offset as i32,
                );
            }
        }

        context.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        context.gl.enable(WebGl2RenderingContext::CULL_FACE);
        context.set_alpha_key_state(false);
        context.gl.cull_face(WebGl2RenderingContext::BACK);
        context.gl.depth_mask(true);
        state.alpha_key = false;
        state.unshaded = force_unshaded;
        state.layer_alpha = 1.;
        state.geoset_color = (1., 1., 1., 1.).into();
    }
}
