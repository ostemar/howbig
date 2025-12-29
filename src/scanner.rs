use std::{
    fs, io,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use rayon::prelude::*;

use crate::tree::DirEntry;

#[derive(Default)]
pub struct ScanStats {
    pub files_scanned: AtomicU64,
    pub dirs_scanned: AtomicU64,
    pub errors: AtomicU64,
}

impl ScanStats {
    pub fn files_scanned(&self) -> u64 {
        self.files_scanned.load(Ordering::Relaxed)
    }

    pub fn dirs_scanned(&self) -> u64 {
        self.dirs_scanned.load(Ordering::Relaxed)
    }

    pub fn errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }
}

pub fn scan_directory(path: &Path, stats: &ScanStats, max_children: usize) -> io::Result<DirEntry> {
    let metadata = fs::metadata(path)?;
    let mut entry = DirEntry::new(path, metadata.is_dir());

    if !metadata.is_dir() {
        entry.size = metadata.len();
        entry.file_count = 1;
        stats.files_scanned.fetch_add(1, Ordering::Relaxed);
        return Ok(entry);
    }

    // It's a directory, let's scan its contents recursively
    stats.dirs_scanned.fetch_add(1, Ordering::Relaxed);

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            stats.errors.fetch_add(1, Ordering::Relaxed);
            eprintln!("Error reading directory {:?}: {}", path, e);
            return Ok(entry);
        }
    };

    let dir_entries: Vec<_> = read_dir.collect();
    let mut children: Vec<DirEntry> = dir_entries
        .into_par_iter()
        .filter_map(|dir_entry| {
            let dir_entry = match dir_entry {
                Ok(de) => de,
                Err(e) => {
                    stats.errors.fetch_add(1, Ordering::Relaxed);
                    eprintln!("Error accessing directory entry in {:?}: {}", path, e);
                    return None;
                }
            };

            let child_path = dir_entry.path();
            match scan_directory(&child_path, stats, max_children) {
                Ok(child) => Some(child),
                Err(e) => {
                    stats.errors.fetch_add(1, Ordering::Relaxed);
                    eprintln!("Error scanning {:?}: {}", child_path, e);
                    None
                }
            }
        })
        .collect();

    // Sort children by size in descending order
    children.sort_unstable_by(|a, b| b.size.cmp(&a.size));

    // Calculate total size/count from all children before pruning
    for child in &children {
        entry.size += child.size;
        entry.file_count += child.file_count;
    }

    // Prune children beyond max_children limit to save memory
    if children.len() > max_children {
        let pruned = children.split_off(max_children);
        entry.other_count = pruned.len() as u64;
        entry.other_size = pruned.iter().map(|c| c.size).sum();
    }

    entry.children = children;

    Ok(entry)
}
