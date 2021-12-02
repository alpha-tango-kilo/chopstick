use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use chopstick::{zero_pad_width, EXTENSION_PREFIX};
use rand::{thread_rng, Rng, RngCore};
use std::cmp::min;
use std::fs;

const FILE_NAME: &str = "chopnplop";
const FIVE_HUNGE_KIB: usize = 500 * 1024;

#[test]
fn num_parts() {
    let mut rng = thread_rng();
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.child(FILE_NAME);
    let mut file_bytes = [0u8; FIVE_HUNGE_KIB];
    rng.fill_bytes(&mut file_bytes);
    temp_file
        .write_binary(&file_bytes)
        .expect("Failed to write test bytes to temp file");

    let num_parts: usize = rng.gen_range(5..=1000);
    println!("Using {} parts", num_parts);
    // Round up division wooo
    let part_size =
        FIVE_HUNGE_KIB / num_parts + (FIVE_HUNGE_KIB % num_parts != 0) as usize;
    let zero_pad_width = zero_pad_width(num_parts as u64);

    // Chop
    Command::cargo_bin("chop")
        .unwrap()
        .args(&[
            "-n",
            &num_parts.to_string(),
            &temp_file.path().to_string_lossy(),
        ])
        .unwrap()
        .assert()
        .success();

    // Check intermediary parts
    (0..num_parts)
        .into_iter()
        .map(|n| (n + 1, n * part_size))
        .for_each(|(part_no, file_bytes_offset)| {
            let child_path = format!(
                "{}.{}{:0>width$}",
                FILE_NAME,
                EXTENSION_PREFIX,
                part_no,
                width = zero_pad_width
            );
            let part = temp_dir.child(&child_path);
            let part_bytes = fs::read(part.path())
                .expect(&format!("Unable to find/read {:?}", &part.path()));
            let end_index =
                min(file_bytes.len(), file_bytes_offset + part_size);
            assert_eq!(
                part_bytes.as_slice(),
                &file_bytes[file_bytes_offset..end_index],
                "File contents differs in part {}",
                part_no,
            );
        });

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
