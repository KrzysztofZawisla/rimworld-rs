//! Theme: command. Split from `layout.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use proc_macro2::Span;
use syn::{spanned::Spanned, File as RustFile, ImplItem, Item};

use super::*;
use crate::{missing, repo_root, Result};

/// Runs the command: it measures every file and reports the ones over the rule. A path limits it to one file,
/// and `--fix` only explains why there is nothing to fix: the rule has no automatic fixer, a file over the
/// limit is split by hand along thematic lines.
pub fn run(fix: bool, file: Option<PathBuf>) -> Result<()> {
    let root = repo_root();
    if fix {
        println!(
            "note          : the rule has no automatic fix: a file over the limit is split by hand, along thematic lines (see `File layout` in AGENTS.md)"
        );
    }

    let only: Option<PathBuf> = file.map(|file| root.join(file));

    let mut files = Vec::new();
    for directory in ["crates", "examples", "mods"] {
        collect(&root.join(directory), ".rs", &mut files);
    }

    let mut measured = Vec::new();
    for file in files {
        if exempt(&file) || only.as_ref().is_some_and(|wanted| !file.ends_with(wanted)) {
            continue;
        }

        measured.push(measure(&file)?);
    }

    let counted: usize = measured.iter().filter_map(|file| file.items).sum();
    println!(
        "{} Rust file(s) within {MAX_LINES} lines and {MAX_ITEMS} items, {counted} item(s) counted",
        measured.iter().filter(|file| file.verdict().is_none()).count()
    );

    let over: Vec<&Measured> = measured.iter().filter(|file| file.verdict().is_some()).collect();
    for file in &over {
        println!("  over  {:>22}  {}", file.verdict().unwrap_or_default(), relative(&root, &file.file));
    }

    let (csharp_over, csharp_within) = measure_csharp(&root, only.as_deref());
    for (file, lines) in &csharp_over {
        println!("  over  {:>22}  {}", format!("{lines} lines"), relative(&root, file));
    }

    if over.is_empty() && csharp_over.is_empty() {
        println!("every file is inside the rule ({csharp_within} C# file(s) measured by lines)");
        return Ok(());
    }

    let total = over.len() + csharp_over.len();
    println!("{total} file(s) need a thematic split: {} Rust, {} C#", over.len(), csharp_over.len());
    Err(missing(format!("{total} file(s) outgrew the layout rule (see `File layout` in AGENTS.md)")))
}
