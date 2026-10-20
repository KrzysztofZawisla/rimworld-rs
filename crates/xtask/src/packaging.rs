use std::{fs, path::Path};

use crate::*;

/// Builds a mod and zips it for distribution (Steam Workshop or a manual install).
pub(crate) fn package(query: Option<&str>) -> Result<()> {
    let info = find_mod(query)?;
    build(query, Mode::Build)?;

    let root = repo_root();
    let output = joined_within(&root.join("build"), &info.mod_name)?;
    let version = mod_version(&info)?;
    let dist = root.join("dist");
    fs::create_dir_all(&dist)?;

    let zip = joined_within(&dist, &format!("{}-{}.zip", info.mod_name, version))?;
    let entries = collect_entries(&output, &output)?;
    write_zip(&zip, &entries)?;

    println!("==> packaged {} ({} files, {} KB)", zip.display(), entries.len(), file_size(&zip)? / 1024);
    println!("    the zip contains only the mod: About/, Assemblies/, Rust/ and the mod's XML");
    Ok(())
}

/// The version of a mod, as cargo resolves it (`version.workspace = true` included, because cargo is the one
/// answering). A package cargo knows always has a version, so the guesswork is gone: no `0.0.0` fallback and no
/// second file read for the workspace table.
pub(crate) fn mod_version(info: &ModInfo) -> Result<String> {
    let packages = workspace_packages()?;
    let package = packages
        .iter()
        .find(|package| package.name == info.package)
        .ok_or_else(|| missing(format!("'{}' is not a package of this workspace", info.package)))?;

    Ok(package.version.to_string())
}

/// Reads a built mod folder into zip entries (relative path with `/` separators, bytes). `walkdir` walks it - the
/// same crate the layout rule uses, so the repository has one walker instead of two.
pub(crate) fn collect_entries(directory: &Path, base: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut entries = Vec::new();
    for entry in walkdir::WalkDir::new(directory).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let relative =
            path.strip_prefix(base).map_err(|error| missing(error.to_string()))?.to_string_lossy().replace('\\', "/");
        entries.push((relative, fs::read(path)?));
    }

    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(entries)
}

pub(crate) fn file_size(path: &Path) -> Result<u64> {
    Ok(fs::metadata(path)?.len())
}

/// Writes a ZIP archive of the mod's files, Deflated so a Workshop upload is not the size of the binaries.
/// The `zip` crate carries the container and the CRC-32; a hand-written writer is exactly the code no one
/// wants to maintain (offsets, central directory, two CRC copies per entry).
pub(crate) fn write_zip(path: &Path, entries: &[(String, Vec<u8>)]) -> Result<()> {
    use std::io::Write;

    let mut writer = zip::ZipWriter::new(fs::File::create(path)?);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, content) in entries {
        writer.start_file(name.as_str(), options)?;
        writer.write_all(content)?;
    }

    writer.finish()?;
    Ok(())
}

// ---------------------------------------------------------------- log watching
