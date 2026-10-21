use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

use crate::{
    error::Error,
    shader::desc::{ShaderDescriptor, UniformType},
};

#[derive(Clone)]
pub struct Program {
    desc: &'static dyn ShaderDescriptor,
    program_id: WebGlProgram,
    uniform_locations: Vec<Option<WebGlUniformLocation>>,
}

unsafe impl Sync for Program {}

impl Program {
    pub fn new(
        desc: &'static dyn ShaderDescriptor,
        program_id: WebGlProgram,
        uniform_locations: Vec<Option<WebGlUniformLocation>>,
    ) -> Self {
        Self {
            desc,
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

    pub fn apply<S>(&self, gl: &WebGl2RenderingContext, state: &S) -> Result<(), Error>
    where
        S: Sized,
    {
        gl.use_program(Some(&self.program_id));

        let state_ptr = (state as *const S).cast::<u8>();

        for (uniform, location) in self.desc.get_uniforms().iter().zip(&self.uniform_locations) {
            let Some(location) = location.as_ref() else {
                continue;
            };
            let count = if uniform.counted {
                let count_bytes = unsafe {
                    std::slice::from_raw_parts(
                        state_ptr.add(uniform.count_offset),
                        size_of::<u32>(),
                    )
                };
                let bytes: [u8; 4] = count_bytes
                    .try_into()
                    .map_err(|_| Error::InvalidUniformData(uniform.name))?;
                (u32::from_ne_bytes(bytes) as usize).min(uniform.count)
            } else {
                uniform.count.max(1)
            };
            if count == 0 {
                continue;
            }
            let length = uniform.typ.type_size() * count;
            let bytes =
                unsafe { std::slice::from_raw_parts(state_ptr.add(uniform.offset), length) };

            match uniform.typ {
                UniformType::Bool => gl.uniform1i(Some(location), i32::from(bytes[0] != 0)),
                UniformType::Int
                | UniformType::Sampler2D
                | UniformType::Sampler2DRect
                | UniformType::Sampler2DArray
                | UniformType::IntVec2 => {
                    let values: Vec<i32> = bytes
                        .chunks_exact(4)
                        .map(|chunk| {
                            let bytes: [u8; 4] = chunk
                                .try_into()
                                .map_err(|_| Error::InvalidUniformData(uniform.name))?;
                            Ok(i32::from_ne_bytes(bytes))
                        })
                        .collect::<Result<_, Error>>()?;
                    if matches!(uniform.typ, UniformType::IntVec2) {
                        gl.uniform2iv_with_i32_array(Some(location), &values);
                    } else {
                        gl.uniform1iv_with_i32_array(Some(location), &values);
                    }
                }
                typ => {
                    let values: Vec<f32> = bytes
                        .chunks_exact(4)
                        .map(|chunk| {
                            let bytes: [u8; 4] = chunk
                                .try_into()
                                .map_err(|_| Error::InvalidUniformData(uniform.name))?;
                            Ok(f32::from_ne_bytes(bytes))
                        })
                        .collect::<Result<_, Error>>()?;
                    match typ {
                        UniformType::Float | UniformType::Count => {
                            gl.uniform1fv_with_f32_array(Some(location), &values)
                        }
                        UniformType::FloatVec2 => {
                            gl.uniform2fv_with_f32_array(Some(location), &values)
                        }
                        UniformType::FloatVec3 => {
                            gl.uniform3fv_with_f32_array(Some(location), &values)
                        }
                        UniformType::FloatVec4 | UniformType::Color => {
                            gl.uniform4fv_with_f32_array(Some(location), &values)
                        }
                        UniformType::FloatMat3Transpose => {
                            let transposed = [
                                values[0], values[3], values[6], values[1], values[4], values[7],
                                values[2], values[5], values[8],
                            ];
                            gl.uniform_matrix3fv_with_f32_array(Some(location), false, &transposed);
                        }
                        UniformType::FloatMat3 => {
                            gl.uniform_matrix3fv_with_f32_array(Some(location), false, &values);
                        }
                        UniformType::FloatMat4 => {
                            gl.uniform_matrix4fv_with_f32_array(Some(location), false, &values);
                        }
                        _ => return Err(Error::InvalidUniformData(uniform.name)),
                    }
                }
            }
        }
        Ok(())
    }
}
