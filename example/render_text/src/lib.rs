use fontdue::{Font, FontSettings};
use renderer::{font::Span, global::Global};
use shared::color::Color32;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl};

#[wasm_bindgen]
pub struct TextRenderer {
    gl: Gl,
    global: Global,
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
        let global = Global::init(&gl, canvas.width(), canvas.height()).map_err(js_error)?;

        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        Ok(Self { gl, global, font })
    }

    pub fn render(
        &self,
        text: &str,
        size: f32,
        red: u8,
        green: u8,
        blue: u8,
    ) -> Result<(), JsValue> {
        self.gl.clear_color(0.08, 0.09, 0.11, 1.);
        self.gl.clear(Gl::COLOR_BUFFER_BIT);

        let mut span = Span::new(&self.font);
        span.set_size(size);
        span.set_color(Color32 {
            red,
            green,
            blue,
            alpha: 255,
        });
        span.write(text);
        span.draw(&self.global, &self.gl, 24., 24.)
            .map_err(js_error)
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
