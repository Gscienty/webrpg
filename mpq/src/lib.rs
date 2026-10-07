use js_sys::{Promise, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(inline_js = r#"
export async function readOpfsFile(name) {
    const root = await navigator.storage.getDirectory();
    const handle = await root.getFileHandle(name);
    const file = await handle.getFile();
    return new Uint8Array(await file.arrayBuffer());
}

export async function writeOpfsFile(name, data) {
    const root = await navigator.storage.getDirectory();
    const handle = await root.getFileHandle(name, { create: true });
    const writable = await handle.createWritable();
    try {
        await writable.write(data);
    } finally {
        await writable.close();
    }
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = readOpfsFile)]
    fn read_opfs_file(name: &str) -> Promise;

    #[wasm_bindgen(js_name = writeOpfsFile)]
    fn write_opfs_file(name: &str, data: &Uint8Array) -> Promise;
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

fn normalize_mpq_path(path: &str) -> String {
    path.replace('/', "\\")
}

pub struct Archive {
    inner: mpq::Archive,
}

impl Archive {
    pub fn load(data: Vec<u8>) -> Result<Self, String> {
        Ok(Self {
            inner: mpq::Archive::load(data).map_err(|error| error.to_string())?,
        })
    }

    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
        let file = self
            .inner
            .open_file(&normalize_mpq_path(path))
            .map_err(|error| error.to_string())?;
        let mut data = vec![0; file.size() as usize];
        let read = file
            .read(&mut self.inner, &mut data)
            .map_err(|error| error.to_string())?;
        data.truncate(read);
        Ok(data)
    }
}

/// An MPQ loaded from OPFS. Opening is asynchronous; reads are synchronous.
#[wasm_bindgen]
pub struct OpfsMpq {
    archive: Archive,
}

#[wasm_bindgen]
impl OpfsMpq {
    /// Load and parse a named MPQ from OPFS once.
    pub async fn open(name: &str) -> Result<OpfsMpq, JsValue> {
        let data = load_mpq(name).await?;
        let archive = Archive::load(data).map_err(js_error)?;
        Ok(Self { archive })
    }

    /// Synchronously read one file from the already loaded MPQ.
    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>, JsValue> {
        self.archive.read_file(path).map_err(js_error)
    }
}

/// Load a named MPQ file from the origin private file system.
#[wasm_bindgen]
pub async fn load_mpq(name: &str) -> Result<Vec<u8>, JsValue> {
    let value = JsFuture::from(read_opfs_file(name)).await?;
    Ok(Uint8Array::new(&value).to_vec())
}

/// Store or replace a named MPQ file in the origin private file system.
#[wasm_bindgen]
pub async fn store_mpq(name: &str, data: &[u8]) -> Result<(), JsValue> {
    let data = Uint8Array::from(data);
    JsFuture::from(write_opfs_file(name, &data)).await?;
    Ok(())
}

/// Read one file from MPQ bytes. Both slash styles are accepted for `path`.
#[wasm_bindgen]
pub fn read_mpq_file(mpq_data: &[u8], path: &str) -> Result<Vec<u8>, JsValue> {
    let mut archive = Archive::load(mpq_data.to_vec()).map_err(js_error)?;
    archive.read_file(path).map_err(js_error)
}

/// Load an MPQ from OPFS and read one file from it.
#[wasm_bindgen]
pub async fn read_opfs_mpq_file(name: &str, path: &str) -> Result<Vec<u8>, JsValue> {
    let data = load_mpq(name).await?;
    read_mpq_file(&data, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_mpq_paths() {
        assert_eq!(
            normalize_mpq_path("war3mapImported/model.mdx"),
            "war3mapImported\\model.mdx"
        );
        assert_eq!(normalize_mpq_path("war3map.j"), "war3map.j");
    }
}
