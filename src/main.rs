mod scanner;
mod tree;

use std::{path::Path, time::Instant};

use clap::Parser;

use crate::scanner::scan_directory;

#[derive(Parser)]
#[command(name = "howbig")]
#[command(author, version, about = "Analyze directory sizes")]
struct Cli {
    /// Directory to analyze
    #[arg(default_value = ".")]
    path: String,

    /// How many levels deep to show in tree output (all dirs still scanned)
    #[arg(short, long, default_value_t = 3)]
    depth: u32,

    /// Show top N largest items per directory
    #[arg(short = 'n', long, default_value_t = 5)]
    top: usize,

    /// Show only summary statistics (no tree)
    #[arg(short, long)]
    summary: bool,

    /// Show all items per directory (no limit)
    #[arg(short, long)]
    all: bool,

    /// Show full tree (no depth limit)
    #[arg(short, long)]
    full: bool,

    /// Hide items smaller than this size (e.g., 1MB, 500KB)
    #[arg(long, value_parser = parse_size)]
    min_size: Option<u64>,
}

fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim().to_uppercase();

    let (num_str, multiplier) = if let Some(n) = s.strip_suffix("TB") {
        (n, 1024u64 * 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("GB") {
        (n, 1024u64 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix("MB") {
        (n, 1024u64 * 1024)
    } else if let Some(n) = s.strip_suffix("KB") {
        (n, 1024u64)
    } else if let Some(n) = s.strip_suffix("B") {
        (n, 1u64)
    } else {
        // Assume bytes if no suffix
        (s.as_str(), 1u64)
    };

    let num: f64 = num_str
        .trim()
        .parse()
        .map_err(|_| format!("Invalid size: {}", s))?;

    Ok((num * multiplier as f64) as u64)
}

fn main() {
    let cli = Cli::parse();
    let path = Path::new(&cli.path);

    println!("Scanning: {}", display_path(path));

    let mut stats = scanner::ScanStats::default();
    let start = Instant::now();
    match scan_directory(path, &mut stats) {
        Ok(mut root) => {
            root.sort_children();
            let scan_time = start.elapsed();

            if !cli.summary {
                let max_depth = if cli.full { None } else { Some(cli.depth) };
                let top_n = if cli.all { None } else { Some(cli.top) };
                print_tree(&root, root.size, 0, max_depth, top_n, cli.min_size);
                println!();
            }

            println!("Scan completed in : {:?}", scan_time);
            println!("Files             : {}", stats.files_scanned);
            println!("Directories       : {}", stats.dirs_scanned);
            println!("Errors            : {}", stats.errors);
            println!("Total size        : {}", format_size(root.size));
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

fn print_tree(
    entry: &tree::DirEntry,
    total_size: u64,
    depth: u32,
    max_depth: Option<u32>,
    top_n: Option<usize>,
    min_size: Option<u64>,
) {
    if let Some(max) = max_depth
        && depth > max
    {
        return;
    }

    if let Some(min) = min_size
        && entry.size < min
    {
        return;
    }

    let indent = "  ".repeat(depth as usize);
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

    let children: Box<dyn Iterator<Item = &tree::DirEntry>> = match top_n {
        Some(n) => Box::new(entry.children.iter().take(n)),
        None => Box::new(entry.children.iter()),
    };

    for child in children {
        print_tree(child, total_size, depth + 1, max_depth, top_n, min_size);
    }
}
