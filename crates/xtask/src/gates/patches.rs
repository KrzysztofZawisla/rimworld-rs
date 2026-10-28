use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::*;

/// Checks every `Patches/*.xml` this repository ships.
///
/// RimWorld skips a patch file whose root element is not `<Patch>`; it only logs "Unexpected
/// document element in patch XML", so the feature the file adds silently does not exist in game.
/// Only the game shows that, which is why the rule is enforced here.
pub(crate) fn check_patch_files(root: &Path) -> Result<()> {
    println!("==> patch file check");

    let mut files = Vec::new();
    for folder in ["examples", "mods"] {
        collect_patch_files(&root.join(folder), &mut files)?;
    }
    files.sort();

    let mods = discover_mods().unwrap_or_default();
    let mut findings = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file)?;
        let shown = file.strip_prefix(root).unwrap_or(file).display();
        match patch_root_element(&text).as_deref() {
            Some("Patch") => {
                if !text.contains("<Operation") {
                    findings.push(format!("{shown}: <Patch> holds no <Operation>"));
                }
            }
            Some(other) => findings
                .push(format!("{shown}: root element is <{other}>, the game only loads <Patch> and skips this file")),
            None => findings.push(format!("{shown}: no root element")),
        }

        // The bridge ships one copy of every XML-facing class per mod, and the game resolves a patch's
        // `Class` across all loaded assemblies, by load order: two Rust mods that both name the shared
        // class hand the comp to whichever bridge loaded first, and the other mod's Rust never sees its
        // events. Only a log line in game would show that, so it is enforced here.
        if let Some(info) = mods.iter().find(|info| file.starts_with(&info.dir)) {
            for name in bridge_class_names(&text) {
                if !name.ends_with(&format!("_{}", info.mod_name)) {
                    findings.push(format!(
                        "{shown}: Class=\"RustBridge.{name}\" is shared by every mod's bridge; name \"RustBridge.{name}_{}\" instead",
                        info.mod_name
                    ));
                }
            }
        }
    }

    if findings.is_empty() {
        println!("    {} patch file(s) use the <Patch> root", files.len());
        return Ok(());
    }

    for finding in &findings {
        println!("    {finding}");
    }
    Err(missing(format!("{} patch file(s) would be skipped by the game", findings.len())))
}

/// Collects `<mod>/Patches/*.xml` below a directory.
pub(crate) fn collect_patch_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !directory.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_patch_files(&path, files)?;
        } else if path.extension() == Some("xml".as_ref())
            && path.parent().and_then(Path::file_name).is_some_and(|name| name == "Patches")
        {
            files.push(path);
        }
    }

    Ok(())
}

/// Every class name a patch file gives in the bridge's namespace (`Class="RustBridge.X"`).
///
/// The `RustBridge.*` class names a patch file names, read with `quick-xml`: the parser owns the quoting, the
/// escaping and what an attribute is. The same words inside a comment no longer count, and a single-quoted
/// `Class` - which XML allows - now counts, where the hand-written scan only knew the double-quoted form.
pub(crate) fn bridge_class_names(text: &str) -> Vec<String> {
    const PREFIX: &str = "RustBridge.";

    let mut names = Vec::new();
    let mut reader = quick_xml::Reader::from_str(text);
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(element)) | Ok(quick_xml::events::Event::Empty(element)) => {
                for attribute in element.attributes().flatten() {
                    if attribute.key.as_ref() != "Class" {
                        continue;
                    }

                    let value = attribute.value.as_ref();
                    if let Some(name) = value.strip_prefix(PREFIX) {
                        names.push(name.to_string());
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }

    names
}

/// The name of a document's first element, read with `quick-xml`: it skips the declaration, the comments and the
/// whitespace on its own, an empty element (`<Patches />`) counts as the root, and a document that is not XML
/// answers `None`. Only the byte-order mark is trimmed here, before the parser sees it.
pub(crate) fn patch_root_element(text: &str) -> Option<String> {
    let mut reader = quick_xml::Reader::from_str(text.trim_start_matches('\u{feff}'));
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Start(element)) | Ok(quick_xml::events::Event::Empty(element)) => {
                return Some(element.name().as_ref().to_string())
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

/// A component class the save file names has to be per mod. The csproj generates one concrete subclass per
/// bridge (`RustGameComponent_<Mod>`, `RustMapComponent_<Mod>`) and the hand-written bases stay abstract, so the
/// game only ever instantiates - and only ever writes into a save - the per-mod type. A concrete save-facing
/// class in `bridge/src` would be one name in every mod's assembly, and the game resolves a name across all of
/// them by load order: the second mod's Rust would then read and write the first mod's save data, or find no
/// component at all.
///
/// A thing comp is deliberately not in this set: it is created from its def, not from a name in the save, and
/// its per-thing state lives in the game component's store.
pub(crate) fn check_component_classes(root: &Path) -> Result<()> {
    println!("==> per-mod component check");

    let directory = root.join("bridge").join("src");
    let mut measured = 0;
    let mut findings = Vec::new();
    for entry in fs::read_dir(&directory)?.flatten() {
        let path = entry.path();
        if !path.extension().is_some_and(|extension| extension == "cs") {
            continue;
        }

        let text = fs::read_to_string(&path)?;
        for (index, line) in text.lines().enumerate() {
            let declaration = line.trim();
            let Some((_, bases)) = declaration.split_once(": ") else {
                continue;
            };
            if !declaration.contains("class ") {
                continue;
            }

            let save_facing = bases.split(',').any(|base| {
                let base = base.trim().trim_end_matches('{').trim();
                base == "GameComponent" || base == "MapComponent"
            });
            if !save_facing {
                continue;
            }

            measured += 1;
            if !declaration.contains("abstract ") {
                findings.push(format!(
                    "{}:{}: {declaration}",
                    path.strip_prefix(root).unwrap_or(&path).display(),
                    index + 1
                ));
            }
        }
    }

    if !findings.is_empty() {
        for finding in &findings {
            println!("FAIL  {finding}");
        }

        return Err(missing(format!(
            "{} concrete game/map component class(es) in bridge/src: the save file records a component by class name, so the concrete type must come from the csproj per mod (keep the base abstract and let WritePerModTypes generate the subclass)",
            findings.len()
        )));
    }

    println!("    {measured} save-facing component class(es) are abstract (per-mod subclasses come from the csproj)");
    Ok(())
}
