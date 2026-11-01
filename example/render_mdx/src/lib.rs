use std::io::Cursor;

use js_sys::{Array, Uint8Array};
use renderer::{
    particle, ribbon,
    shader::{ParticleState, Program, builtin::MODEL_SHADER},
    sky::Sky,
    sprite::{RenderParams, RenderState, Sprite, render_phase::RenderPhase},
    texture::Texture,
    texture_set::TextureSet,
};
use shared::{
    color::Color32,
    math3::{Frustum3, Matrix4, Vector3},
};
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};
use webrpg_mpq::Archive as _;

const SKY_PATH: &str = "Environment\\Sky\\LordaeronSummerSky\\LordaeronSummerSky.mdx";

#[wasm_bindgen]
pub fn list_mpq_files(name: &str, data: &[u8]) -> Result<String, JsValue> {
    let mut archive = webrpg_mpq::MpqArchive::load(name, data.to_vec()).map_err(js_error)?;
    archive
        .list_files()
        .map(|files| files.join("\n"))
        .map_err(js_error)
}

#[wasm_bindgen]
pub struct MdxRenderer {
    gl: Gl,
    sprite: Sprite<'static>,
    sky: Option<Sky<'static>>,
    texture_set: TextureSet,
    model_program: Program,
    animations: Vec<(String, [u32; 2])>,
    animation_interval: [u32; 2],
    ribbon_state: ribbon::Instance,
    particle_state: particle::Resource,
    particle_render: particle::ParticleRender,
    particle_shader_state: ParticleState,
    last_elapsed_ms: u32,
    team: u32,
    eye: Vector3,
    view_projection: Matrix4,
}

