use std::{
    io::{self, Read},
    mem::size_of,
};

use shared::math3;

use crate::mdx::{model::Model, node::Node, track::Tracks, utils::read_value};

const KRAL: u32 = u32::from_le_bytes(*b"KRAL");
const KRCO: u32 = u32::from_le_bytes(*b"KRCO");
const KRHA: u32 = u32::from_le_bytes(*b"KRHA");
const KRHB: u32 = u32::from_le_bytes(*b"KRHB");
const KRTX: u32 = u32::from_le_bytes(*b"KRTX");
const KRVS: u32 = u32::from_le_bytes(*b"KRVS");

#[repr(C)]
struct RibbonEmitterData {
    height_above: f32,
    height_below: f32,
    alpha: f32,
    color: math3::Vector3,
    lifespan: f32,
    texture_slot: u32,
    emission_rate: u32,
    rows: u32,
    columns: u32,
    material_id: u32,
    gravity: f32,
}

#[derive(Debug)]
pub struct RibbonEmitter<'s> {
    pub node: Node<'s>,
    pub height_above: f32,
    pub height_below: f32,
    pub alpha: f32,
    pub color: math3::Vector3,
    pub lifespan: f32,
    pub texture_slot: u32,
    pub emission_rate: u32,
    pub rows: u32,
    pub columns: u32,
    pub material_id: u32,
    pub gravity: f32,

    pub visibility: Option<Tracks<'s, f32>>,
    pub height_above_tracks: Option<Tracks<'s, f32>>,
    pub height_below_tracks: Option<Tracks<'s, f32>>,
    pub alpha_tracks: Option<Tracks<'s, f32>>,
    pub color_tracks: Option<Tracks<'s, math3::Vector3>>,
    pub texture_tracks: Option<Tracks<'s, u32>>,
}

impl<'s> RibbonEmitter<'s> {
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
        let size: u32 = read_value(reader)?;
        let payload_size = (size as usize)
            .checked_sub(size_of::<u32>())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid ribbon emitter size")
            })?;
        let mut payload = reader.take(payload_size as u64);
        let node = Node::read(model, &mut payload)?;
        let data: RibbonEmitterData = read_value(&mut payload)?;
        let mut result = Self {
            node,
            height_above: data.height_above,
            height_below: data.height_below,
            alpha: data.alpha,
            color: data.color,
            lifespan: data.lifespan,
            texture_slot: data.texture_slot,
            emission_rate: data.emission_rate,
            rows: data.rows,
            columns: data.columns,
            material_id: data.material_id,
            gravity: data.gravity,
            visibility: None,
            height_above_tracks: None,
            height_below_tracks: None,
            alpha_tracks: None,
            color_tracks: None,
            texture_tracks: None,
        };

        while payload.limit() > 0 {
            match read_value(&mut payload)? {
                KRVS => result.visibility = Some(Tracks::read(model, &mut payload)?),
                KRHA => result.height_above_tracks = Some(Tracks::read(model, &mut payload)?),
                KRHB => result.height_below_tracks = Some(Tracks::read(model, &mut payload)?),
                KRAL => result.alpha_tracks = Some(Tracks::read(model, &mut payload)?),
                KRCO => result.color_tracks = Some(Tracks::read(model, &mut payload)?),
                KRTX => result.texture_tracks = Some(Tracks::read(model, &mut payload)?),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported ribbon emitter track",
                    ));
                }
            }
        }

        Ok(result)
    }

    pub fn read_list(model: &'s Model, reader: &mut impl Read, size: usize) -> io::Result<Vec<Self>> {
        let mut payload = reader.take(size as u64);
        let mut values = Vec::new();
        while payload.limit() > 0 {
            values.push(Self::read(model, &mut payload)?);
        }
        Ok(values)
    }
}
