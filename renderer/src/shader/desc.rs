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

#[derive(Debug, Clone, Copy)]
pub enum PrecisionType {
    Low,
    Medium,
    High,
    Default,
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeID {
    Position,
    Color,
    Texcoord,
    Normal,
    Skin,
    BoneWeight,
    ParticleAxis,
    ParticleSize,
    ParticleTail,
    Instance,
}

#[derive(Debug)]
pub struct Uniform {
    offset: usize,
    name: &'static str,
    typ: UniformType,
    precision: PrecisionType,
    count: usize,
    count_offset: usize,
    counted: bool,
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
}