#[wasm_bindgen]
impl MdxRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas: HtmlCanvasElement,
        mpq_names: Array,
        mpq_data: Array,
        mdx_path: &str,
    ) -> Result<MdxRenderer, JsValue> {
        let gl = canvas
            .get_context("webgl2")?
            .ok_or_else(|| JsValue::from_str("WebGL2 is unavailable"))?
            .dyn_into::<Gl>()?;
        let width = canvas.width();
        let height = canvas.height();
        gl.viewport(0, 0, width as i32, height as i32);
        gl.enable(Gl::DEPTH_TEST);
        Texture::init(&gl).map_err(js_error)?;

        let has_war3 = mpq_names
            .iter()
            .filter_map(|name| name.as_string())
            .any(|name| name.eq_ignore_ascii_case("war3.mpq"));
        let mut archive = load_archives(&mpq_names, &mpq_data)?;
        let mdx = archive.read_file(mdx_path).map_err(js_error)?;
        let model = fileformat::mdx::Model::read(&mut Cursor::new(mdx)).map_err(js_error)?;

        let center = model.bounds.cube.center();
        let radius = model.bounds.radius.max(32.);
        let eye = center + Vector3::from((radius * 1.8, -radius * 1.8, radius * 1.3));
        let view_projection = Matrix4::identity()
            .perspective(
                45_f32.to_radians(),
                width as f32 / height.max(1) as f32,
                (radius * 0.01).max(0.1),
                (radius * 20.).max(10_000.),
            )
            .look_at(eye, center - eye, (0., 0., 1.).into());
        let animations: Vec<_> = model
            .sequences
            .iter()
            .map(|sequence| (sequence.name.to_string(), sequence.interval))
            .collect();
        let animation_interval = animations
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Stand"))
            .or_else(|| animations.first())
            .map(|(_, interval)| *interval)
            .unwrap_or_default();

        let mut texture_set = if has_war3 {
            TextureSet::init(&gl, Some(&mut archive))
        } else {
            TextureSet::init(&gl, None)
        }
        .map_err(js_error)?;
        let sprite =
            Sprite::new(&gl, model, Some(&mut archive), &mut texture_set).map_err(js_error)?;
        let sky = if has_war3 {
            let sky_data = archive.read_file(SKY_PATH).map_err(js_error)?;
            let sky_model =
                fileformat::mdx::Model::read(&mut Cursor::new(sky_data)).map_err(js_error)?;
            Some(Sky::new(&gl, sky_model, Some(&mut archive), &mut texture_set).map_err(js_error)?)
        } else {
            None
        };
        let ribbon_state = sprite.create_ribbon_instance(1);
        let particle_state = particle::Resource::default();
        let particle_render = particle::ParticleRender::new(&gl).map_err(js_error)?;
        let particle_shader_state = ParticleState {
            view_projection,
            texture_matrix: Matrix4::identity(),
            model: Matrix4::identity(),
            eye,
            ..Default::default()
        };
        let model_program = MODEL_SHADER.compile_program(&gl).map_err(js_error)?;

        Ok(Self {
            gl,
            sprite,
            sky,
            texture_set,
            model_program,
            animations,
            animation_interval,
            ribbon_state,
            particle_state,
            particle_render,
            particle_shader_state,
            last_elapsed_ms: 0,
            team: 1,
            eye,
            view_projection,
        })
    }

    pub fn animation_names(&self) -> String {
        self.animations
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn select_animation(&mut self, index: usize) -> Result<(), JsValue> {
        self.animation_interval = self
            .animations
            .get(index)
            .map(|(_, interval)| *interval)
            .ok_or_else(|| JsValue::from_str("invalid animation index"))?;
        Ok(())
    }

    pub fn select_team(&mut self, team: u32) -> Result<(), JsValue> {
        if team >= renderer::texture_set::MAX_TEAMS as u32 {
            return Err(JsValue::from_str("invalid team index"));
        }
        self.team = team;
        self.ribbon_state.team = team;
        Ok(())
    }

    pub fn render(&mut self, elapsed_ms: u32) -> Result<(), JsValue> {
        self.gl.clear_color(0.035, 0.045, 0.06, 1.);
        self.gl.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);

        let duration = (self.animation_interval[1] - self.animation_interval[0]).max(1);
        let frame = self.animation_interval[0] + elapsed_ms % duration;
        let delta = elapsed_ms.saturating_sub(self.last_elapsed_ms);
        self.last_elapsed_ms = elapsed_ms;
        if let Some(sky) = &self.sky {
            sky.render(
                &self.gl,
                self.eye,
                elapsed_ms,
                self.view_projection,
                &self.texture_set,
                &self.model_program,
            )
            .map_err(js_error)?;
        }
        for render_phase in [RenderPhase::Solid, RenderPhase::Alpha] {
            self.sprite
                .render(
                    &self.gl,
                    RenderParams {
                        origin: Vector3::default(),
                        angle: 0.,
                        scale: 1.,
                        no_frustum_cull: true,
                        no_face_cull: true,
                        no_lighting: true,
                        no_depth_test: false,
                        no_depth_write: false,
                        not_selectable: false,
                        emission_paused: false,
                        render_phase,
                        msaa_samples: 0,
                        lerpfrac: 0.,
                        tint: Color32::white(),
                        team: self.team,
                        global_time: elapsed_ms,
                        frame,
                        previous_frame: frame,
                        delta,
                        view_projection: self.view_projection,
                        texture: Matrix4::identity(),
                        light: Matrix4::identity(),
                        fog: None,
                        override_texture: None,
                        override_slot: 0,
                        frustum: Frustum3::calculate(self.view_projection),
                        texture_set: &self.texture_set,
                        model_program: &self.model_program,
                    },
                    Some(RenderState {
                        ribbon: &mut self.ribbon_state,
                        particle: &mut self.particle_state,
                    }),
                )
                .map_err(js_error)?;
        }
        self.texture_set.white.bind(&self.gl, 4);
        self.particle_state
            .draw(
                &self.gl,
                &self.particle_render,
                &mut self.particle_shader_state,
                delta,
            )
            .map_err(js_error)?;
        Ok(())
    }
}

fn load_archives(names: &Array, data: &Array) -> Result<webrpg_mpq::MultiArchive, JsValue> {
    if names.length() == 0 || names.length() != data.length() {
        return Err(JsValue::from_str("invalid MPQ archive set"));
    }

    let mut archives: Vec<Box<dyn webrpg_mpq::Archive>> =
        Vec::with_capacity(names.length() as usize);
    for index in 0..names.length() {
        let name = names
            .get(index)
            .as_string()
            .ok_or_else(|| JsValue::from_str("invalid MPQ name"))?;
        let bytes = Uint8Array::new(&data.get(index)).to_vec();
        let archive = webrpg_mpq::MpqArchive::load(name, bytes).map_err(js_error)?;
        archives.push(Box::new(archive));
    }
    Ok(webrpg_mpq::MultiArchive::new(archives))
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
