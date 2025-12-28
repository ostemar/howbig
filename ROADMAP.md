# Roadmap for howbig

## Simple TUI

Navigate the size tree with arrows. Probably using `ratatui`.

Interactive features:

- delete files or directories
- filtering
- searching

## Parallel scanning

Improve performance of scanning by doing it in parallel, perhaps using `rayon`
or similar.

## Caching results

For faster subsequent scans, cache previous results and only rescan changed files.

## Common culprits

Find common culprits of large sizes like **node_modules** or similar.

Maybe find duplicate entries using file hashes.
