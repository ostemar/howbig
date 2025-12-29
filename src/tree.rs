use std::path::Path;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub size: u64,
    pub file_count: u64,
    pub children: Vec<DirEntry>,
    pub is_dir: bool,
    /// Number of children pruned to save memory
    pub other_count: u64,
    /// Total size of pruned children
    pub other_size: u64,
}

impl DirEntry {
    pub fn new(path: &Path, is_dir: bool) -> Self {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Self {
            name,
            size: 0,
            file_count: 0,
            children: Vec::new(),
            is_dir,
            other_count: 0,
            other_size: 0,
        }
    }

    pub fn percentage_of(&self, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (self.size as f64 / total as f64) * 100.0
        }
    }
}
