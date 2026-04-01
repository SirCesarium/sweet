// @swt-disable max-repetition
//! Integration tests for the Sweet CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Tests the basic analyzer execution on an empty directory.
#[test]
fn test_cli_empty_dir() {
    let mut cmd = Command::cargo_bin("swt").expect("binary should exist");
    let dir = tempdir().expect("failed to create temp dir");

    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No supported files found"));
}

/// Tests that the CLI fails when analyzing a project that violates thresholds.
#[test]
fn test_cli_violation_failure() {
    let dir = tempdir().expect("failed to create temp dir");
    let file_path = dir.path().join("main.rs");

    // Create a file with many lines to trigger a violation
    let content = "println!(\"sweet\");\n".repeat(400);
    fs::write(&file_path, content).expect("failed to write test file");

    let mut cmd = Command::cargo_bin("swt").expect("binary should exist");
    cmd.arg(dir.path()).assert().failure();
}

/// Tests the JSON output format.
#[test]
fn test_cli_json_output() {
    let dir = tempdir().expect("failed to create temp dir");
    let file_path = dir.path().join("test.rs");
    // Create a file with many lines to trigger a violation
    let content = "fn main() {}\n".repeat(410);
    fs::write(&file_path, content).expect("failed to write test file");

    let mut cmd = Command::cargo_bin("swt").expect("binary should exist");
    cmd.arg(dir.path())
        .arg("--json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"line\": 1"))
        .stdout(predicate::str::contains("\"message\": \"File too long"));
}

/// Tests that ignored files are not analyzed.
#[test]
fn test_cli_ignore_logic() {
    let dir = tempdir().expect("failed to create temp dir");
    let file_path = dir.path().join("ignored.rs");
    fs::write(&file_path, "// @sweetignore\nfn main() {}").expect("failed to write test file");

    let mut cmd = Command::cargo_bin("swt").expect("binary should exist");
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No supported files found"));
}
