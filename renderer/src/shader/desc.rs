use web_sys::{WebGl2RenderingContext, WebGlShader};

use super::{Error, Program};

#[derive(Debug, Clone, Copy)]
pub enum UniformType {
    Float,
    FloatVec2,
    FloatVec3,
    FloatVec4,

    Color,

    Int,
    IntVec2,

    Bool,

    FloatMat3,
    FloatMat3Transpose,
    FloatMat4,

    Sampler2D,
    Sampler2DRect,
    Sampler2DArray,

    Count,
}

impl std::fmt::Display for UniformType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Float => "float",
            Self::FloatVec2 => "vec2",
            Self::FloatVec3 => "vec3",
            Self::FloatVec4 | Self::Color => "vec4",
            Self::Int => "int",
            Self::IntVec2 => "ivec2",
            Self::Bool => "bool",
            Self::FloatMat3 | Self::FloatMat3Transpose => "mat3",
            Self::FloatMat4 => "mat4",
            Self::Sampler2D => "sampler2D",
            Self::Sampler2DRect => "sampler2DRect",
            Self::Sampler2DArray => "sampler2DArray",
            _ => "float",
        })
    }
}

impl UniformType {
    pub const fn type_size(&self) -> usize {
        match self {
            Self::Float => size_of::<f32>(),
            Self::FloatVec2 => 2 * size_of::<f32>(),
            Self::FloatVec3 => 3 * size_of::<f32>(),
            Self::FloatVec4 | Self::Color => 4 * size_of::<f32>(),
            Self::Int => size_of::<i32>(),
            Self::IntVec2 => 2 * size_of::<i32>(),
            Self::Bool => size_of::<bool>(),
            Self::FloatMat3 | Self::FloatMat3Transpose => 9 * size_of::<f32>(),
            Self::FloatMat4 => 16 * size_of::<f32>(),
            Self::Sampler2D | Self::Sampler2DRect | Self::Sampler2DArray => size_of::<i32>(),
            _ => size_of::<f32>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AttributeID, UniformType};
    use crate::shader::{FOW_RAYCAST_SHADER, MODEL_SHADER, PARTICLE_SHADER};

    #[test]
    fn displays_glsl_type_names() {
        assert_eq!(UniformType::FloatVec3.to_string(), "vec3");
        assert_eq!(UniformType::Color.to_string(), "vec4");
        assert_eq!(UniformType::FloatMat3Transpose.to_string(), "mat3");
        assert_eq!(UniformType::Sampler2DArray.to_string(), "sampler2DArray");
    }

    #[test]
    fn attribute_locations_match_vertex_array_layout() {
        assert_eq!(AttributeID::Color as u32, 0);
        assert_eq!(AttributeID::Position as u32, 1);
        assert_eq!(AttributeID::Texcoord as u32, 2);
        assert_eq!(AttributeID::Skin as u32, 3);
        assert_eq!(AttributeID::BoneWeight as u32, 4);
        assert_eq!(AttributeID::Normal as u32, 5);
    }

    #[test]
    fn particle_shader_declares_particle_inputs_and_fog() {
        let vertex = PARTICLE_SHADER.generate_shader_source(true);
        let fragment = PARTICLE_SHADER.generate_shader_source(false);

        for input in ["a_size", "a_tail", "a_axis"] {
            assert!(vertex.contains(input));
        }
        assert!(fragment.contains("texture(u_fog_of_war, v_texcoord2)"));
        assert!(fragment.contains("u_alpha_cutoff"));
    }

    #[test]
    fn model_alpha_key_discards_transparent_fragments() {
        let fragment = MODEL_SHADER.generate_shader_source(false);

        assert!(fragment.contains("if (col.a < u_alpha_cutoff) discard;"));
    }

    #[test]
    fn fow_raycast_shader_declares_raycast_inputs() {
        let vertex = FOW_RAYCAST_SHADER.generate_shader_source(true);
        let fragment = FOW_RAYCAST_SHADER.generate_shader_source(false);

        assert!(vertex.contains("uniform vec2 u_eye_position"));
        assert!(vertex.contains("distance(eye.xy, a_position.xy) > 100.0"));
        assert!(fragment.contains("smoothstep(0.0, 0.2, v_texcoord.y)"));
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PrecisionType {
    Low,
    Medium,
    High,
    Default,
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum AttributeID {
    Color = 0,
    Position = 1,
    Texcoord = 2,
    Skin = 3,
    BoneWeight = 4,
    Normal = 5,
    ParticleAxis = 6,
    ParticleSize = 7,
    ParticleTail = 8,
    ParticleQuadRight = 9,
    ParticleQuadUp = 10,
    Instance = 11,
}

#[derive(Debug)]
pub struct Uniform {
    pub(crate) offset: usize,
    pub(crate) name: &'static str,
    pub(crate) typ: UniformType,
    precision: PrecisionType,
    pub(crate) count: usize,
    pub(crate) count_offset: usize,
    pub(crate) counted: bool,
}

#[derive(Debug)]
pub struct Attribute {
    name: &'static str,
    attrib: AttributeID,
    typ: UniformType,
}

#[derive(Debug)]
pub struct Varying {
    name: &'static str,
    typ: UniformType,
}

#[derive(Debug)]
pub struct Descriptor<const U: usize, const A: usize, const S: usize> {
    name: &'static str,
    uniforms: [Uniform; U],
    attributes: [Attribute; A],
    shared: [Varying; S],

    vertex_body: &'static str,
    fragment_body: &'static str,
}

pub trait ShaderDescriptor: Sync {
    fn get_uniforms(&self) -> &[Uniform];
}

impl Uniform {
    pub const fn new(
        offset: usize,
        count: usize,
        count_offset: usize,
        name: &'static str,
        typ: UniformType,
        precision: PrecisionType,
    ) -> Self {
        Self {
            offset,
            name,
            typ,
            precision,
            count,
            count_offset,
            counted: count != 0 && count_offset != 0,
        }
    }
}

#[macro_export]
macro_rules! uniform_declare {
    ($shader_type: ty, $field: ident, $typ: expr, $prec: expr) => {
        $crate::shader::Uniform::new(
            std::mem::offset_of!($shader_type, $field),
            0,
            0,
            stringify!($field),
            $typ,
            $prec,
        )
    };

    ($shader_type: ty, $field: ident, $typ: expr, $prec: expr, $count: expr) => {
        $crate::shader::Uniform::new(
            std::mem::offset_of!($shader_type, $field),
            $count,
            0,
            stringify!($field),
            $typ,
            $prec,
        )
    };

    ($shader_type: ty, $field: ident, $typ: expr, $prec: expr, $count: expr, $count_field: ident) => {
        $crate::shader::Uniform::new(
            std::mem::offset_of!($shader_type, $field),
            $count,
            std::mem::offset_of!($shader_type, $count_field),
            stringify!($field),
            $typ,
            $prec,
        )
    };
}

impl Attribute {
    pub const fn new(name: &'static str, attrib: AttributeID, typ: UniformType) -> Self {
        Self { name, attrib, typ }
    }
}

#[macro_export]
macro_rules! attribute_declare {
    ($field: ident, $attrib: expr, $typ: expr) => {
        $crate::shader::Attribute::new(stringify!($field), $attrib, $typ)
    };
}

impl Varying {
    pub const fn new(name: &'static str, typ: UniformType) -> Self {
        Self { name, typ }
    }
}

#[macro_export]
macro_rules! varying_declare {
    ($field: ident, $typ: expr) => {
        $crate::shader::Varying::new(stringify!($field), $typ)
    };
}

impl<const U: usize, const A: usize, const S: usize> Descriptor<U, A, S> {
    pub const fn new(
        name: &'static str,
        uniforms: [Uniform; U],
        attributes: [Attribute; A],
        shared: [Varying; S],
        vertex_body: &'static str,
        fragment_body: &'static str,
    ) -> Self {
        Self {
            name,
            uniforms,
            attributes,
            shared,
            vertex_body,
            fragment_body,
        }
    }

    fn generate_shader_source(&self, is_vertex: bool) -> String {
        let mut source =
            "#version 300 es\nprecision highp float;\nprecision highp int;\n".to_owned();

        for uniform in self.uniforms.iter() {
            if uniform.count > 1 {
                source += &format!(
                    "uniform {} u_{}[{}];\n",
                    uniform.typ, uniform.name, uniform.count
                );
            } else {
                source += &format!("uniform {} u_{};\n", uniform.typ, uniform.name);
            }
        }

        if is_vertex {
            for attribute in self.attributes.iter() {
                source += &format!("in {} a_{};\n", attribute.typ, attribute.name);
            }

            for shared in self.shared.iter() {
                source += &format!("out {} v_{};\n", shared.typ, shared.name);
            }

            source += self.vertex_body;

            source += "void main() { gl_Position = vert(); }\n";
        } else {
            for shared in self.shared.iter() {
                source += &format!("in {} v_{};\n", shared.typ, shared.name);
            }
            source += "out vec4 o_color;\n";

            source += self.fragment_body;

            source += "void main() { o_color = frag(); }\n"
        }

        source
    }

    fn create_vertex_shader(&self, gl: &WebGl2RenderingContext) -> Result<WebGlShader, Error> {
        let source = self.generate_shader_source(true);
        Self::create_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, &source)
    }

    fn create_fragment_shader(&self, gl: &WebGl2RenderingContext) -> Result<WebGlShader, Error> {
        let source = self.generate_shader_source(false);
        Self::create_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, &source)
    }

    fn create_shader(
        gl: &WebGl2RenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, Error> {
        let shader = gl
            .create_shader(shader_type)
            .ok_or(Error::CreateShaderFailed)?;

        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        if !gl
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let message = gl
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| "unknown error".to_owned());
            gl.delete_shader(Some(&shader));
            return Err(Error::CompileShaderFailed(message));
        }

        Ok(shader)
    }

    pub fn compile_program(&'static self, gl: &WebGl2RenderingContext) -> Result<Program, Error> {
        let vertex_shader = self.create_vertex_shader(gl)?;
        let fragment_shader = match self.create_fragment_shader(gl) {
            Ok(shader) => shader,
            Err(error) => {
                gl.delete_shader(Some(&vertex_shader));
                return Err(error);
            }
        };
        let program = match gl.create_program() {
            Some(program) => program,
            None => {
                gl.delete_shader(Some(&vertex_shader));
                gl.delete_shader(Some(&fragment_shader));
                return Err(Error::CreateProgramFailed);
            }
        };

        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        for attribute in &self.attributes {
            gl.bind_attrib_location(
                &program,
                attribute.attrib as u32,
                &format!("a_{}", attribute.name),
            );
        }
        gl.link_program(&program);

        let linked = gl
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false);
        let message = (!linked).then(|| {
            gl.get_program_info_log(&program)
                .unwrap_or_else(|| "unknown error".to_owned())
        });

        gl.delete_shader(Some(&vertex_shader));
        gl.delete_shader(Some(&fragment_shader));

        if let Some(message) = message {
            gl.delete_program(Some(&program));
            return Err(Error::LinkProgramFailed(message));
        }

        let uniform_locations = self
            .uniforms
            .iter()
            .map(|uniform| {
                let name = if uniform.count > 1 {
                    format!("u_{}[0]", uniform.name)
                } else {
                    format!("u_{}", uniform.name)
                };
                gl.get_uniform_location(&program, &name)
            })
            .collect();

        Ok(Program::new(self, program, uniform_locations))
    }
}

impl<const U: usize, const A: usize, const S: usize> ShaderDescriptor for Descriptor<U, A, S> {
    fn get_uniforms(&self) -> &[Uniform] {
        &self.uniforms
    }
}
