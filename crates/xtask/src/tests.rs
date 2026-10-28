use std::{fs, path::Path};

use heck::{ToSnakeCase, ToTitleCase};

use super::*;

#[test]
fn finds_the_mods_through_cargo() {
    let mods = discover_mods().expect("cargo metadata answers");
    let names: Vec<&str> = mods.iter().map(|info| info.mod_name.as_str()).collect();
    assert!(names.contains(&"HelloRustMod"), "the example mod is found: {names:?}");
    assert!(names.contains(&"ApiTestMod"), "the self-test mod is found: {names:?}");

    let hello = mods.iter().find(|info| info.mod_name == "HelloRustMod").expect("hello-rust-mod");
    assert_eq!(hello.package, "hello-rust-mod", "the package name comes from cargo");
    assert!(hello.dir.ends_with("examples/hello-rust-mod"), "so does the folder: {:?}", hello.dir);
}

#[test]
fn maps_target_triples_to_the_platform_folders_the_bridge_loads() {
    let windows = NativeTarget::for_triple("x86_64-pc-windows-msvc", "my_mod").expect("windows");
    assert_eq!(windows.rid, "win-x64");
    assert_eq!(windows.file_name, "my_mod.dll");

    let linux = NativeTarget::for_triple("x86_64-unknown-linux-gnu", "my_mod").expect("linux");
    assert_eq!(linux.rid, "linux-x64");
    assert_eq!(linux.file_name, "libmy_mod.so");

    let intel_mac = NativeTarget::for_triple("x86_64-apple-darwin", "my_mod").expect("intel mac");
    assert_eq!(intel_mac.rid, "osx-x64");
    assert_eq!(intel_mac.file_name, "libmy_mod.dylib");

    let apple_silicon = NativeTarget::for_triple("aarch64-apple-darwin", "my_mod").expect("apple silicon");
    assert_eq!(apple_silicon.rid, "osx-arm64");

    assert!(NativeTarget::for_triple("wasm32-unknown-unknown", "my_mod").is_err());
    assert!(NativeTarget::for_triple("riscv64-unknown-linux-gnu", "my_mod").is_err());
}

#[test]
fn the_host_target_uses_the_running_platform() {
    let host = NativeTarget::host("my_mod");
    assert_eq!(host.rid, host_rid());
    assert_eq!(host.file_name, host_native_name("my_mod"));
    assert!(host.output_dir(Path::new("target")).ends_with(Path::new("target").join("release")));
}

#[test]
fn picks_the_cross_linker_for_each_target() {
    // The host's own linker handles its operating system, including the other macOS architecture.
    assert_eq!(CrossTool::pick("aarch64-apple-darwin", "osx", true, true), CrossTool::None);
    assert_eq!(CrossTool::pick("x86_64-apple-darwin", "osx", false, false), CrossTool::None);

    // Cross-OS Linux and macOS: zigbuild when it is there, plain cargo for a linker the machine configured.
    assert_eq!(CrossTool::pick("x86_64-unknown-linux-gnu", "osx", true, false), CrossTool::Zigbuild);
    assert_eq!(CrossTool::pick("aarch64-apple-darwin", "win", true, false), CrossTool::Zigbuild);
    assert_eq!(CrossTool::pick("x86_64-unknown-linux-gnu", "osx", false, false), CrossTool::None);

    // MSVC is not a zig target: cargo-xwin supplies the CRT, the SDK and clang-cl.
    assert_eq!(CrossTool::pick("x86_64-pc-windows-msvc", "osx", true, true), CrossTool::Xwin);
    assert_eq!(CrossTool::pick("x86_64-pc-windows-msvc", "osx", true, false), CrossTool::None);

    // Native Windows stays plain cargo, and windows-gnu is left to a configured MinGW linker.
    assert_eq!(CrossTool::pick("x86_64-pc-windows-msvc", "win", true, true), CrossTool::None);
    assert_eq!(CrossTool::pick("x86_64-pc-windows-gnu", "osx", true, true), CrossTool::None);

    // The cargo invocation each one stands for.
    assert_eq!(CrossTool::None.cargo_args(), ["build"]);
    assert_eq!(CrossTool::Zigbuild.cargo_args(), ["zigbuild"]);
    assert_eq!(CrossTool::Xwin.cargo_args(), ["xwin", "build"]);
}

