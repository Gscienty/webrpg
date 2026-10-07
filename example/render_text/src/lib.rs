use fontdue::{Font, FontSettings};
use renderer::{context::Context, draw::Span};
use shared::color::Color32;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};

#[wasm_bindgen]
pub struct TextRenderer {
    context: Context,
    font: Font,
}

#[wasm_bindgen]
impl TextRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement, font_data: &[u8]) -> Result<TextRenderer, JsValue> {
        let gl = canvas
            .get_context("webgl2")?
            .ok_or_else(|| JsValue::from_str("WebGL2 is unavailable"))?
            .dyn_into::<Gl>()?;
        let font = Font::from_bytes(font_data, FontSettings::default())
            .map_err(|error| JsValue::from_str(&format!("invalid font: {error}")))?;
        let width = canvas.width();
        let height = canvas.height();
        let context = Context::init(gl, width, height, width, height, None).map_err(js_error)?;

        context.gl.viewport(0, 0, width as i32, height as i32);

        Ok(Self { context, font })
    }

    pub fn render(
        &self,
        text: &str,
        size: f32,
        red: u8,
        green: u8,
        blue: u8,
    ) -> Result<(), JsValue> {
        self.context.gl.clear_color(0.08, 0.09, 0.11, 1.);
        self.context.gl.clear(Gl::COLOR_BUFFER_BIT);

        let mut span = Span::new(&self.font);
        span.set_size(size);
        span.set_color(Color32 {
            red,
            green,
            blue,
            alpha: 255,
        });
        span.write(text);
        span.draw(&self.context, 24., 24.).map_err(js_error)
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
