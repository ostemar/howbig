use std::{fs, io, path::Path};

use crate::tree::DirEntry;

#[derive(Default)]
pub struct ScanStats {
    pub files_scanned: u64,
    pub dirs_scanned: u64,
    pub errors: u64,
}

pub fn scan_directory(path: &Path, stats: &mut ScanStats) -> io::Result<DirEntry> {
    let metadata = fs::metadata(path)?;
    let mut entry = DirEntry::new(path.to_path_buf(), metadata.is_dir());

    if !metadata.is_dir() {
        entry.size = metadata.len();
        entry.file_count = 1;
        stats.files_scanned += 1;
        return Ok(entry);
    }

    // It's a directory, let's scan its contents recursively
    stats.dirs_scanned += 1;

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            stats.errors += 1;
            eprintln!("Error reading directory {:?}: {}", path, e);
            return Ok(entry);
        }
    };

    for dir_entry in read_dir {
        let dir_entry = match dir_entry {
            Ok(de) => de,
            Err(e) => {
                stats.errors += 1;
                eprintln!("Error accessing directory entry in {:?}: {}", path, e);
                continue;
            }
        };

        let child_path = dir_entry.path();
        match scan_directory(&child_path, stats) {
            Ok(child) => {
                entry.size += child.size;
                entry.file_count += child.file_count;
                entry.children.push(child);
            }
            Err(e) => {
                stats.errors += 1;
                eprintln!("Error scanning {:?}: {}", child_path, e);
            }
        }
    }

    Ok(entry)
}
