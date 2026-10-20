use std::{fs, path::Path};

use heck::{ToSnakeCase, ToTitleCase};

use crate::*;

mod templates;

pub(crate) use templates::*;

/// Creates a mod under `mods/<Name>/` in this repository by default, where it is registered in the workspace.
/// With `--path <DIR>` it creates a standalone project in that directory instead, outside this repository.
pub(crate) fn scaffold(
    name: &str,
    path: Option<&Path>,
    with_comp: bool,
    with_job: bool,
    with_itab: bool,
) -> Result<()> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(missing("the mod name must be alphanumeric (for example `MyFirstMod`)"));
    }

    let repo = repo_root();
    let (directory, external) = match path {
        Some(path) => (
            std::path::absolute(path)
                .map_err(|error| missing(format!("cannot resolve {}: {error}", path.display())))?,
            true,
        ),
        None => (repo.join("mods").join(name), false),
    };
    if directory.exists() && fs::read_dir(&directory)?.next().is_some() {
        return Err(missing(format!("{} already exists and is not empty", directory.display())));
    }

    let package = name.to_snake_case();
    let package_id = format!("rimworld.rs.{}", package);
    let display = name.to_title_case();
    let sdk = repo.join("crates").join("rimworld-rs").to_string_lossy().replace('\\', "/");

    fs::create_dir_all(directory.join("src"))?;
    fs::create_dir_all(directory.join("mod").join("About"))?;
    fs::create_dir_all(directory.join("companion"))?;

    fs::write(directory.join("Cargo.toml"), cargo_manifest(name, &package, &sdk, external))?;
    let gitignore = if external {
        "/target\n/build\n/dist\n/src/generated_defs/\n/companion/bin/\n/companion/obj/\n"
    } else {
        "/src/generated_defs/\n"
    };
    fs::write(directory.join(".gitignore"), gitignore)?;

    fs::write(directory.join("src").join("lib.rs"), LIB_TEMPLATE.replace("{display}", &display))?;

    let companion = directory.join("companion");
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

    if !external {
        add_workspace_member(name)?;
    }

    println!("==> created {}", directory.display());
    if external {
        println!("    build it with: cargo xtask build --path {}", directory.display());
        println!("    the source stays outside this repository; it was not added to its workspace");
    } else {
        println!("    build it with: cargo xtask build {name}");
        println!(
            "    the mod is registered in this repository's workspace; delete it with `cargo xtask remove {name}`"
        );
    }
    println!(
        "    companion/{name}.Companion.csproj is compiled too; managed game types (ThingComp,\n    JobDriver, ITab, Window, ...) belong there - see docs/companion.md"
    );
    for sample in samples {
        println!("    sample included: {sample}");
    }

    let defs_hint = if external {
        format!("cargo xtask gen-defs --path {}", directory.display())
    } else {
        format!("cargo xtask gen-defs {name}")
    };
    println!("    def-name constants: run `{defs_hint}` and use `mod defs;`");
    Ok(())
}

/// Builds a `Cargo.toml` with `toml_edit` instead of formatting the TOML by hand. The default scaffold
/// inherits version/edition/license and the SDK dependency from this repository's workspace; `--path`
/// writes a standalone package with its own shipping release profile.
fn cargo_manifest(name: &str, package: &str, sdk: &str, external: bool) -> String {
    use toml_edit::{Array, DocumentMut, InlineTable, Item, Table};

    fn inherited() -> InlineTable {
        let mut table = InlineTable::new();
        table.insert("workspace", true.into());
        table
    }

    let mut document = DocumentMut::new();
    let mut package_table = Table::new();
    package_table["name"] = toml_edit::value(package);
    if external {
        package_table["version"] = toml_edit::value("0.1.0");
        package_table["edition"] = toml_edit::value("2021");
        package_table["license"] = toml_edit::value("MIT");
    } else {
        package_table["version"] = toml_edit::value(inherited());
        package_table["edition"] = toml_edit::value(inherited());
        package_table["license"] = toml_edit::value(inherited());
    }

    let mut rimworld = Table::new();
    rimworld["mod-name"] = toml_edit::value(name);
    let mut metadata = Table::new();
    metadata["rimworld"] = Item::Table(rimworld);
    package_table["metadata"] = Item::Table(metadata);
    document["package"] = Item::Table(package_table);

    let mut lib = Table::new();
    let mut crate_types = Array::new();
    crate_types.push("cdylib");
    lib["crate-type"] = toml_edit::value(crate_types);
    document["lib"] = Item::Table(lib);

    let mut dependencies = Table::new();
    let mut rimworld_dependency = InlineTable::new();
    if external {
        rimworld_dependency.insert("path", sdk.into());
    } else {
        rimworld_dependency.insert("workspace", true.into());
    }
    dependencies["rimworld-rs"] = toml_edit::value(rimworld_dependency);
    if external {
        dependencies["serde_json"] = toml_edit::value("1");
    } else {
        dependencies["serde_json"] = toml_edit::value(inherited());
    }
    document["dependencies"] = Item::Table(dependencies);

    if external {
        let mut release = Table::new();
        release["opt-level"] = toml_edit::value(3);
        release["lto"] = toml_edit::value("fat");
        release["codegen-units"] = toml_edit::value(1);
        release["panic"] = toml_edit::value("unwind");
        release["incremental"] = toml_edit::value(false);
        release["debug"] = toml_edit::value(false);
        release["overflow-checks"] = toml_edit::value(false);
        let mut profile = Table::new();
        profile["release"] = Item::Table(release);
        document["profile"] = Item::Table(profile);
    }

    document.to_string()
}