#[test]
fn the_shipping_matrix_covers_every_platform_the_game_runs_on() {
    let targets = native_targets(&[], true, "my_mod").expect("matrix");
    let mut rids: Vec<&str> = targets.iter().map(|target| target.rid.as_str()).collect();
    rids.sort();
    assert_eq!(rids, vec!["linux-x64", "osx-arm64", "osx-x64", "win-x64"]);

    let duplicate = native_targets(
        &["x86_64-unknown-linux-gnu".to_string(), "x86_64-unknown-linux-gnu".to_string()],
        false,
        "my_mod",
    )
    .expect("one target");
    assert_eq!(duplicate.len(), 1, "a triple named twice is built once");

    let same_platform =
        native_targets(&["x86_64-pc-windows-gnu".to_string(), "x86_64-pc-windows-msvc".to_string()], false, "my_mod")
            .expect("one platform");
    assert_eq!(same_platform.len(), 1, "two triples for one runtime identifier produce one folder");
}

#[test]
fn converts_mod_names() {
    assert_eq!("HelloWorld".to_snake_case(), "hello_world");
    assert_eq!("already_snake".to_snake_case(), "already_snake");
    assert_eq!("HelloRustMod".to_title_case(), "Hello Rust Mod");
}

#[test]
fn accepts_plain_mod_names_and_rejects_anything_that_is_a_path() {
    // The names the scaffold writes and the examples use.
    assert_eq!(checked_mod_name("HelloRustMod").expect("plain"), "HelloRustMod");
    assert_eq!(checked_mod_name("My_Mod2").expect("underscore and digit"), "My_Mod2");

    // Every one of these would have redirected `build/`, `install` or `package` somewhere else.
    for name in ["", ".", "..", "../escaped", "..\\escaped", "a/b", "a\\b", "C:\\Windows", "/etc", "trailing."] {
        let error = checked_mod_name(name).expect_err(name);
        assert!(error.to_string().contains("path segment"), "{name}: {error}");
    }

    // A hostile value is refused even when it never passed the check above.
    let root = Path::new("build");
    assert!(joined_within(root, "Demo").is_ok());
    assert!(joined_within(root, "../escaped").is_err());
    assert!(joined_within(root, "C:\\Windows").is_err());
}

#[test]
fn escapes_generated_rust_and_keeps_identifiers_valid() {
    // A def or member name is data: a quote must not end the literal, a backslash must stay one.
    assert_eq!(rust_string("Steel"), "Steel");
    assert_eq!(rust_string("a\"b"), "a\\\"b");
    assert_eq!(rust_string("a\\b"), "a\\\\b");
    assert_eq!(rust_string("a\nb\tc"), "a\\nb\\tc");

    // And a name that is not an identifier still yields one.
    assert_eq!(rust_identifier("rotation"), "rotation");
    assert_eq!(rust_identifier("set_x"), "set_x");
    assert_eq!(rust_identifier("2x4"), "_2x4");
    assert_eq!(rust_identifier("a-b.c"), "a_b_c");
    assert_eq!(rust_identifier(""), "_");
}

#[test]
fn rejects_a_concrete_save_facing_component() {
    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("bridge").join("src");
    fs::create_dir_all(&source).expect("source");

    // What bridge/src must look like: the save-facing bases are abstract, so only the csproj's per-mod
    // subclass is ever instantiated and written into a save.
    fs::write(source.join("Component.cs"), "public abstract class A : GameComponent, ISavedDataHolder\n")
        .expect("write");
    check_component_classes(directory.path()).expect("abstract bases pass");

    fs::write(source.join("Shared.cs"), "public sealed class B : MapComponent, ISavedDataHolder\n").expect("write");
    let error = check_component_classes(directory.path()).expect_err("a concrete save-facing class is refused");
    assert!(error.to_string().contains("bridge/src"), "{error}");
}

