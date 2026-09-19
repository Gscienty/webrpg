use web_sys::WebGl2RenderingContext;

pub struct RenderStats {
    pub draws: usize,
    pub vertices: usize,
    pub triangles: usize,
    pub instances: usize,
}

impl RenderStats {
    fn primitive_triangles(mode: u32, count: usize, instances: usize) -> usize {
        if mode == WebGl2RenderingContext::TRIANGLES {
            (count / 3) * instances
        } else {
            0
        }
    }

    pub fn draw(&mut self, mode: u32, count: usize, instances: usize) {
        self.draws += 1;
        self.vertices += count * instances;
        self.triangles += Self::primitive_triangles(mode, count, instances);
        self.instances += instances;
    }
}

impl Default for RenderStats {
    fn default() -> Self {
        Self {
            draws: 0,
            vertices: 0,
            triangles: 0,
            instances: 0,
        }
    }
}
