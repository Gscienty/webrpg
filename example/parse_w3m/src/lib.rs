use std::io::Cursor;

use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

const TERRAIN_PATH: &str = "war3map.w3e";

#[wasm_bindgen]
pub fn list_mpq_files(name: &str, data: &[u8]) -> Result<String, JsValue> {
    let mut archive = webrpg_mpq::Archive::load(name, data.to_vec()).map_err(js_error)?;
    match archive.list_files() {
        Ok(files) => Ok(files.join("\n")),
        Err(_error)
            if name.to_ascii_lowercase().ends_with(".w3m")
                || name.to_ascii_lowercase().ends_with(".w3x") =>
        {
            archive.read_file(TERRAIN_PATH).map_err(js_error)?;
            Ok(TERRAIN_PATH.into())
        }
        Err(error) => Err(js_error(error)),
    }
}

#[wasm_bindgen]
pub struct Terrain {
    width: u32,
    height: u32,
    tileset: u8,
    cells: Vec<u8>,
}

#[wasm_bindgen]
impl Terrain {
    #[wasm_bindgen(constructor)]
    pub fn new(mpq_name: &str, mpq_data: &[u8], path: &str) -> Result<Terrain, JsValue> {
        let mut archive =
            webrpg_mpq::Archive::load(mpq_name, mpq_data.to_vec()).map_err(js_error)?;
        let data = archive.read_file(path).map_err(js_error)?;
        let lower_path = path.to_ascii_lowercase();
        let terrain_data = if lower_path.ends_with(".w3m") || lower_path.ends_with(".w3x") {
            let mut map_archive = webrpg_mpq::Archive::load(path, data).map_err(js_error)?;
            map_archive.read_file(TERRAIN_PATH).map_err(js_error)?
        } else {
            data
        };
        let map = fileformat::w3m::Map::read(&mut Cursor::new(terrain_data)).map_err(js_error)?;
        if map.width < 2 || map.height < 2 {
            return Err(JsValue::from_str(
                "terrain must contain at least 2 x 2 vertices",
            ));
        }

        let cell_width = map.width - 1;
        let cell_height = map.height - 1;
        let mut cells = Vec::with_capacity((cell_width * cell_height) as usize * 3);
        for y in 0..cell_height {
            for x in 0..cell_width {
                let vertex = &map.vertices[(y * map.width + x) as usize];
                cells.extend([vertex.ground, vertex.water, vertex.blight]);
            }
        }

        Ok(Self {
            width: cell_width,
            height: cell_height,
            tileset: map.tileset,
            cells,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[wasm_bindgen(getter)]
    pub fn tileset(&self) -> String {
        char::from(self.tileset).to_string()
    }

    pub fn cells(&self) -> Vec<u8> {
        self.cells.clone()
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
