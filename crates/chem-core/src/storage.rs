use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct UserStore(PathBuf);
impl UserStore {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
    pub fn load(&self) -> Result<Value, String> {
        if !self.0.exists() {
            return Ok(serde_json::json!({}));
        }
        let bytes = std::fs::read(&self.0).map_err(|e| e.to_string())?;
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("Файл настроек повреждён: {e}"))?;
        if !value.is_object() {
            return Err("Файл настроек должен содержать объект".into());
        }
        Ok(value)
    }
    pub fn save(&self, value: &Value) -> Result<(), String> {
        if !value.is_object() {
            return Err("Некорректные настройки".into());
        }
        // Do not replace an unreadable original without explicitly backing it up.
        self.load()?;
        atomic_write(
            &self.0,
            &serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
        )
    }
    pub fn recover(&self) -> Result<(), String> {
        if self.0.exists() {
            let mut backup = self.0.with_extension("json.corrupt");
            let mut index = 1;
            while backup.exists() {
                backup = self.0.with_extension(format!("json.corrupt.{index}"));
                index += 1;
            }
            std::fs::rename(&self.0, backup).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Некорректный путь")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary.write_all(bytes).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    std::fs::File::open(parent)
        .and_then(|file| file.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(())
}
