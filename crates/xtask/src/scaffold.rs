//! Theme: scaffold. Split from `main.rs`.
#![allow(unused_imports)]

use std::{
    env, fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

use heck::{ToSnakeCase, ToTitleCase};

use crate::*;

mod templates;

pub(crate) use templates::*;

pub(crate) fn scaffold(name: &str, with_comp: bool, with_job: bool, with_itab: bool) -> Result<()> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(missing("the mod name must be alphanumeric (for example `MyFirstMod`)"));
    }

    let root = repo_root();
    let directory = root.join("mods").join(name);
    if directory.exists() {
        return Err(missing(format!("{} already exists", directory.display())));
    }

    let package = name.to_snake_case();
    let package_id = format!("rimworld.rs.{}", package);
    let display = name.to_title_case();

    fs::create_dir_all(directory.join("src"))?;
    fs::create_dir_all(directory.join("mod").join("About"))?;

    fs::write(
        directory.join("Cargo.toml"),
        format!(
            "[package]\n\
             name = \"{package}\"\n\
             version.workspace = true\n\
             edition.workspace = true\n\
             license.workspace = true\n\
             \n\
             [lib]\n\
             crate-type = [\"cdylib\"]\n\
             \n\
             [dependencies]\n\
             rimworld-rs = {{ workspace = true }}\n\
             serde_json = {{ workspace = true }}\n\
             \n\
             [package.metadata.rimworld]\n\
             mod-name = \"{name}\"\n"
        ),
    )?;

    fs::write(directory.join("src").join("lib.rs"), LIB_TEMPLATE.replace("{display}", &display))?;

    let companion = directory.join("companion");
    fs::create_dir_all(&companion)?;
    fs::write(companion.join(format!("{name}.Companion.csproj")), COMPANION_PROJECT_TEMPLATE.replace("{name}", name))?;
    fs::write(
        companion.join("Companion.cs"),
        COMPANION_TEMPLATE.replace("{name}", name).replace("{namespace}", &format!("{name}Companion")),
    )?;

    let namespace = format!("{name}Companion");
    let mut samples: Vec<&str> = Vec::new();
    if with_comp {
        fs::write(
            companion.join("OnSpawnComp.cs"),
            THING_COMP_TEMPLATE.replace("{name}", name).replace("{namespace}", &namespace),
        )?;
        fs::create_dir_all(directory.join("mod").join("Patches"))?;
        fs::write(
            directory.join("mod").join("Patches").join(format!("{name}Comp.xml")),
            COMP_PATCH_TEMPLATE.replace("{namespace}", &namespace),
        )?;
        samples.push("ThingComp (OnSpawnComp + mod/Patches)");
    }

    if with_job {
        fs::write(
            companion.join("WalkToCellJobDriver.cs"),
            JOB_DRIVER_TEMPLATE.replace("{name}", name).replace("{namespace}", &namespace),
        )?;
        samples.push("JobDriver (WalkToCellJobDriver)");
    }

    if with_itab {
        fs::write(
            companion.join("RustInfoTab.cs"),
            INSPECTOR_TAB_TEMPLATE.replace("{name}", name).replace("{namespace}", &namespace),
        )?;
        samples.push("ITab (RustInfoTab)");
    }

    fs::write(
        directory.join("mod").join("About").join("About.xml"),
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
             <ModMetaData>\n\
             \t<name>{display}</name>\n\
             \t<author>anonymous</author>\n\
             \t<packageId>{package_id}</packageId>\n\
             \t<description>A RimWorld mod written in Rust.</description>\n\
             \t<supportedVersions>\n\
             \t\t<li>1.7</li>\n\
             \t</supportedVersions>\n\
             </ModMetaData>\n"
        ),
    )?;

    println!("==> created {}", directory.display());
    add_workspace_member(name)?;
    println!("    build it with: cargo xtask build {name}");
    println!(
        "    companion/{name}.Companion.csproj is compiled too; managed game types (ThingComp,\n    JobDriver, ITab, Window, ...) belong there - see docs/companion.md"
    );
    for sample in samples {
        println!("    sample included: {sample}");
    }

    println!("    def-name constants: run `cargo xtask gen-defs {name}` and use `mod defs;`");
    Ok(())
}
