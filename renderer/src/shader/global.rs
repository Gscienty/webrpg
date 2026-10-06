use std::any::Any;

use web_sys::WebGl2RenderingContext;

use crate::shader::{
    COMMANDBUTTON_SHADER, DEFAULT_SHADER, DefaultState, Error, FOW_RAYCAST_SHADER,
    MINIMAP_FOG_SHADER, MINIMAP_SHADER, MODEL_INSTANCED_SHADER, MODEL_SHADER, ModelState,
    PARTICLE_SHADER, Program, SHADOW_SPLAT_SHADER, SPLAT_SHADER, SpriteState, UNLIT_SHADER,
};

#[derive(Clone, Copy)]
pub enum ShaderType {
    Default,
    Splat,
    ShadowSplat,
    CommandButton,
    Minimap,
    MinimapFog,
    Unlit,
    Particle,
    FowRaycast,
    UI,
}

pub struct BuiltinShader {
    default_shader_prog: Program,

    model_instance_prog: Program,
    model_prog: Program,
    particle_prog: Program,
    fow_raycast_prog: Program,

    commandbutton_prog: Program,
    minimap_fog_prog: Program,
    minimap_prog: Program,
    shadow_splat_prog: Program,
    splat_prog: Program,
    unlit_prog: Program,
}

impl BuiltinShader {
    pub fn init(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        let default_shader_prog = DEFAULT_SHADER.compile_program(gl)?;

        let model_instance_prog = MODEL_INSTANCED_SHADER.compile_program(gl)?;
        let model_prog = MODEL_SHADER.compile_program(gl)?;
        let particle_prog = PARTICLE_SHADER.compile_program(gl)?;
        let fow_raycast_prog = FOW_RAYCAST_SHADER.compile_program(gl)?;

        let commandbutton_prog = COMMANDBUTTON_SHADER.compile_program(gl)?;
        let minimap_fog_prog = MINIMAP_FOG_SHADER.compile_program(gl)?;
        let minimap_prog = MINIMAP_SHADER.compile_program(gl)?;
        let shadow_splat_prog = SHADOW_SPLAT_SHADER.compile_program(gl)?;
        let splat_prog = SPLAT_SHADER.compile_program(gl)?;
        let unlit_prog = UNLIT_SHADER.compile_program(gl)?;

        Ok(Self {
            default_shader_prog,

            model_instance_prog,
            model_prog,
            particle_prog,
            fow_raycast_prog,

            commandbutton_prog,
            minimap_fog_prog,
            minimap_prog,
            shadow_splat_prog,
            splat_prog,
            unlit_prog,
        })
    }

    pub fn apply_model(
        &self,
        gl: &WebGl2RenderingContext,
        is_instance: bool,
        state: &ModelState,
    ) -> Result<(), Error> {
        if is_instance {
            self.model_instance_prog.apply(gl, state)
        } else {
            self.model_prog.apply(gl, state)
        }
    }

    pub fn apply<T>(
        &self,
        gl: &WebGl2RenderingContext,
        typ: ShaderType,
        state: &T,
    ) -> Result<(), Error>
    where
        T: Any + Sized,
    {
        match typ {
            ShaderType::Splat => self.splat_prog.apply(gl, state),
            ShaderType::ShadowSplat => self.shadow_splat_prog.apply(gl, state),
            ShaderType::MinimapFog => self.minimap_fog_prog.apply(gl, state),
            ShaderType::Minimap => self.minimap_prog.apply(gl, state),
            ShaderType::CommandButton => self.commandbutton_prog.apply(gl, state),
            ShaderType::Particle => self.particle_prog.apply(gl, state),
            ShaderType::FowRaycast => self.fow_raycast_prog.apply(gl, state),
            ShaderType::UI | ShaderType::Unlit => self.unlit_prog.apply(gl, state),
            ShaderType::Default => self.default_shader_prog.apply(gl, state),
        }
    }
}
