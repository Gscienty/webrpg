use std::io::{self, Read};

use crate::mdx::utils::read_value;

#[repr(C)]
#[derive(Debug, Default)]
pub struct Bounds {
    pub radius: f32,
    pub cube: shared::math3::Box3,
}

impl Bounds {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        read_value(reader)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn reads_little_endian_bounds() {
        let bytes: Vec<_> = (1..=7)
            .flat_map(|value| (value as f32).to_le_bytes())
            .collect();
        let bounds = Bounds::read(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(bounds.radius, 1.);
        assert_eq!(bounds.cube.min, (2., 3., 4.).into());
        assert_eq!(bounds.cube.max, (5., 6., 7.).into());
    }
}
