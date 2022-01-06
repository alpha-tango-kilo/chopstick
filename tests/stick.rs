use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use chopstick::EXTENSION_PREFIX;
use walkdir::WalkDir;

const FILE_NAME: &str = "stick_me";
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
fn combine() {
    let temp_dir = TempDir::new().unwrap();
    (0..10)
        .into_iter()
        .map(|n| (n + 1, &TEST_BYTES[n * 10..n * 10 + 10]))
        .try_for_each(|(part_no, slice)| {
            let child_path =
                format!("{}.{}{:0>2}", FILE_NAME, EXTENSION_PREFIX, part_no);
            println!("Part {}, bytes {:?}", part_no, slice);
            let part = temp_dir.child(&child_path);
            part.write_binary(slice)
        })
        .expect("Failed to setup test: writing temp file");

    Command::cargo_bin("stick")
        .unwrap()
        .current_dir(&temp_dir)
        .arg(FILE_NAME)
        .unwrap()
        .assert()
        .success();

    let reassembled = temp_dir.child(FILE_NAME);
    reassembled.assert(&TEST_BYTES[..]);
}

#[test]
fn retain() {
    let temp_dir = TempDir::new().unwrap();
    let mut child_paths = Vec::with_capacity(10);
    (0..10)
        .into_iter()
        .map(|n| (n + 1, &TEST_BYTES[n * 10..n * 10 + 10]))
        .for_each(|(part_no, slice)| {
            let child_path =
                format!("{}.{}{:0>2}", FILE_NAME, EXTENSION_PREFIX, part_no);
            let part = temp_dir.child(&child_path);
            part.write_binary(slice).expect("Failed to write part");
            child_paths.push(part);
        });

    Command::cargo_bin("stick")
        .unwrap()
        .current_dir(&temp_dir)
        .args(&["-r", FILE_NAME])
        .unwrap()
        .assert()
        .success();

    child_paths.into_iter().for_each(|part| {
        assert!(part.exists(), "{} not found", part.to_string_lossy())
    });
}

#[test]
fn dont_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    (0..10)
        .into_iter()
        .map(|n| (n + 1, &TEST_BYTES[n * 10..n * 10 + 10]))
        .try_for_each(|(part_no, slice)| {
            let child_path =
                format!("{}.{}{:0>2}", FILE_NAME, EXTENSION_PREFIX, part_no);
            println!("Part {}, bytes {:?}", part_no, slice);
            let part = temp_dir.child(&child_path);
            part.write_binary(slice)
        })
        .expect("Failed to setup test: writing temp file");
    temp_dir
        .child(FILE_NAME)
        .write_binary(&TEST_BYTES[..])
        .expect("Failed to setup test: writing temp file");

    Command::cargo_bin("stick")
        .unwrap()
        .current_dir(&temp_dir)
        .arg(FILE_NAME)
        .assert()
        .failure()
        .code(2);
}

#[test]
fn dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let mut child_paths = Vec::with_capacity(10);
    (0..10)
        .into_iter()
        .map(|n| (n + 1, &TEST_BYTES[n * 10..n * 10 + 10]))
        .for_each(|(part_no, slice)| {
            let child_path =
                format!("{}.{}{:0>2}", FILE_NAME, EXTENSION_PREFIX, part_no);
            let part = temp_dir.child(&child_path);
            part.write_binary(slice).expect("Failed to write part");
            child_paths.push((part, slice));
        });

    let num_files = WalkDir::new(&temp_dir)
        .follow_links(true)
        .min_depth(1)
        .into_iter()
        //.map(|foo| { println!("{:?}", &foo); foo })
        .count();

    assert_eq!(
        num_files,
        child_paths.len(),
        "Should have only been {} files",
        child_paths.len()
    );
    child_paths.into_iter().for_each(|(cp, bytes)| {
        cp.assert(bytes);
    });
}
