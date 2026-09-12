use renderer::texture::Texture;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext as Gl, WebGlProgram, WebGlShader};

#[wasm_bindgen]
pub fn render_blp(canvas: HtmlCanvasElement, data: &[u8]) -> Result<(), JsValue> {
    web_sys::console::log_1(
        &format!(
            "BLP: magic={:?}, bytes={}",
            data.get(..4).unwrap_or(data),
            data.len()
        )
        .into(),
    );
    let gl = canvas
        .get_context("webgl2")?
        .ok_or_else(|| JsValue::from_str("WebGL2 is unavailable"))?
        .dyn_into::<Gl>()?;
    let texture =
        Texture::load(&gl, data).map_err(|error| JsValue::from_str(&error.to_string()))?;
    check_gl(&gl, "texture upload")?;

    let vertex = compile_shader(
        &gl,
        Gl::VERTEX_SHADER,
        r#"#version 300 es
        in vec2 position;
        in vec2 uv;
        out vec2 texture_uv;
        void main() {
            texture_uv = uv;
            gl_Position = vec4(position, 0.0, 1.0);
        }"#,
    )?;
    let fragment = compile_shader(
        &gl,
        Gl::FRAGMENT_SHADER,
        r#"#version 300 es
        precision mediump float;
        uniform sampler2D image;
        in vec2 texture_uv;
        out vec4 color;
        void main() {
            color = texture(image, texture_uv);
        }"#,
    )?;
    let program = link_program(&gl, &vertex, &fragment)?;
    let buffer = gl
        .create_buffer()
        .ok_or_else(|| JsValue::from_str("failed to create vertex buffer"))?;
    let vertices: [f32; 24] = [
        -1.0, -1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, -1.0, 1.0, 0.0, 0.0, 1.0,
        -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
    ];

    gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&buffer));
    unsafe {
        gl.buffer_data_with_array_buffer_view(
            Gl::ARRAY_BUFFER,
            &js_sys::Float32Array::view(&vertices),
            Gl::STATIC_DRAW,
        );
    }
    gl.use_program(Some(&program));

    let position = gl.get_attrib_location(&program, "position") as u32;
    gl.enable_vertex_attrib_array(position);
    gl.vertex_attrib_pointer_with_i32(position, 2, Gl::FLOAT, false, 16, 0);
    let uv = gl.get_attrib_location(&program, "uv") as u32;
    gl.enable_vertex_attrib_array(uv);
    gl.vertex_attrib_pointer_with_i32(uv, 2, Gl::FLOAT, false, 16, 8);
    check_gl(&gl, "vertex setup")?;

    gl.active_texture(Gl::TEXTURE0);
    texture.bind();
    if let Some(image) = gl.get_uniform_location(&program, "image") {
        gl.uniform1i(Some(&image), 0);
    }
    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
    gl.clear_color(0.08, 0.08, 0.1, 1.0);
    gl.clear(Gl::COLOR_BUFFER_BIT);
    gl.draw_arrays(Gl::TRIANGLES, 0, 6);
    check_gl(&gl, "draw")?;

    gl.delete_buffer(Some(&buffer));
    gl.delete_program(Some(&program));
    gl.delete_shader(Some(&vertex));
    gl.delete_shader(Some(&fragment));
    Ok(())
}

fn check_gl(gl: &Gl, stage: &str) -> Result<(), JsValue> {
    let error = gl.get_error();
    if error == Gl::NO_ERROR {
        web_sys::console::log_1(&format!("WebGL {stage}: OK").into());
        Ok(())
    } else {
        Err(JsValue::from_str(&format!(
            "WebGL {stage} failed: 0x{error:04x}"
        )))
    }
}

fn compile_shader(gl: &Gl, kind: u32, source: &str) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(kind)
        .ok_or_else(|| JsValue::from_str("failed to create shader"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if gl
        .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "shader compilation failed".into()),
        ))
    }
}

fn link_program(
    gl: &Gl,
    vertex: &WebGlShader,
    fragment: &WebGlShader,
) -> Result<WebGlProgram, JsValue> {
    let program = gl
        .create_program()
        .ok_or_else(|| JsValue::from_str("failed to create shader program"))?;
    gl.attach_shader(&program, vertex);
    gl.attach_shader(&program, fragment);
    gl.link_program(&program);
    if gl
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(JsValue::from_str(
            &gl.get_program_info_log(&program)
                .unwrap_or_else(|| "shader link failed".into()),
        ))
    }
}
