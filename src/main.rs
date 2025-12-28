mod scanner;
mod tree;

use std::{env, path::Path};

use crate::scanner::scan_directory;

fn main() {
    let app_name = env!("CARGO_PKG_NAME");
    let app_version = env!("CARGO_PKG_VERSION");
    let app_author = env!("CARGO_PKG_AUTHORS");
    println!("{} v{}", app_name, app_version);
    println!("(c) {}", app_author);

    let args: Vec<String> = env::args().collect();
    let path = args.get(1).map_or(".", |s| s.as_str());
    let path = Path::new(path);
    println!("Scanning: {}", display_path(path));

    let mut stats = scanner::ScanStats::default();
    match scan_directory(path, &mut stats) {
        Ok(mut root) => {
            root.sort_children();

            println!("Scan complete");
            println!("Files: {}", stats.files_scanned);
            println!("Directories: {}", stats.dirs_scanned);
            println!("Errors: {}", stats.errors);
            println!("Total size: {}", format_size(root.size));
            println!();

            print_tree(&root, root.size, 0, 3);
        }
        Err(e) => {
            eprintln!("Failed to scan directory: {}", e);
        }
    }
}

fn display_path(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or(path.to_path_buf());
    let path_str = canonical.to_string_lossy();
    path_str
        .strip_prefix(r"\\?\")
        .unwrap_or(&path_str)
        .to_string()
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

fn print_tree(entry: &tree::DirEntry, total_size: u64, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }

    let indent = "  ".repeat(depth);
    let size_str = format_size(entry.size);
    let percentage = entry.percentage_of(total_size);

    println!(
        "{:>6.2}% {:>10} {} {}{}",
        percentage,
        size_str,
        indent,
        entry.name,
        if entry.is_dir { "/" } else { "" }
    );

    for child in entry.children.iter().take(5) {
        print_tree(child, total_size, depth + 1, max_depth);
    }
}
