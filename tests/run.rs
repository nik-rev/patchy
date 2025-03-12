use tempfile::{tempdir, TempDir};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use copy_dir::copy_dir;
use std::process::Command;

fn initialize(repository: &str, branch: &str, pull_requests: Vec<&str>, patches: Vec<&str>) -> TempDir {
  let temp_dir = tempdir().expect("tempdir failed");

  Command::new("git")
    .args(["init"])
    .current_dir(temp_dir.path())
    .output()
    .expect("git init failed");
  
    copy_dir("tests/fixtures/patches", temp_dir.path().join(".patchy")).expect("copy_dir failed");

  std::fs::write(temp_dir.path().join(".patchy").join("config.toml"), format!("
repo = \"{repository}\"
remote-branch = \"{branch}\"
local-branch = \"patchy\"
pull-requests = {pull_requests:?}
patches = {patches:?}
")).expect("writing config.toml failed");

  Command::new("git")
    .args(["add", ".patchy"])
    .current_dir(temp_dir.path())
    .output()
    .expect("git add failed");

  Command::new("git")
    .args(["commit", "-m=initial commit"])
    .current_dir(temp_dir.path())
    .output()
    .expect("git commit failed");

  temp_dir
}

#[test]
fn test_helix_remove_tab() {
    let tmp = initialize("helix-editor/helix", "master", vec![], vec!["helix-remove-tab"]);

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(["run", "--yes"])
        .current_dir(tmp.as_ref())
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Applied patch helix-remove-tab feat: remove tab keybindings"))
        .stdout(predicate::str::contains("Success!"));
}

#[test]
fn test_conflicting_patches() {
    let tmp = initialize("helix-editor/helix", "master", vec![], vec!["helix-readme-all-every", "helix-readme-all-most", "helix-readme-all-some"]);

    std::process::Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(["run", "--yes"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("✓ Applied patch helix-readme-all-every patch-every"))
        .stderr(predicate::str::contains("✗ Could not apply patch helix-readme-all-most, skipping"))
        .stdout(predicate::str::contains("Success!").not());
}

#[test]
fn test_sequential_patches() {
    let tmp = initialize("helix-editor/helix", "master", vec![], vec!["helix-readme-all-some", "helix-readme-some-most", "helix-readme-most-every"]);

    std::process::Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(["run", "--yes"])
        .current_dir(tmp.path())
        .assert()
        // This should pass, as the patches are applied in order
        .success()
        .stdout(predicate::str::contains("✓ Applied patch helix-readme-all-some patch-some"))
        .stdout(predicate::str::contains("✓ Applied patch helix-readme-some-most patch-most"))
        .stdout(predicate::str::contains("✓ Applied patch helix-readme-most-every patch-every"))
        .stdout(predicate::str::contains("Success!"));
}

#[test]
fn test_nonexistent_patch() {
    let tmp = initialize("helix-editor/helix", "master", vec![], vec!["foo"]);

    std::process::Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .args(["run", "--yes"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("✗ Could not find patch foo, skipping"))
        .stdout(predicate::str::contains("Success!"));
}