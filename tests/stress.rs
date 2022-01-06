use assert_cmd::prelude::*;
use assert_cmd::Command;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use bytesize::ByteSize;
use chopstick::{digits, round_up_div, EXTENSION_PREFIX};
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::cmp::min;
use std::fs;

const FILE_NAME: &str = "chopnplop";
const FIVE_HUNGE_KIB: usize = 500 * 1024;
//const ONE_HUNGE_MIB: usize = 100 * 1024 * 1024;
//const FIVE_GIB: u64 = 5 * 1024 * 1024 * 1024;

// Dangerous amounts of code re-use from chop/lib.rs
struct Split {
    part_size: u64,
    num_parts: u64,
    flag: &'static str,
}

impl Split {
    fn from_part_size(file_size: u64, part_size: u64) -> Self {
        if part_size >= file_size {
            panic!("Part size greater than file size")
        } else {
            let (part_size, num_parts) =
                Split::closest_factors_to(file_size, part_size);
            Split {
                part_size,
                num_parts,
                flag: "-s",
            }
        }
    }

    fn from_num_parts(file_size: u64, num_parts: u64) -> Self {
        if num_parts >= file_size {
            panic!("Number of parts greater than file size")
        } else {
            let (num_parts, part_size) =
                Split::closest_factors_to(file_size, num_parts);
            Split {
                part_size,
                num_parts,
                flag: "-n",
            }
        }
    }

    const fn closest_factors_to(target: u64, divisor: u64) -> (u64, u64) {
        let factor_two = round_up_div(target, divisor);
        /*
        In the edge case where the user's choice (`divisor`) would result in
        an empty part (see test `disobey` below), the expression
        `(divisor - 1).saturating_sub(target / factor_two)`
        will be >0, thus disregarding their choice to reduce the error in
        reproducing the `target`
        */
        let factor_one =
            divisor - (divisor - 1).saturating_sub(target / factor_two);
        (factor_one, factor_two)
    }

    fn flag_val(&self) -> String {
        match self.flag {
            "-s" => ByteSize(self.part_size).to_string(),
            "-n" => self.num_parts.to_string(),
            _ => unreachable!(),
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
        Self::new_with_rng(&mut thread_rng())
    }

    fn new_with_rng<R: RngCore>(rng: &mut R) -> Self {
        let temp_dir = TempDir::new().unwrap();
        println!("Using temp dir {}", temp_dir.to_string_lossy());
        let original_file = temp_dir.child(FILE_NAME);
        let mut file_bytes = [0u8; N];
        rng.fill_bytes(&mut file_bytes);
        original_file
            .write_binary(&file_bytes)
            .expect("Failed to write test bytes to temp file");

        TestScenario {
            temp_dir,
            original_file,
            file_bytes,
        }
    }

    fn run_with(&self, split: Split) {
        println!(
            "Chopping {} byte file into {} parts, {}B each",
            N, split.num_parts, split.part_size,
        );
        // Chop
        Command::cargo_bin("chop")
            .unwrap()
            .args(&[
                split.flag,
                &split.flag_val(),
                &self.original_file.path().to_string_lossy(),
            ])
            .unwrap()
            .assert()
            .success();
        println!("Ran chop");

        // Check intermediary parts
        (0..split.num_parts)
            .into_iter()
            .map(|n| (n + 1, (n * split.part_size) as usize))
            .for_each(|(part_no, file_bytes_offset)| {
                let child_path = format!(
                    "{}.{}{:0>width$}",
                    FILE_NAME,
                    EXTENSION_PREFIX,
                    part_no,
                    width = digits(split.num_parts),
                );
                let part = self.temp_dir.child(&child_path);
                let part_bytes = fs::read(part.path()).unwrap_or_else(|_| {
                    panic!("Unable to find/read {:?}", &part.path())
                });
                let end_index = min(
                    self.file_bytes.len(),
                    file_bytes_offset + split.part_size as usize,
                );
                assert_eq!(
                    part_bytes.as_slice(),
                    &self.file_bytes[file_bytes_offset..end_index],
                    "File contents differs in part {} of {}",
                    part_no,
                    split.num_parts,
                );
            });
        println!("All intermediary parts are as expected");

        // Stick
        Command::cargo_bin("stick")
            .unwrap()
            .current_dir(self.temp_dir.path())
            .arg(FILE_NAME)
            .unwrap()
            .assert()
            .success();
        println!("Ran stick");

        // Test
        let reconstructed_bytes = fs::read(self.original_file.path())
            .expect("Unable to find/read reconstructed file");
        assert_eq!(
            reconstructed_bytes.as_slice(),
            &self.file_bytes[..],
            "File contents differs",
        );
        println!("Reassembled file as expected");
    }
}

#[test]
fn num_parts() {
    let test = TestScenario::<FIVE_HUNGE_KIB>::new();
    test.run_with(Split::from_num_parts(
        FIVE_HUNGE_KIB as u64,
        thread_rng().gen_range(10..=1000),
    ));
}

#[test]
fn part_size() {
    let test = TestScenario::<FIVE_HUNGE_KIB>::new();
    test.run_with(Split::from_part_size(
        FIVE_HUNGE_KIB as u64,
        thread_rng().gen_range(10..=50 * 1024),
    ));
}

#[test]
#[ignore]
fn something_specific() {
    // https://rust-random.github.io/book/guide-seeding.html#a-simple-number
    let mut fixed_seed = Pcg64::seed_from_u64(14);
    let test = TestScenario::<FIVE_HUNGE_KIB>::new_with_rng(&mut fixed_seed);
    test.run_with(Split::from_num_parts(FIVE_HUNGE_KIB as u64, 986));
}

// TODO: large files, relative directories
