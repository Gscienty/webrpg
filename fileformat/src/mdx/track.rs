use std::io::{self, Read};

use shared;

use crate::mdx::{model::Model, sequence::Sequence, utils::read_value};

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
    T: Sized + Copy + Clone + Default + shared::Interp,
{
    pub time: u32,
    pub data: T,

    pub tan: Option<(T, T)>,
}

#[derive(Debug)]
pub struct Tracks<'s, T>
where
    T: Sized + Copy + Clone + Default + shared::Interp,
{
    pub interpolation_type: TrackType,
    pub global_sequence_id: i32,
    pub values: Vec<Frame<T>>,

    model: &'s Model<'s>,
}

#[repr(C)]
struct TracksData {
    count: u32,
    interpolation_type: u32,
    global_sequence_id: i32,
}

impl<T> Frame<T>
where
    T: Sized + Copy + Clone + Default + shared::Interp,
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

    fn get_out_tan_value(&self) -> T {
        let Some(value) = self.tan else {
            return T::default();
        };

        value.1
    }

    fn get_in_tan_value(&self) -> T {
        let Some(value) = self.tan else {
            return T::default();
        };

        value.0
    }

    fn eval_key_frame_value(left: &Self, right: &Self, t: f32, interpolation_type: TrackType) -> T {
        match interpolation_type {
            TrackType::NoInterp => left.data,
            TrackType::Bezier => shared::Interp::bezier(
                left.data,
                left.get_out_tan_value(),
                right.get_in_tan_value(),
                right.data,
                t,
            ),
            TrackType::Hermite => shared::Interp::hermite(
                left.data,
                left.get_out_tan_value(),
                right.get_in_tan_value(),
                right.data,
                t,
            ),
            _ => shared::Interp::lerp(left.data, right.data, t),
        }
    }

    fn get_key_frame_value(
        left: &Self,
        right: &Self,
        time: u32,
        interpolation_type: TrackType,
    ) -> T {
        if right.time == left.time {
            return left.data;
        }

        Self::eval_key_frame_value(
            left,
            right,
            (time - left.time) as f32 / (right.time - left.time) as f32,
            interpolation_type,
        )
    }
}

impl<'s, T> Tracks<'s, T>
where
    T: Sized + Copy + Clone + Default + shared::Interp,
{
    pub fn read(model: &'s Model, reader: &mut impl Read) -> io::Result<Self> {
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

            model,
        })
    }

    pub fn get_value(&self, global_time: u32, mut time: u32) -> Option<T> {
        if self.values.is_empty() {
            return None;
        }

        let mut interval = [0u32; 2];

        if self.global_sequence_id != -1 {
            if self.global_sequence_id as usize >= self.model.global_sequences.len() {
                return None;
            }

            interval[0] = 0;
            interval[1] = self.model.global_sequences[self.global_sequence_id as usize];

            if self.values[0].time > interval[1] {
                return Some(self.values[0].data);
            }

            let gs_len = interval[1] + 1;
            time = if gs_len > 0 { global_time % gs_len } else { 0 };
        } else {
            let seq = self.model.find_sequence_at_time(time)?;
            interval = seq.interval;
        }

        let first_index = self.key_frame_bound(interval[0], false);
        let end_index = self.key_frame_bound(interval[1], true);

        if first_index >= end_index {
            return None;
        }

        let first = &self.values[first_index];
        let last = &self.values[end_index - 1];

        if time >= last.time {
            let span = (interval[1] - last.time) + (first.time - interval[0]);
            if first_index != end_index - 1 && span != 0 && time < interval[1] {
                return Some(Frame::<T>::eval_key_frame_value(
                    last,
                    first,
                    (time - last.time) as f32 / span as f32,
                    self.interpolation_type,
                ));
            }
            return Some(last.data);
        }

        let right_index = self.key_frame_bound(time, false);
        let right = &self.values[right_index];

        if right_index == first_index || right.time == time {
            return Some(right.data);
        }

        let left = &self.values[right_index - 1];

        Some(Frame::<T>::get_key_frame_value(
            left,
            right,
            time,
            self.interpolation_type,
        ))
    }

    pub fn key_frame_bound(&self, time: u32, upper: bool) -> usize {
        let mut left = 0;
        let mut right = self.values.len();

        while left < right {
            let mid = (left + right) / 2;
            let mid_time = self.values[mid].time;

            if mid_time < time || (upper && mid_time == time) {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        left
    }
}

impl<'s> Model<'s> {
    fn find_sequence_at_time(&self, time: u32) -> Option<&Sequence> {
        self.sequences
            .iter()
            .find(|&seq| seq.interval[0] <= time && time < seq.interval[1])
    }
}
