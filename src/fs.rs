use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub struct DirEntries {
    pub entries: Vec<DirEntry>,
    pub error: Option<String>,
}

pub fn load_dir_entries(path: &Path) -> DirEntries {
    let mut entries: Vec<DirEntry> = Vec::new();
    let mut error: Option<String> = None;
    
    match fs::read_dir(path) {
        Ok(read_dir) => {
            for entry in read_dir {
                match entry {
                    Ok(dir_entry) => {
                        let entry_path = dir_entry.path();
                        let name = dir_entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry_path.is_dir();
                        entries.push(DirEntry { name, path: entry_path, is_dir });
                    }
                    Err(_e) => {
                        continue;
                    }
                }
            }
        }
        Err(e) => {
            error = Some(e.to_string());
        }
    }
    
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    DirEntries { entries, error }
}
