use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use rand::{thread_rng, RngCore};
use std::fs;

const FILE_NAME: &str = "chopnplop";
const FIVE_HUNGE_KIB: usize = 500 * 1024;

#[test]
fn combined() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.child(FILE_NAME);
    let mut file_bytes = [0u8; FIVE_HUNGE_KIB];
    thread_rng().fill_bytes(&mut file_bytes);
    temp_file
        .write_binary(&file_bytes)
        .expect("Failed to write test bytes to temp file");

    // Chop
    // TODO: random splitting opcode & operand
    Command::cargo_bin("chop")
        .unwrap()
        .args(&["-n", "10", &temp_file.path().to_string_lossy()])
        .unwrap()
        .assert()
        .success();

    // TODO: validate successful chop

    // Stick
    Command::cargo_bin("stick")
        .unwrap()
        .current_dir(temp_dir.path())
        .arg(FILE_NAME)
        .unwrap()
        .assert()
        .success();

    // Test
    let reconstructed_bytes = fs::read(temp_file.path())
        .expect("Unable to find/read reconstructed file");
    assert_eq!(
        reconstructed_bytes.as_slice(),
        &file_bytes[..],
        "File contents differs",
    );
}

// TODO: large files
