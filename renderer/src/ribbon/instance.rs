use std::collections::BTreeMap;

use fileformat::mdx;
use shared::math3;

use crate::{ribbon::Trail, shader};

pub struct Instance {
    pub team: u32,
    pub stamp: u32,

    pub trails: Vec<Trail>,
}

impl Instance {
    pub fn new(team: u32) -> Self {
        Self {
            team,
            stamp: 0,

            trails: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit(
        &mut self,
        index: usize,
        ribbon: &mdx::RibbonEmitter,
        transform: math3::Matrix4,
        bone_matrices: &BTreeMap<u32, math3::Matrix4>,
        global_time: u32,
        frame: u32,
        delta: u32,
    ) -> Option<shader::Vertices> {
        if self.trails.len() <= index {
            self.trails.resize_with(index + 1, Trail::default);
        }
        let trail = self.trails.get_mut(index)?;

        let (stamp, vertices) =
            trail.emit(ribbon, transform, bone_matrices, global_time, frame, delta);

        self.stamp = stamp;

        Some(vertices)
    }
}
