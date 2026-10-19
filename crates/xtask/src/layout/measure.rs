//! Theme: measure. Split from `layout.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use proc_macro2::Span;
use syn::{spanned::Spanned, File as RustFile, ImplItem, Item};
use walkdir::WalkDir;

use super::*;
use crate::{missing, repo_root, Result};

/// The number of lines a hand-written file may reach before the rule asks for a thematic split.
pub(crate) const MAX_LINES: usize = 500;

/// The number of lines a crate root (`lib.rs`, `main.rs`, `mod.rs`) may reach: a root declares modules
/// and re-exports, it does not hold the code.
pub(crate) const MAX_ROOT_LINES: usize = 300;

/// The number of items a hand-written file may hold before the rule asks for a thematic split.
pub(crate) const MAX_ITEMS: usize = 40;

/// C# files are measured by lines only (counting their members needs Roslyn); the bridge keeps the
/// limit it had before the Rust rule was tightened.
pub(crate) const MAX_CSHARP_LINES: usize = 800;

/// What one measured file came to.
pub(crate) struct Measured {
    pub(crate) file: PathBuf,
    pub(crate) lines: usize,
    /// `None` when the file does not parse: then only its lines are measured.
    pub(crate) items: Option<usize>,
}

impl Measured {
    /// Whether the file is over the rule, and by which of the two numbers.
    pub(crate) fn verdict(&self) -> Option<String> {
        let root = is_root(&self.file);
        let generated = is_generated(&self.file);
        // A generated `mod.rs` is a generated file first: its size is the generator's business, and 500
        // is the limit it has to keep. The lower root limit is for hand-written roots.
        let line_limit = if generated {
            MAX_LINES
        } else if root {
            MAX_ROOT_LINES
        } else {
            MAX_LINES
        };
        let too_many_lines = self.lines > line_limit;
        let check_items = !root && !generated;
        let too_many_items = check_items && self.items.is_some_and(|items| items > MAX_ITEMS);
        if !too_many_lines && !too_many_items {
            return None;
        }

        if !check_items {
            return Some(format!("{:>5} lines", self.lines));
        }

        let items = match self.items {
            Some(items) => format!("{items} items"),
            None => "unparsable".to_string(),
        };
        Some(format!("{:>5} lines, {items}", self.lines))
    }
}

/// Line count and item count of one file.
pub(crate) fn measure(file: &Path) -> Result<Measured> {
    let text = fs::read_to_string(file).map_err(|error| missing(format!("reading {}: {error}", file.display())))?;
    let lines = text.lines().count();
    // A root is measured by lines alone, and a generated file's constants are not "items" the rule is
    // meant to count: parsing them would only make the gate report a file the generator owns.
    let items = if is_root(file) || is_generated(file) {
        None
    } else {
        match syn::parse_file(&text) {
            Ok(parsed) => Some(items_of(&parsed.items)),
            Err(_) => None,
        }
    };

    Ok(Measured { file: file.to_path_buf(), lines, items })
}

/// Every item of a file, inline modules included: a module that hides three hundred items behind one `mod`
/// line is exactly the file the rule is meant to catch.
///
/// An `impl` block is not counted: it defines no name, and holding a type's whole body is what an `impl` is
/// for - the line limit is what keeps one reasonable. Counting its methods reported a normal block as forty
/// items and split a 421-line file.
pub(crate) fn items_of(items: &[Item]) -> usize {
    let mut total = 0usize;
    for item in items {
        match item {
            Item::Impl(_) => {}
            Item::Mod(module) => {
                if let Some((_, content)) = &module.content {
                    total += items_of(content);
                }
            }
            _ => total += 1,
        }
    }

    total
}

/// Lines only: an item count for C# needs Roslyn. Returns the files over the line limit, and how many are
/// inside it.
pub(crate) fn measure_csharp(root: &Path, only: Option<&Path>) -> (Vec<(PathBuf, usize)>, usize) {
    let mut files = Vec::new();
    for directory in ["bridge", "examples", "tools"] {
        collect(&root.join(directory), ".cs", &mut files);
    }

    let mut over = Vec::new();
    let mut within = 0usize;
    for file in files {
        if only.is_some_and(|wanted| !file.ends_with(wanted)) {
            continue;
        }

        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let lines = text.lines().count();
        if lines > MAX_CSHARP_LINES {
            over.push((file, lines));
        } else {
            within += 1;
        }
    }

    (over, within)
}

/// The roots cargo looks for: a crate root has to keep its name, and a module root is nothing but
/// `mod` declarations. They are measured by lines, not by items.
pub(crate) fn is_root(file: &Path) -> bool {
    let name = file.file_name().unwrap_or_default().to_string_lossy();
    matches!(name.as_ref(), "lib.rs" | "main.rs" | "mod.rs")
}

/// Files a tool writes: their contents (and their item count) are the generator's business, but their
/// size is still measured, because a generator that emits one huge file is the defect this rule catches.
pub(crate) fn is_generated(file: &Path) -> bool {
    let name = file.file_name().unwrap_or_default().to_string_lossy();
    let path = file.to_string_lossy();
    name.contains("generated")
        || name.starts_with("RustPerModTypes")
        || path.contains("subsystems_extra")
        || path.contains("generated_defs")
}

pub(crate) fn relative(root: &Path, file: &Path) -> String {
    file.strip_prefix(root).unwrap_or(file).display().to_string()
}

pub(crate) fn collect(directory: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    // `walkdir` rather than a hand-rolled recursion: it refuses to follow a symlink into a loop and
    // reports an unreadable directory the same way it reports a missing one (both dropped here).
    let walker = WalkDir::new(directory).into_iter().filter_entry(|entry| {
        !entry.file_type().is_dir()
            || !matches!(entry.file_name().to_string_lossy().as_ref(), "target" | "obj" | "bin" | "game-refs")
    });

    for entry in walker.flatten().filter(|entry| entry.file_type().is_file()) {
        if entry.file_name().to_string_lossy().ends_with(extension) {
            files.push(entry.into_path());
        }
    }
}
