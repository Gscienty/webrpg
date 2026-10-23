use std::io::Cursor;

use renderer::{
    shader::{Program, builtin::MODEL_SHADER},
    sprite::{Sprite, render_params::RenderParams, render_phase::RenderPhase},
    texture::Texture,
    texture_set::TextureSet,
};
use shared::{
    color::Color32,
    math3::{Frustum3, Matrix4, Vector3},
};
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};

#[wasm_bindgen]
pub fn list_mpq_files(name: &str, data: &[u8]) -> Result<String, JsValue> {
    let mut archive = webrpg_mpq::Archive::load(name, data.to_vec()).map_err(js_error)?;
    archive
        .list_files()
        .map(|files| files.join("\n"))
        .map_err(js_error)
}

#[wasm_bindgen]
pub struct MdxRenderer {
    gl: Gl,
    sprite: Sprite<'static>,
    texture_set: TextureSet,
    model_program: Program,
    animation_interval: [u32; 2],
    view_projection: Matrix4,
}

#[wasm_bindgen]
impl MdxRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas: HtmlCanvasElement,
        mpq_name: &str,
        mpq_data: &[u8],
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

        let mut archive =
            webrpg_mpq::Archive::load(mpq_name, mpq_data.to_vec()).map_err(js_error)?;
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
                radius * 20.,
            )
            .look_at(eye, center - eye, (0., 0., 1.).into());
        let animation_interval = model
            .sequences
            .iter()
            .find(|sequence| sequence.name.eq_ignore_ascii_case("Stand"))
            .or_else(|| model.sequences.first())
            .map(|sequence| sequence.interval)
            .unwrap_or_default();

        let mut texture_set = TextureSet::init(&gl, None).map_err(js_error)?;
        let sprite =
            Sprite::new(&gl, model, Some(&mut archive), &mut texture_set).map_err(js_error)?;
        let model_program = MODEL_SHADER.compile_program(&gl).map_err(js_error)?;

        Ok(Self {
            gl,
            sprite,
            texture_set,
            model_program,
            animation_interval,
            view_projection,
        })
    }

    pub fn render(&self, elapsed_ms: u32) -> Result<(), JsValue> {
        self.gl.clear_color(0.035, 0.045, 0.06, 1.);
        self.gl.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);

        let duration = (self.animation_interval[1] - self.animation_interval[0]).max(1);
        let frame = self.animation_interval[0] + elapsed_ms % duration;
        for render_phase in [RenderPhase::Solid, RenderPhase::Alpha] {
            self.sprite
                .render(
                    &self.gl,
                    &RenderParams {
                        origin: Vector3::default(),
                        angle: 0.,
                        scale: 1.,
                        no_frustum_cull: true,
                        no_lighting: true,
                        render_phase,
                        msaa_samples: 0,
                        lerpfrac: 0.,
                        tint: Color32::white(),
                        team: 0,
                        global_time: elapsed_ms,
                        frame,
                        previous_frame: frame,
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
                )
                .map_err(js_error)?;
        }
        Ok(())
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
