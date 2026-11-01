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

pub trait Archive {
    fn id(&self) -> &str;

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String>;

    fn list_files(&mut self) -> Result<Vec<String>, String> {
        let list =
            String::from_utf8(self.read_file("(listfile)")?).map_err(|error| error.to_string())?;
        Ok(list
            .lines()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(str::to_owned)
            .collect())
    }
}

pub struct MpqArchive {
    id: String,
    inner: mpq::Archive,
}

impl MpqArchive {
    pub fn load(id: impl Into<String>, data: Vec<u8>) -> Result<Self, String> {
        Ok(Self {
            id: id.into(),
            inner: mpq::Archive::load(data).map_err(|error| error.to_string())?,
        })
    }
}

impl Archive for MpqArchive {
    fn id(&self) -> &str {
        &self.id
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
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

pub struct MultiArchive {
    id: String,
    archives: Vec<Box<dyn Archive>>,
}

impl MultiArchive {
    pub fn new(archives: Vec<Box<dyn Archive>>) -> Self {
        let id = archives
            .iter()
            .map(|archive| archive.id())
            .collect::<Vec<_>>()
            .join("|");
        Self { id, archives }
    }
}

impl Archive for MultiArchive {
    fn id(&self) -> &str {
        &self.id
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
        let mut errors = Vec::with_capacity(self.archives.len());
        for archive in &mut self.archives {
            match archive.read_file(path) {
                Ok(data) => return Ok(data),
                Err(error) => errors.push(format!("{}: {error}", archive.id())),
            }
        }
        Err(format!(
            "failed to read {path} from {}: {}",
            self.id,
            errors.join("; ")
        ))
    }
}

pub struct NestedArchive {
    archives: MultiArchive,
}

impl NestedArchive {
    pub fn load(mut parent: Box<dyn Archive>, child_path: &str) -> Result<Self, String> {
        let child_data = parent.read_file(child_path)?;
        let child = MpqArchive::load(child_path, child_data)?;
        Ok(Self::new(parent, Box::new(child)))
    }

    pub fn new(parent: Box<dyn Archive>, child: Box<dyn Archive>) -> Self {
        Self {
            archives: MultiArchive::new(vec![child, parent]),
        }
    }
}

impl Archive for NestedArchive {
    fn id(&self) -> &str {
        self.archives.id()
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.archives.read_file(path)
    }
}

/// An MPQ loaded from OPFS. Opening is asynchronous; reads are synchronous.
#[wasm_bindgen]
pub struct OpfsMpq {
    archive: MpqArchive,
}

#[wasm_bindgen]
impl OpfsMpq {
    /// Load and parse a named MPQ from OPFS once.
    pub async fn open(name: &str) -> Result<OpfsMpq, JsValue> {
        let data = load_mpq(name).await?;
        let archive = MpqArchive::load(name, data).map_err(js_error)?;
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
    let mut archive = MpqArchive::load("", mpq_data.to_vec()).map_err(js_error)?;
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
    use std::collections::BTreeMap;

    use super::*;

    struct MemoryArchive {
        id: String,
        files: BTreeMap<String, Vec<u8>>,
    }

    impl MemoryArchive {
        fn new(id: &str, files: &[(&str, &[u8])]) -> Self {
            Self {
                id: id.into(),
                files: files
                    .iter()
                    .map(|(path, data)| ((*path).into(), data.to_vec()))
                    .collect(),
            }
        }
    }

    impl Archive for MemoryArchive {
        fn id(&self) -> &str {
            &self.id
        }

        fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
            self.files
                .get(path)
                .cloned()
                .ok_or_else(|| format!("{path} not found"))
        }
    }

    #[test]
    fn normalizes_mpq_paths() {
        assert_eq!(
            normalize_mpq_path("war3mapImported/model.mdx"),
            "war3mapImported\\model.mdx"
        );
        assert_eq!(normalize_mpq_path("war3map.j"), "war3map.j");
    }

    #[test]
    fn multi_archive_uses_first_match_and_falls_back() {
        let first = MemoryArchive::new("first", &[("shared", b"first")]);
        let second = MemoryArchive::new(
            "second",
            &[("shared", b"second"), ("fallback", b"fallback")],
        );
        let mut archives = MultiArchive::new(vec![Box::new(first), Box::new(second)]);

        assert_eq!(archives.read_file("shared").unwrap(), b"first");
        assert_eq!(archives.read_file("fallback").unwrap(), b"fallback");
        assert!(archives.read_file("missing").is_err());
    }

    #[test]
    fn nested_archive_prefers_child_then_parent() {
        let parent = MemoryArchive::new(
            "parent",
            &[("shared", b"parent"), ("parent-only", b"parent")],
        );
        let child = MemoryArchive::new("child", &[("shared", b"child")]);
        let mut nested = NestedArchive::new(Box::new(parent), Box::new(child));

        assert_eq!(nested.read_file("shared").unwrap(), b"child");
        assert_eq!(nested.read_file("parent-only").unwrap(), b"parent");
    }

    #[test]
    fn archive_lists_trimmed_non_empty_paths() {
        let mut archive = MemoryArchive::new("map", &[("(listfile)", b"a\\b\n\n c ")]);

        assert_eq!(archive.list_files().unwrap(), ["a\\b", "c"]);
    }
}
