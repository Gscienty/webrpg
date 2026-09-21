#[derive(Default)]
pub struct CameraHeightMap {
    pub samples: Vec<f32>,
    pub width: u32,
    pub height: u32,
    pub origin: shared::math3::Vector2,
    pub cell_size: f32,
}

pub struct CameraHeightBuild<'a, T> {
    pub data: &'a T,
    pub width: u32,
    pub height_count: u32,
    pub radius: u32,
    pub samples: u32,
    pub origin: shared::math3::Vector2,
    pub cell_size: f32,
    pub get_height: &'a dyn Fn(&T, u32, u32) -> f32,
}

impl CameraHeightMap {
    pub fn build<T>(param: CameraHeightBuild<'_, T>) -> Option<Self> {
        if param.width == 0 || param.height_count == 0 || param.samples < 2 || param.cell_size == 0.
        {
            return None;
        }

        let len = (param.width * param.height_count) as usize;
        let mut map = Self {
            samples: Vec::new(),
            width: param.width,
            height: param.height_count,
            origin: param.origin,
            cell_size: param.cell_size,
        };
        if map.samples.try_reserve_exact(len).is_err() {
            return None;
        }
        map.samples.resize(len, 0.);

        let step = param.radius as f32 * 2. / (param.samples - 1) as f32;
        for y in 0..param.height_count {
            for x in 0..param.width {
                let mut sum = 0.;
                let mut count = 0_u64;
                for iy in 0..param.samples {
                    let sy = (y as i64 + (-(param.radius as f32) + iy as f32 * step).round() as i64)
                        .clamp(0, param.height_count as i64 - 1)
                        as u32;
                    for ix in 0..param.samples {
                        let sx = (x as i64
                            + (-(param.radius as f32) + ix as f32 * step).round() as i64)
                            .clamp(0, param.width as i64 - 1)
                            as u32;
                        sum += (param.get_height)(param.data, sx, sy);
                        count += 1;
                    }
                }
                map.samples[(x + y * param.width) as usize] = sum / count as f32;
            }
        }

        Some(map)
    }

    pub fn sample_camera_height_map(&self, x: f32, y: f32) -> f32 {
        let len = (self.width * self.height) as usize;
        if self.cell_size == 0. || self.width == 0 || self.height == 0 || self.samples.len() < len {
            return 0.;
        }

        let gx = ((x - self.origin[0]) / self.cell_size).clamp(0., self.width as f32 - 1.);
        let gy = ((y - self.origin[1]) / self.cell_size).clamp(0., self.height as f32 - 1.);
        let x0 = gx.floor() as usize;
        let y0 = gy.floor() as usize;
        let x1 = (x0 + 1).min(self.width as usize - 1);
        let y1 = (y0 + 1).min(self.height as usize - 1);
        let tx = gx - x0 as f32;
        let ty = gy - y0 as f32;
        let width = self.width as usize;

        let h0 = shared::math3::lerp_number(
            self.samples[x0 + y0 * width],
            self.samples[x1 + y0 * width],
            tx,
        );
        let h1 = shared::math3::lerp_number(
            self.samples[x0 + y1 * width],
            self.samples[x1 + y1 * width],
            tx,
        );
        shared::math3::lerp_number(h0, h1, ty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_blurred_height_samples_with_clamped_edges() {
        let heights = [0., 1., 2., 10., 11., 12., 20., 21., 22.];
        let get_height = |data: &[f32; 9], x, y| data[(x + y * 3) as usize];
        let map = CameraHeightMap::build(CameraHeightBuild {
            data: &heights,
            width: 3,
            height_count: 3,
            radius: 1,
            samples: 2,
            origin: (4., 5.).into(),
            cell_size: 2.,
            get_height: &get_height,
        })
        .expect("valid height map");

        assert_eq!(
            map.samples,
            [5.5, 6., 6.5, 10.5, 11., 11.5, 15.5, 16., 16.5]
        );
        assert_eq!(map.width, 3);
        assert_eq!(map.height, 3);
        assert_eq!(map.origin, (4., 5.).into());
        assert_eq!(map.cell_size, 2.);
    }

    #[test]
    fn rejects_invalid_build_parameters() {
        let get_height = |_: &(), _, _| 0.;
        let map = CameraHeightMap::build(CameraHeightBuild {
            data: &(),
            width: 1,
            height_count: 1,
            radius: 1,
            samples: 1,
            origin: Default::default(),
            cell_size: 1.,
            get_height: &get_height,
        });

        assert!(map.is_none());
    }

    #[test]
    fn samples_height_map_with_bilinear_interpolation_and_clamping() {
        let map = CameraHeightMap {
            samples: vec![0., 10., 20., 30.],
            width: 2,
            height: 2,
            origin: (100., 200.).into(),
            cell_size: 10.,
        };

        assert_eq!(map.sample_camera_height_map(105., 205.), 15.);
        assert_eq!(map.sample_camera_height_map(90., 250.), 20.);
        assert_eq!(map.sample_camera_height_map(200., 100.), 10.);
    }

    #[test]
    fn invalid_height_map_samples_as_zero() {
        let map = CameraHeightMap {
            samples: vec![1.],
            width: 2,
            height: 2,
            origin: Default::default(),
            cell_size: 1.,
        };

        assert_eq!(map.sample_camera_height_map(0., 0.), 0.);
    }
}
