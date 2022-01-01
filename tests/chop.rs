use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use chopstick::EXTENSION_PREFIX;
use std::cmp::min;

const FILE_NAME: &str = "split_me";
const TEST_BYTES: [u8; 100] = [
    164, 108, 152, 89, 172, 190, 243, 194, 202, 143, 158, 187, 192, 211, 33,
    195, 34, 27, 108, 57, 177, 144, 199, 135, 136, 143, 57, 246, 45, 100, 247,
    59, 163, 101, 168, 68, 244, 190, 137, 114, 216, 67, 112, 196, 124, 170, 74,
    78, 35, 53, 204, 163, 235, 101, 179, 30, 51, 41, 9, 199, 125, 89, 132, 75,
    221, 221, 102, 190, 51, 255, 246, 185, 199, 168, 19, 14, 9, 205, 59, 31,
    124, 106, 58, 100, 67, 81, 95, 200, 96, 1, 205, 206, 67, 38, 21, 224, 247,
    75, 1, 131,
];

#[test]
fn exact_split() {
    const PART_SIZE: usize = 20;
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.child(FILE_NAME);
    temp_file
        .write_binary(&TEST_BYTES)
        .expect("Failed to write test bytes to temp file");

    Command::cargo_bin("chop")
        .unwrap()
        .args(&["-n", "5", &temp_file.path().to_string_lossy()])
        .unwrap()
        .assert()
        .success();

    (0..5).into_iter().map(|n| (n + 1, n * PART_SIZE)).for_each(
        |(part_no, part_byte_offset)| {
            let child_path =
                format!("{}.{}{}", FILE_NAME, EXTENSION_PREFIX, part_no);
            let part = temp_dir.child(&child_path);
            part.assert(
                &TEST_BYTES[part_byte_offset..part_byte_offset + PART_SIZE],
            );
        },
    );
}

#[test]
fn split() {
    const NUM_PARTS: usize = 7;
    const PART_SIZE: usize = 15;
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.child(FILE_NAME);
    temp_file
        .write_binary(&TEST_BYTES)
        .expect("Failed to write test bytes to temp file");

    Command::cargo_bin("chop")
        .unwrap()
        .args(&["-s", "15", &temp_file.path().to_string_lossy()])
        .unwrap()
        .assert()
        .success();

    (0..NUM_PARTS)
        .into_iter()
        .map(|n| (n + 1, n * PART_SIZE))
        .for_each(|(part_no, part_byte_offset)| {
            let child_path =
                format!("{}.{}{}", FILE_NAME, EXTENSION_PREFIX, part_no);
            let part = temp_dir.child(&child_path);
            let end_index = min(TEST_BYTES.len(), part_byte_offset + PART_SIZE);
            part.assert(&TEST_BYTES[part_byte_offset..end_index]);
        });
}
