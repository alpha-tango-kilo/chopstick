use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use chopstick::{round_up_div, zero_pad_width, EXTENSION_PREFIX};
use rand::{thread_rng, Rng, RngCore};
use std::cmp::min;
use std::fs;
use Method::*;

const FILE_NAME: &str = "chopnplop";
const FIVE_HUNGE_KIB: usize = 500 * 1024;
//const ONE_HUNGE_MIB: usize = 100 * 1024 * 1024;
//const FIVE_GIB: u64 = 5 * 1024 * 1024 * 1024;

#[derive(Copy, Clone)]
enum Method {
    NumParts(u64),
    PartSize(u64),
}

impl Method {
    const fn val(self) -> u64 {
        match self {
            NumParts(n) | PartSize(n) => n,
        }
    }

    const fn flag(&self) -> &str {
        match self {
            NumParts(_) => "-n",
            PartSize(_) => "-s",
        }
    }

    const fn num_parts(self, file_size: usize) -> u64 {
        match self {
            NumParts(n) => n,
            PartSize(_) => self.other(file_size),
        }
    }

    const fn part_size(self, file_size: usize) -> u64 {
        match self {
            NumParts(_) => self.other(file_size),
            PartSize(n) => n,
        }
    }

    const fn other(&self, file_size: usize) -> u64 {
        round_up_div(file_size as u64, self.val())
    }

    const fn zero_pad_width(&self, file_size: usize) -> usize {
        match self {
            NumParts(n) => zero_pad_width(*n),
            PartSize(_) => zero_pad_width(self.other(file_size)),
        }
    }
}

struct TestScenario<const N: usize> {
    temp_dir: TempDir,
    original_file: ChildPath,
    file_bytes: [u8; N],
}

impl<const N: usize> TestScenario<N> {
    fn new() -> Self {
        let mut rng = thread_rng();
        let temp_dir = TempDir::new().unwrap();
        let original_file = temp_dir.child(FILE_NAME);
        let mut file_bytes = [0u8; N];
        rng.fill_bytes(&mut file_bytes);
        original_file
            .write_binary(&file_bytes)
            .expect("Failed to write test bytes to temp file");
        assert!(N > 1000, "N must be over 1000");

        TestScenario {
            temp_dir,
            original_file,
            file_bytes,
        }
    }

    fn run_with(&self, method: Method) {
        // Chop
        Command::cargo_bin("chop")
            .unwrap()
            .args(&[
                method.flag(),
                // TODO: format as bytes if appropriate
                &method.val().to_string(),
                &self.original_file.path().to_string_lossy(),
            ])
            .unwrap()
            .assert()
            .success();

        // Check intermediary parts
        (0..method.num_parts(N))
            .into_iter()
            .map(|n| (n + 1, (n * method.part_size(N)) as usize))
            .for_each(|(part_no, file_bytes_offset)| {
                let child_path = format!(
                    "{}.{}{:0>width$}",
                    FILE_NAME,
                    EXTENSION_PREFIX,
                    part_no,
                    width = method.zero_pad_width(N),
                );
                let part = self.temp_dir.child(&child_path);
                let part_bytes = fs::read(part.path()).unwrap_or_else(|_| {
                    panic!("Unable to find/read {:?}", &part.path())
                });
                let end_index = min(
                    self.file_bytes.len(),
                    file_bytes_offset + method.part_size(N) as usize,
                );
                assert_eq!(
                    part_bytes.as_slice(),
                    &self.file_bytes[file_bytes_offset..end_index],
                    "File contents differs in part {}",
                    part_no,
                );
            });

        // Stick
        Command::cargo_bin("stick")
            .unwrap()
            .current_dir(self.temp_dir.path())
            .arg(FILE_NAME)
            .unwrap()
            .assert()
            .success();

        // Test
        let reconstructed_bytes = fs::read(self.original_file.path())
            .expect("Unable to find/read reconstructed file");
        assert_eq!(
            reconstructed_bytes.as_slice(),
            &self.file_bytes[..],
            "File contents differs",
        );
    }
}

#[test]
fn num_parts() {
    let test = TestScenario::<FIVE_HUNGE_KIB>::new();
    test.run_with(NumParts(thread_rng().gen_range(10..=1000)));
}

#[test]
fn part_size() {
    let test = TestScenario::<FIVE_HUNGE_KIB>::new();
    test.run_with(PartSize(thread_rng().gen_range(10..=50 * 1024)));
}

// TODO: large files, relative directories
