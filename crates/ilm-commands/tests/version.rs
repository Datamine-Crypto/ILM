use ilm_commands::command::version_text;
use std::path::PathBuf;

#[test]
fn the_version_of_the_program_is_the_version_of_the_workspace() {
    let crate_folder = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the folder of the crate"));
    let workspace = crate_folder.parent().and_then(|crates| crates.parent()).expect("the workspace holds the crate").join("Cargo.toml");
    let text = std::fs::read_to_string(&workspace).expect("the manifest of the workspace");
    let package = text.split("[workspace.package]").nth(1).expect("a part for the package");
    let version = package.lines().find_map(|line| line.trim().strip_prefix("version = ")).expect("a version line");
    assert_eq!(version.trim_matches('"'), version_text());
}
