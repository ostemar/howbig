use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DirEntry {
    #[allow(dead_code, reason = "will be used for file navigation later")]
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub file_count: u64,
    pub children: Vec<DirEntry>,
    pub is_dir: bool,
}

impl DirEntry {
    pub fn new(path: PathBuf, is_dir: bool) -> Self {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Self {
            path,
            name,
            size: 0,
            file_count: 0,
            children: Vec::new(),
            is_dir,
        }
    }

    pub fn percentage_of(&self, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (self.size as f64 / total as f64) * 100.0
        }
    }

    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| b.size.cmp(&a.size));
        for child in &mut self.children {
            child.sort_children();
        }
    }
}
