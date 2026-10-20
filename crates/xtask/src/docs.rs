//! Documentation paths. Markdown links and repository paths must still resolve.

use std::{fs, path::Path};

use crate::*;

/// Checks every Markdown file: local links must resolve, and a `crates/`, `examples/`, `docs/`,
/// `bridge/`, `tools/` or `skills/` path in an inline code span must exist in the repository.
pub(crate) fn check_markdown() -> Result<()> {
    let root = repo_root();
    let mut problems = Vec::new();
    let mut files = 0usize;

    for entry in walkdir::WalkDir::new(&root).into_iter().filter_entry(|entry| {
        !entry.file_type().is_dir()
            || !matches!(
                entry.file_name().to_string_lossy().as_ref(),
                "target" | "build" | "dist" | "game-refs" | "obj" | "bin" | ".git" | ".commandcode"
            )
    }) {
        let entry = entry.map_err(|error| missing(format!("walking the repository: {error}")))?;
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|extension| extension == "md") {
            files += 1;
            problems.extend(check_file(entry.path(), &root)?);
        }
    }

    for problem in &problems {
        println!("  docs  {problem}");
    }
    if problems.is_empty() {
        println!("{files} Markdown file(s): every local link and repository path resolves");
        return Ok(());
    }

    Err(missing(format!("{} Markdown link/path problem(s)", problems.len())))
}

fn check_file(path: &Path, root: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).map_err(|error| missing(format!("reading {}: {error}", path.display())))?;
    let directory = path.parent().unwrap_or(root);
    let mut problems = Vec::new();

    let mut index = 0usize;
    while let Some(relative) = text[index..].find("](") {
        let start = index + relative + 2;
        let Some(end_relative) = text[start..].find(')') else {
            break;
        };
        let target = text[start..start + end_relative].trim();
        index = start + end_relative + 1;

        let Some(target) = target.split_whitespace().next() else {
            continue;
        };
        let target = target.trim_matches(|character| character == '<' || character == '>');
        if !is_local_link(target) {
            continue;
        }
        let target = target.split(['#', '?']).next().unwrap_or("");
        if target.is_empty() || !is_plain_path(target) {
            continue;
        }

        if !directory.join(target).exists() && !root.join(target).exists() {
            problems.push(format!("{}: link `{target}` does not resolve", path.display()));
        }
    }

    let mut index = 0usize;
    while let Some(relative) = text[index..].find('`') {
        let start = index + relative;
        let Some(end_relative) = text[start + 1..].find('`') else {
            break;
        };
        let end = start + 1 + end_relative;
        let content = &text[start + 1..end];
        index = end + 1;

        if content.is_empty() || content.contains('\n') || !looks_like_repository_path(content) {
            continue;
        }
        let candidate = content.trim_start_matches("./");
        if !root.join(candidate).exists() && !directory.join(candidate).exists() {
            problems.push(format!("{}: path `{candidate}` does not exist", path.display()));
        }
    }

    Ok(problems)
}

fn is_local_link(target: &str) -> bool {
    !target.is_empty()
        && !target.starts_with('#')
        && !target.starts_with("http://")
        && !target.starts_with("https://")
        && !target.starts_with("mailto:")
        && !target.starts_with("steam://")
        && !target.starts_with("file://")
        && is_plain_path(target)
}

fn is_plain_path(target: &str) -> bool {
    !target.is_empty()
        && !target.contains(['<', '>', '*', '{', '}', '$', '%', '|', '\n', '(', ')', ' '])
        && !target.contains("...")
}

fn looks_like_repository_path(content: &str) -> bool {
    let content = content.trim().trim_start_matches("./");
    let known = ["crates/", "examples/", "docs/", "bridge/", "tools/", "skills/"];
    let local = ["build/", "target/", "dist/", "game-refs/", "mods/", "obj/", "bin/", "src/", "generated_defs"];

    if local.iter().any(|prefix| content.starts_with(prefix)) || content.contains("generated_defs") {
        return false;
    }

    if known.iter().any(|prefix| content.starts_with(prefix)) {
        return is_plain_path(content);
    }

    matches!(
        content,
        "AGENTS.md" | "README.md" | "LICENSE" | "Cargo.toml" | "rustfmt.toml" | "typos.toml" | ".gitignore"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_resolving_links_and_paths() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::create_dir_all(root.path().join("docs")).expect("mkdir");
        fs::create_dir_all(root.path().join("crates/rimworld-rs/src/subsystems_extra")).expect("mkdir");
        fs::write(root.path().join("docs/other.md"), "").expect("write");
        let page = root.path().join("docs/page.md");
        fs::write(&page, "[other](other.md)\n`crates/rimworld-rs/src/subsystems_extra/`\n").expect("write");

        assert!(check_file(&page, root.path()).expect("check").is_empty());
    }

    #[test]
    fn reports_a_missing_link_and_an_old_path() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::create_dir_all(root.path().join("docs")).expect("mkdir");
        let page = root.path().join("docs/page.md");
        fs::write(&page, "[gone](missing.md)\n`crates/rimworld-rs/src/subsystems_extra.rs`\n").expect("write");

        let problems = check_file(&page, root.path()).expect("check");
        assert!(problems.iter().any(|problem| problem.contains("missing.md")), "{problems:?}");
        assert!(problems.iter().any(|problem| problem.contains("subsystems_extra.rs")), "{problems:?}");
    }
}
