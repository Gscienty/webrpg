use web_sys::{WebGlProgram, WebGlUniformLocation};

pub struct Program {
    program_id: WebGlProgram,
    uniform_locations: Vec<Option<WebGlUniformLocation>>,
}

impl Program {
    pub fn new(
        program_id: WebGlProgram,
        uniform_locations: Vec<Option<WebGlUniformLocation>>,
    ) -> Self {
        Self {
            program_id,
            uniform_locations,
        }
    }

    pub fn id(&self) -> &WebGlProgram {
        &self.program_id
    }

    pub fn uniform_locations(&self) -> &[Option<WebGlUniformLocation>] {
        &self.uniform_locations
    }
}