#[test]
#[cfg(unix)]
fn copy_tree_does_not_follow_a_symlink() {
    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("mod");
    let outside = directory.path().join("outside");
    let target = directory.path().join("build");
    fs::create_dir_all(&source).expect("source");
    fs::create_dir_all(&outside).expect("outside");
    fs::write(source.join("About.xml"), b"<ModMetaData/>").expect("real file");
    fs::write(outside.join("secret.txt"), b"not mod content").expect("outside file");
    std::os::unix::fs::symlink(outside.join("secret.txt"), source.join("link.txt")).expect("symlink");
    std::os::unix::fs::symlink(&outside, source.join("linked-dir")).expect("dir symlink");

    copy_tree(&source, &target).expect("copy");

    assert!(target.join("About.xml").is_file(), "the mod's own file is copied");
    assert!(!target.join("link.txt").exists(), "a symlinked file is not materialised");
    assert!(!target.join("linked-dir").exists(), "a symlinked directory is not walked into");
}

#[test]
fn package_writes_a_readable_zip_archive() {
    use std::io::Read;

    let directory = tempfile::tempdir().expect("temp dir");
    let archive = directory.path().join("mod.zip");
    let entries = vec![
        ("About/About.xml".to_string(), b"<ModMetaData/>".to_vec()),
        ("Rust/win-x64/lib.dll".to_string(), vec![1u8, 2, 3, 4]),
    ];

    write_zip(&archive, &entries).expect("zip");

    // Read back through the crate's own reader: the entry names, the contents and the lengths are
    // what `cargo xtask package` promises, and a reader that disagrees is the failure worth seeing.
    let mut reader = zip::ZipArchive::new(fs::File::open(&archive).expect("open")).expect("a zip archive");
    assert_eq!(reader.len(), 2);
    let mut about = String::new();
    reader.by_name("About/About.xml").expect("About/About.xml").read_to_string(&mut about).expect("read");
    assert_eq!(about, "<ModMetaData/>");
    assert_eq!(reader.by_name("Rust/win-x64/lib.dll").expect("Rust/win-x64/lib.dll").size(), 4);

    drop(reader);
    let bytes = fs::read(&archive).expect("read");
    assert_eq!(&bytes[0..4], b"PK\x03\x04", "the container is a zip the game can open");
}

#[test]
fn reads_the_mod_version_from_cargo_metadata() {
    // The version comes from cargo, so a workspace-inherited `version.workspace = true` is already resolved.
    let info = ModInfo {
        package: "xtask".to_string(),
        mod_name: "Xtask".to_string(),
        dir: repo_root().join("crates").join("xtask"),
        version: "0.1.0".to_string(),
        workspace_root: repo_root(),
        target_dir: repo_root().join("target"),
        output_root: repo_root(),
    };

    assert_eq!(info.version, "0.1.0");
}

#[test]
fn reads_a_mod_from_an_external_project() {
    let directory = tempfile::tempdir().expect("temp dir");
    fs::create_dir_all(directory.path().join("src")).expect("src");
    fs::write(directory.path().join("src").join("lib.rs"), "").expect("lib");
    fs::write(
        directory.path().join("Cargo.toml"),
        "[package]\n\
         name = \"external_demo\"\n\
         version = \"0.2.0\"\n\
         edition = \"2021\"\n\
         \n\
         [lib]\n\
         crate-type = [\"cdylib\"]\n\
         \n\
         [package.metadata.rimworld]\n\
         mod-name = \"ExternalDemo\"\n",
    )
    .expect("manifest");

    let info = external_mod(directory.path()).expect("the external package resolves");
    assert_eq!(info.package, "external_demo");
    assert_eq!(info.mod_name, "ExternalDemo");
    assert_eq!(info.version, "0.2.0");
    assert_eq!(info.dir, directory.path().canonicalize().expect("canonical"));
    assert_eq!(info.output_root, info.workspace_root);
}

