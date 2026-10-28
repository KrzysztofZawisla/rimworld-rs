use std::{fs, path::Path};

use crate::*;

/// Builds a mod and zips it for distribution (Steam Workshop or a manual install). One archive
/// carries every native target that was built: Steam Workshop items are platform-neutral, so the
/// mod folder itself is what decides which library the running game loads.
pub(crate) fn package(query: Option<&str>, path: Option<&Path>, targets: &[String], all_targets: bool) -> Result<()> {
    let info = find_mod(query, path)?;
    build(query, path, Mode::Build, targets, all_targets)?;

    let output = joined_within(&info.output_root.join("build"), &info.mod_name)?;
    let version = info.version.clone();
    let dist = info.output_root.join("dist");
    fs::create_dir_all(&dist)?;

    let zip = joined_within(&dist, &format!("{}-{}.zip", info.mod_name, version))?;
    let entries = collect_entries(&output, &output)?;
    write_zip(&zip, &entries)?;

    println!("==> packaged {} ({} files, {} KB)", zip.display(), entries.len(), file_size(&zip)? / 1024);
    println!("    the zip contains only the mod: About/, Assemblies/, Rust/<platform>/ and the mod's XML");
    Ok(())
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
