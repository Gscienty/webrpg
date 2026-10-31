use std::io::{self, Read};

use shared::math3;

use crate::w3m::Vertex;

const VERTEX_BATCH: usize = 256;
const VERTEX_FILE_SIZE: usize = 7;

pub struct Map {
    pub header: u32,
    pub version: u32,
    pub tileset: u8,
    pub custom: u32,
    pub grounds: Vec<u32>,
    pub cliffs: Vec<u32>,
    pub center: math3::Vector2,
    pub width: u32,
    pub height: u32,
    pub vertices: Vec<Vertex>,
}

impl Map {
    pub fn read(reader: &mut impl Read) -> Result<Self, io::Error> {
        let header = read_u32(reader)?;
        let version = read_u32(reader)?;
        let tileset = read_u8(reader)?;
        let custom = read_u32(reader)?;
        let grounds = read_palette(reader)?;
        let cliffs = read_palette(reader)?;
        let width = read_u32(reader)?;
        let height = read_u32(reader)?;
        let center = (read_f32(reader)?, read_f32(reader)?).into();

        let vertex_count = (width * height) as usize;
        let mut vertices = Vec::new();
        vertices.try_reserve_exact(vertex_count).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "map vertex allocation is too large",
            )
        })?;
        let mut packed = [0; VERTEX_BATCH * VERTEX_FILE_SIZE];
        while vertices.len() < vertex_count {
            let count = VERTEX_BATCH.min(vertex_count - vertices.len());
            let bytes = &mut packed[..count * VERTEX_FILE_SIZE];
            reader.read_exact(bytes)?;
            for record in bytes.chunks_exact(VERTEX_FILE_SIZE) {
                let mut decoded = [0; VERTEX_FILE_SIZE];
                decoded.copy_from_slice(record);
                vertices.push(Vertex::from(decoded));
            }
        }

        Ok(Self {
            header,
            version,
            tileset,
            custom,
            grounds,
            cliffs,
            center,
            width,
            height,
            vertices,
        })
    }
}

fn read_palette(reader: &mut impl Read) -> io::Result<Vec<u32>> {
    let count = read_u32(reader)?;

    (0..count).map(|_| read_u32(reader)).collect()
}

fn read_u8(reader: &mut impl Read) -> io::Result<u8> {
    let mut bytes = [0];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0])
}

fn read_u32(reader: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_f32(reader: &mut impl Read) -> io::Result<f32> {
    read_u32(reader).map(f32::from_bits)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn u32_bytes(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend(value.to_le_bytes());
    }

    fn valid_map() -> Vec<u8> {
        let mut bytes = Vec::new();
        u32_bytes(&mut bytes, u32::from_le_bytes(*b"W3E!"));
        u32_bytes(&mut bytes, 11);
        bytes.push(b'L');
        u32_bytes(&mut bytes, 1);
        u32_bytes(&mut bytes, 1);
        u32_bytes(&mut bytes, u32::from_le_bytes(*b"Ldrt"));
        u32_bytes(&mut bytes, 1);
        u32_bytes(&mut bytes, u32::from_le_bytes(*b"CLdi"));
        u32_bytes(&mut bytes, 2);
        u32_bytes(&mut bytes, 2);
        u32_bytes(&mut bytes, 12.5_f32.to_bits());
        u32_bytes(&mut bytes, (-4.25_f32).to_bits());
        for _ in 0..4 {
            bytes.extend([0x34, 0x12, 0x45, 0xc3, 0xed, 0xb5, 0xa7]);
        }
        bytes
    }

    #[test]
    fn reads_map_and_decodes_vertex_flags() {
        let bytes = valid_map();
        let mut reader = Cursor::new(&bytes);
        let map = Map::read(&mut reader).unwrap();

        assert_eq!(map.header, u32::from_le_bytes(*b"W3E!"));
        assert_eq!(map.version, 11);
        assert_eq!(map.tileset, b'L');
        assert_eq!(map.grounds, [u32::from_le_bytes(*b"Ldrt")]);
        assert_eq!(map.cliffs, [u32::from_le_bytes(*b"CLdi")]);
        assert_eq!(map.center, math3::Vector2::from((12.5, -4.25)));
        let vertex = &map.vertices[0];
        assert_eq!(vertex.accurate_height, 0x1234);
        assert_eq!(vertex.water_level, 0x0345);
        assert_eq!(vertex.map_edge, 1);
        assert_eq!(vertex.ground, 13);
        assert_eq!(
            (vertex.ramp, vertex.blight, vertex.water, vertex.boundary),
            (0, 1, 1, 1)
        );
        assert_eq!((vertex.ground_variation, vertex.cliff_variation), (21, 5));
        assert_eq!((vertex.level, vertex.cliff), (7, 10));
        assert_eq!(reader.position(), bytes.len() as u64);
    }

    #[test]
    fn rejects_short_reads() {
        let mut bytes = valid_map();
        bytes.pop();

        assert_eq!(
            Map::read(&mut Cursor::new(bytes)).err().unwrap().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn rejects_invalid_dimensions_before_allocating_vertices() {
        let mut bytes = valid_map();
        bytes[29..33].copy_from_slice(&1_u32.to_le_bytes());

        assert_eq!(
            Map::read(&mut Cursor::new(bytes)).err().unwrap().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