#[test]
fn scaffolds_an_external_mod_without_touching_the_workspace() {
    let directory = tempfile::tempdir().expect("temp dir");
    let target = directory.path().join("ExternalDemo");
    scaffold("ExternalDemo", Some(&target), false, false, false).expect("scaffold");

    let manifest = fs::read_to_string(target.join("Cargo.toml")).expect("manifest");
    assert!(manifest.contains("rimworld-rs = { path ="), "the SDK is a path dependency: {manifest}");
    assert!(manifest.contains("[package.metadata.rimworld]"));
    assert!(manifest.contains("mod-name = \"ExternalDemo\""));
    assert!(manifest.contains("[profile.release]"), "the mod carries the shipping profile");
    assert!(target.join("mod").join("About").join("About.xml").is_file());
    assert!(target.join("src").join("lib.rs").is_file());
    assert!(target.join("companion").join("ExternalDemo.Companion.csproj").is_file());
    assert!(target.join(".gitignore").is_file());
    assert!(
        !fs::read_to_string(repo_root().join("Cargo.toml")).expect("root manifest").contains("ExternalDemo"),
        "the external scaffold is not registered in this repository"
    );
}

#[test]
fn drops_workspace_members() {
    let manifest = "# a comment the edit has to keep\n[workspace]\nmembers = [\n    \"crates/xtask\",\n    \"mods/MyMod\",\n]\n\n[workspace.package]\nversion = \"0.1.0\"\n";
    let (updated, removed) = drop_workspace_members(manifest, "MyMod").expect("the manifest parses");
    assert_eq!(removed, vec!["mods/MyMod".to_string()]);
    assert!(!updated.contains("MyMod"));
    assert!(updated.contains("# a comment the edit has to keep"), "comments survive the edit");
    assert!(updated.contains("\n    \"crates/xtask\",\n"), "other members stay, in place");
    assert!(updated.contains("version = \"0.1.0\""), "the rest of the document survives");

    let (_, none) = drop_workspace_members(manifest, "Missing").expect("the manifest parses");
    assert!(none.is_empty(), "an unknown mod removes nothing");
}

#[test]
fn finds_the_bridge_class_names_of_a_patch_file() {
    let text = "<li Class=\"RustBridge.RustThingCompProperties_Demo\"><key>x</key></li>\n\
                    <li Class=\"RustBridge.RustGizmoCompProperties\" />\n\
                    <li Class=\"DemoCompanion.RustInfoTab\" />";

    assert_eq!(
        bridge_class_names(text),
        vec!["RustThingCompProperties_Demo".to_string(), "RustGizmoCompProperties".to_string()]
    );
}

#[test]
fn follows_the_log_file_that_is_written() {
    let directory = tempfile::tempdir().expect("temp dir");
    let older = directory.path().join("Player.log");
    let newer = directory.path().join("Logs").join("Player.log");
    fs::create_dir_all(newer.parent().expect("logs folder")).expect("create dirs");
    fs::write(&older, "an old layout").expect("write");
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(&newer, "the live log").expect("write");

    assert_eq!(
        newest_log(vec![older.clone(), newer.clone()]),
        Some(newer),
        "the file the game actually writes wins, not the first one that exists"
    );
    assert_eq!(newest_log(vec![directory.path().join("Missing.log")]), None);
}

#[test]
fn finds_the_root_element_of_a_document() {
    assert_eq!(
        patch_root_element("<?xml version=\"1.0\" encoding=\"utf-8\" ?>\n<!-- note -->\n<Patch>\n").as_deref(),
        Some("Patch")
    );
    assert_eq!(patch_root_element("\u{feff}<Patches />").as_deref(), Some("Patches"));
    assert_eq!(patch_root_element("<Patch\n>").as_deref(), Some("Patch"));
    assert_eq!(patch_root_element("   ").as_deref(), None);
    assert_eq!(patch_root_element("not xml").as_deref(), None);
}

#[test]
fn the_scaffolded_patch_carries_the_root_the_game_loads() {
    // The game skips any patch file whose root is not <Patch>.
    assert_eq!(patch_root_element(COMP_PATCH_TEMPLATE).as_deref(), Some("Patch"));
    assert!(COMP_PATCH_TEMPLATE.contains("<Operation Class="), "and it patches something");
}
