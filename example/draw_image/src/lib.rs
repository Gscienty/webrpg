use renderer::{context::Context, draw::DrawImage, texture::Texture};
use shared::color::Color32;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};

#[wasm_bindgen]
pub fn render_blp(canvas: HtmlCanvasElement, data: &[u8]) -> Result<(), JsValue> {
    render_image(canvas, "image.blp", data)
}

#[wasm_bindgen]
pub fn render_dds(canvas: HtmlCanvasElement, data: &[u8]) -> Result<(), JsValue> {
    render_image(canvas, "image.dds", data)
}

#[wasm_bindgen]
pub fn render_pcx(canvas: HtmlCanvasElement, data: &[u8]) -> Result<(), JsValue> {
    render_image(canvas, "image.pcx", data)
}

#[wasm_bindgen]
pub fn render_tga(canvas: HtmlCanvasElement, data: &[u8]) -> Result<(), JsValue> {
    render_image(canvas, "image.tga", data)
}

fn render_image(canvas: HtmlCanvasElement, filename: &str, data: &[u8]) -> Result<(), JsValue> {
    let gl = canvas
        .get_context("webgl2")?
        .ok_or_else(|| JsValue::from_str("WebGL2 is unavailable"))?
        .dyn_into::<Gl>()?;

    if filename.ends_with(".dds") {
        Texture::init(&gl).map_err(js_error)?;
    }

    let texture = Texture::load(&gl, filename, data).map_err(js_error)?;
    let context = Context::init(
        gl,
        canvas.width(),
        canvas.height(),
        canvas.width(),
        canvas.height(),
        None,
    )
    .map_err(js_error)?;

    context
        .gl
        .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
    context.gl.clear_color(0.08, 0.09, 0.11, 1.);
    context.gl.clear(Gl::COLOR_BUFFER_BIT);

    DrawImage::new(&texture, context.scene_rect(), None, Color32::white())
        .draw(&context)
        .map_err(js_error)
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
