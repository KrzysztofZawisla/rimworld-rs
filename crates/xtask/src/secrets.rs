use crate::*;

/// Scans for committed secrets with gitleaks. The default mode scans the whole git history; `--staged` is the
/// fast mode the pre-commit hook uses, so a secret is stopped before it ever enters a commit.
pub(crate) fn secrets(staged: bool) -> Result<()> {
    let root = repo_root();
    let args: Vec<&str> = if staged {
        vec!["protect", "--staged", "--source", ".", "--config", ".gitleaks.toml", "--redact", "--verbose"]
    } else {
        vec!["git", "--config", ".gitleaks.toml", "--redact", "--verbose"]
    };

    run_optional_gate("gitleaks", &args, &root)
}
