//! Documentation paths. Markdown links and repository paths must still resolve.

use std::{fs, path::Path};

use pulldown_cmark::{Event, Options, Parser, Tag};

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

/// The extensions these documents are written with: a link inside a table cell is still a link.
fn markdown_options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS
}

/// One file's local links and repository paths.
///
/// A CommonMark parser decides what a link is. A `](` inside a fenced block or an inline code span is text
/// that shows how a link is written, not a link, and a destination with a parenthesis (`notes(1).md`) is one
/// destination instead of one cut short at the bracket.
fn check_file(path: &Path, root: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).map_err(|error| missing(format!("reading {}: {error}", path.display())))?;
    let directory = path.parent().unwrap_or(root);
    let mut problems = Vec::new();

    for event in Parser::new_ext(&text, markdown_options()) {
        match event {
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                let Some(target) = local_target(&dest_url) else {
                    continue;
                };

                if !directory.join(&target).exists() && !root.join(&target).exists() {
                    problems.push(format!("{}: link `{target}` does not resolve", path.display()));
                }
            }
            Event::Code(span) => {
                let Some(candidate) = repository_path(&span) else {
                    continue;
                };

                if !root.join(&candidate).exists() && !directory.join(&candidate).exists() {
                    problems.push(format!("{}: path `{candidate}` does not exist", path.display()));
                }
            }
            _ => {}
        }
    }

    Ok(problems)
}

/// The path a local link names, or `None` for an external URL, a pure anchor or a placeholder.
fn local_target(destination: &str) -> Option<String> {
    let target = destination.trim();
    let external = ["http://", "https://", "mailto:", "steam://", "file://"];
    if target.is_empty()
        || target.starts_with('#')
        || external.iter().any(|scheme| target.starts_with(scheme))
        || !is_plain_path(target)
    {
        return None;
    }

    let target = target.split(['#', '?']).next().unwrap_or("");
    (!target.is_empty()).then(|| target.to_string())
}

/// The repository path an inline code span names, if it names one at all.
fn repository_path(span: &str) -> Option<String> {
    let span = span.trim();
    if span.is_empty() || span.contains('\n') || !looks_like_repository_path(span) {
        return None;
    }

    Some(span.trim_start_matches("./").to_string())
}

/// A literal path, not prose about one: `*`, `{}`, `$`, `%` and `...` mark a template, and a space or a
/// newline means the span is a sentence. Parentheses are allowed - a parser reads `notes(1).md` as one path.
fn is_plain_path(target: &str) -> bool {
    !target.is_empty()
        && !target.contains(['<', '>', '*', '{', '}', '$', '%', '|', '\n', ' '])
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

    fn page(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let root = tempfile::tempdir().expect("temp dir");
        fs::create_dir_all(root.path().join("docs")).expect("mkdir");
        let page = root.path().join("docs/page.md");
        fs::write(&page, content).expect("write");
        (root, page)
    }

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
    fn reports_a_missing_link() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::create_dir_all(root.path().join("docs")).expect("mkdir");
        let page = root.path().join("docs/page.md");
        fs::write(&page, "[gone](missing.md)\n").expect("write");

        let problems = check_file(&page, root.path()).expect("check");
        assert!(problems.iter().any(|problem| problem.contains("missing.md")), "{problems:?}");
    }

    #[test]
    fn reports_a_missing_repository_path() {
        let (root, page) = page("`crates/rimworld-rs/src/gone.rs`\n");

        let problems = check_file(&page, root.path()).expect("check");
        assert!(problems.iter().any(|problem| problem.contains("gone.rs")), "{problems:?}");
    }

    #[test]
    fn ignores_markdown_that_only_shows_how_a_link_is_written() {
        // A fenced block and an inline code span are examples: `](` there is text, and the old scan for `](`
        // read both as links to a file that was never meant to exist.
        let (root, page) = page("```markdown\n[gone](missing.md)\n```\n\nWrite `[name](missing.md)` to link.\n");

        assert!(check_file(&page, root.path()).expect("check").is_empty());
    }

    #[test]
    fn reads_a_destination_with_a_parenthesis_whole() {
        let (root, page) = page("[notes](notes(1).md)\n![figure](figure(2).png)\n");
        fs::write(root.path().join("docs/notes(1).md"), "").expect("write");
        fs::write(root.path().join("docs/figure(2).png"), "").expect("write");

        assert!(check_file(&page, root.path()).expect("check").is_empty());
    }

    #[test]
    fn checks_an_image_target_like_a_link() {
        let (root, page) = page("![gone](missing.png)\n");

        let problems = check_file(&page, root.path()).expect("check");
        assert!(problems.iter().any(|problem| problem.contains("missing.png")), "{problems:?}");
    }
}
