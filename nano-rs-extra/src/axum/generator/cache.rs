use crate::axum::generator::write_if_changed;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const CACHE_DIR_NAME: &str = ".nano_cache";
const API_INFO_HASH_FILE: &str = "api_info.hash";
const ROUTE_HASH_FILE: &str = "route.hash";
const DOC_SCHEMA_FILE: &str = "doc_schema.json";

fn cache_dir_path(base: &Path) -> PathBuf {
    base.join(CACHE_DIR_NAME)
}

pub fn read_api_info_hash(base: &Path) -> io::Result<String> {
    fs::read_to_string(cache_dir_path(base).join(API_INFO_HASH_FILE))
}

pub fn write_api_info_hash(base: &Path, hash: &str) -> io::Result<()> {
    let dir = cache_dir_path(base);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(API_INFO_HASH_FILE), hash)
}

pub fn read_route_hash(base: &Path) -> io::Result<String> {
    fs::read_to_string(cache_dir_path(base).join(ROUTE_HASH_FILE))
}

pub fn write_route_hash(base: &Path, hash: &str) -> io::Result<()> {
    let dir = cache_dir_path(base);
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(ROUTE_HASH_FILE), hash)
}

#[derive(Debug, Clone)]
pub struct FileFingerprint {
    pub modified: u128,
    pub len: u64,
}

impl FileFingerprint {
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis())
            .unwrap_or_default();
        Ok(Self {
            modified,
            len: metadata.len(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocSchemaCache {
    files: BTreeMap<String, FileCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileCacheEntry {
    pub(crate) modified: u128,
    pub(crate) len: u64,
    pub(crate) structs: Vec<String>,
    pub(crate) enums: Vec<String>,
}

impl DocSchemaCache {
    pub fn load(base: &Path) -> Self {
        let path = cache_dir_path(base).join(DOC_SCHEMA_FILE);
        match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, base: &Path) -> io::Result<()> {
        let dir = cache_dir_path(base);
        let path = dir.join(DOC_SCHEMA_FILE);
        let json = serde_json::to_string_pretty(self)?;
        let _ = write_if_changed(path.as_path(), json.as_str())?;
        Ok(())
    }

    fn key(base: &Path, file: &Path) -> String {
        file.strip_prefix(base)
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|_| file.to_string_lossy().to_string())
    }

    pub fn get(
        &self,
        base: &Path,
        file: &Path,
        fingerprint: &FileFingerprint,
    ) -> Option<&FileCacheEntry> {
        let key = Self::key(base, file);
        self.files.get(&key).and_then(|entry| {
            if entry.modified == fingerprint.modified && entry.len == fingerprint.len {
                Some(entry)
            } else {
                None
            }
        })
    }

    pub fn update(
        &mut self,
        base: &Path,
        file: &Path,
        fingerprint: &FileFingerprint,
        structs: Vec<String>,
        enums: Vec<String>,
    ) {
        let key = Self::key(base, file);
        self.files.insert(
            key,
            FileCacheEntry {
                modified: fingerprint.modified,
                len: fingerprint.len,
                structs,
                enums,
            },
        );
    }
}
