use std::process::Command;

use crate::*;

/// The metadata of every asked type, from one harness run.
pub(crate) fn harness_members(references: &std::path::Path, types: &[String]) -> Result<Vec<TypeDump>> {
    let mut command = Command::new("dotnet");
    command
        .args(["run", "--project", "tools/abi-smoke", "--", "--members-json"])
        .args(types)
        .arg("--dir")
        .arg(references)
        .current_dir(repo_root());
    let output = command.output().map_err(|error| missing(format!("running abi-smoke: {error}")))?;
    if !output.status.success() {
        return Err(missing(format!("abi-smoke --members-json failed: {}", String::from_utf8_lossy(&output.stderr))));
    }

    serde_json::from_slice(&output.stdout).map_err(|error| missing(format!("reading abi-smoke's JSON: {error}")))
}

pub(crate) fn find_dump<'a>(dumps: &'a [TypeDump], kind: &str) -> Option<&'a TypeDump> {
    dumps.iter().find(|dump| dump.name == kind)
}

#[derive(serde::Deserialize)]
pub(crate) struct TypeDump {
    #[serde(rename = "type")]
    pub(crate) name: String,
    pub(crate) members: Vec<MemberDump>,
}

#[derive(serde::Deserialize, Clone)]
pub(crate) struct MemberDump {
    pub(crate) name: String,
    pub(crate) kind: String,

    #[serde(rename = "static")]
    pub(crate) is_static: bool,
    pub(crate) visibility: String,
    #[serde(rename = "type")]
    pub(crate) member_type: String,
    pub(crate) arity: Option<i64>,
}
