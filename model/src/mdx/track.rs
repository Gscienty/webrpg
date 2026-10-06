use std::io::{self, Read};

use crate::mdx::utils::read_value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TrackType {
    NoInterp = 0x00,
    Linear = 0x01,
    Hermite = 0x02,
    Bezier = 0x03,
}

impl TryFrom<u32> for TrackType {
    type Error = io::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoInterp),
            1 => Ok(Self::Linear),
            2 => Ok(Self::Hermite),
            3 => Ok(Self::Bezier),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid track type",
            )),
        }
    }
}

#[derive(Debug)]
pub struct Frame<T>
where
    T: Sized,
{
    pub time: u32,
    pub data: T,

    pub tan: Option<(T, T)>,
}

#[derive(Debug)]
pub struct Tracks<T>
where
    T: Sized,
{
    pub interpolation_type: TrackType,
    pub global_sequence_id: u32,
    pub values: Vec<Frame<T>>,
}

#[repr(C)]
struct TracksData {
    count: u32,
    interpolation_type: u32,
    global_sequence_id: u32,
}

impl<T> Frame<T>
where
    T: Sized,
{
    fn read(reader: &mut impl Read, interpolation_type: TrackType) -> io::Result<Self> {
        Ok(Self {
            time: read_value(reader)?,
            data: read_value(reader)?,
            tan: if matches!(interpolation_type, TrackType::Hermite | TrackType::Bezier) {
                Some((read_value(reader)?, read_value(reader)?))
            } else {
                None
            },
        })
    }
}

impl<T> Tracks<T>
where
    T: Sized,
{
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let data: TracksData = read_value(reader)?;
        let interpolation_type = TrackType::try_from(data.interpolation_type)?;
        let mut values = Vec::new();
        for _ in 0..data.count {
            values.push(Frame::read(reader, interpolation_type)?);
        }

        Ok(Self {
            interpolation_type,
            global_sequence_id: data.global_sequence_id,
            values,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn reads_key_track_and_optional_tangents() {
        let mut bytes = Vec::new();
        for value in [1_u32, TrackType::Linear as u32, 7, 10] {
            bytes.extend(value.to_le_bytes());
        }
        for value in [1_f32, 2., 3.] {
            bytes.extend(value.to_le_bytes());
        }

        let track = Tracks::<f32>::read(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(track.interpolation_type, TrackType::Linear);
        assert_eq!(track.global_sequence_id, 7);
        assert_eq!(track.values[0].time, 10);
        assert_eq!(track.values[0].data, 1.);
        assert_eq!(track.values[0].tan, Some((2., 3.)));
    }
}
