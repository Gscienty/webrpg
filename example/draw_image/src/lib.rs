use fileformat::image::Image;
use renderer::texture::Texture;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl, WebGlProgram, WebGlShader};

const VERTEX_SHADER: &str = r#"#version 300 es
out vec2 v_uv;

void main() {
    vec2 positions[4] = vec2[4](
        vec2(-1.0, -1.0),
        vec2( 1.0, -1.0),
        vec2(-1.0,  1.0),
        vec2( 1.0,  1.0)
    );
    vec2 uvs[4] = vec2[4](
        vec2(0.0, 1.0),
        vec2(1.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 0.0)
    );
    gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
    v_uv = uvs[gl_VertexID];
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 300 es
precision mediump float;

uniform sampler2D u_texture;
in vec2 v_uv;
out vec4 out_color;

void main() {
    out_color = texture(u_texture, v_uv);
}
"#;

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

    let image = Image::load(filename, data).map_err(js_error)?;
    let texture = Texture::new(&gl, image).map_err(js_error)?;
    let program = match create_program(&gl) {
        Ok(program) => program,
        Err(error) => {
            texture.drop(&gl);
            return Err(error);
        }
    };

    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
    gl.clear_color(0.08, 0.09, 0.11, 1.);
    gl.clear(Gl::COLOR_BUFFER_BIT);
    gl.use_program(Some(&program));
    texture.bind(&gl, 0);
    gl.uniform1i(gl.get_uniform_location(&program, "u_texture").as_ref(), 0);
    gl.draw_arrays(Gl::TRIANGLE_STRIP, 0, 4);

    gl.delete_program(Some(&program));
    texture.drop(&gl);
    Ok(())
}

fn create_program(gl: &Gl) -> Result<WebGlProgram, JsValue> {
    let vertex = compile_shader(gl, Gl::VERTEX_SHADER, VERTEX_SHADER)?;
    let fragment = match compile_shader(gl, Gl::FRAGMENT_SHADER, FRAGMENT_SHADER) {
        Ok(fragment) => fragment,
        Err(error) => {
            gl.delete_shader(Some(&vertex));
            return Err(error);
        }
    };
    let Some(program) = gl.create_program() else {
        gl.delete_shader(Some(&vertex));
        gl.delete_shader(Some(&fragment));
        return Err(JsValue::from_str("failed to create WebGL program"));
    };
    gl.attach_shader(&program, &vertex);
    gl.attach_shader(&program, &fragment);
    gl.link_program(&program);
    gl.delete_shader(Some(&vertex));
    gl.delete_shader(Some(&fragment));

    if gl
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        let message = gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "failed to link WebGL program".into());
        gl.delete_program(Some(&program));
        Err(JsValue::from_str(&message))
    }
}

fn compile_shader(gl: &Gl, shader_type: u32, source: &str) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("failed to create WebGL shader"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        let message = gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "failed to compile WebGL shader".into());
        gl.delete_shader(Some(&shader));
        Err(JsValue::from_str(&message))
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
