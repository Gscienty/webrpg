use std::io::Cursor;

use renderer::{
    context::Context,
    model::mdx::Model,
    render_entity::{RenderEntity, RenderEntityFlag},
    view::{View, ViewRDFlag},
};
use shared::math3::{Matrix4, Rect, Vector3};
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
    context: Context,
    model: Model,
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
        let mut context =
            Context::init(gl, width, height, width, height, None).map_err(js_error)?;
        let mut archive =
            webrpg_mpq::Archive::load(mpq_name, mpq_data.to_vec()).map_err(js_error)?;
        let mdx = archive.read_file(mdx_path).map_err(js_error)?;
        let model = Model::parse(&mut context, Some(&mut archive), &mut Cursor::new(mdx))
            .map_err(js_error)?;

        let center = model.bounds.box_.center();
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

        Ok(Self {
            context,
            model,
            view_projection,
        })
    }

    pub fn render(&mut self, elapsed_ms: u32) {
        self.context.gl.clear_color(0.035, 0.045, 0.06, 1.);
        self.context
            .gl
            .clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);

        let frame = self
            .model
            .select_ui_sequence("Stand")
            .map(|sequence| {
                let duration = (sequence.interval[1] - sequence.interval[0]).max(1);
                sequence.interval[0] + elapsed_ms % duration
            })
            .unwrap_or(0);
        let entity = RenderEntity {
            frame,
            previous_frame: frame,
            flags: RenderEntityFlag::NoFogOfWar as u32
                | RenderEntityFlag::NoShadow as u32
                | RenderEntityFlag::NoLighting as u32,
            ..RenderEntity::new(&self.model)
        };
        let mut view = View {
            viewport: Rect::new(0., 0., 1., 1.),
            scissor: Rect::new(0., 0., 1., 1.),
            view_projection_matrix: self.view_projection,
            light_matrix: Matrix4::identity(),
            entities: vec![entity],
            time: elapsed_ms,
            rdflags: ViewRDFlag::NoWorldModel as u32 | ViewRDFlag::NoFrustumCull as u32,
            ..Default::default()
        };
        view.render(&mut self.context);
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
