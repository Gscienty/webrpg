use crate::shader::Program;

pub struct Global {
    default_shader_prog: Program,

    model_instance_prog: Program,
    model_prog: Program,

    commonbutton_prog: Program,
    minimap_fog_prog: Program,
    minimap_prog: Program,
    shadow_splat_prog: Program,
    splat_prog: Program,
    unlit_prog: Program,
}
