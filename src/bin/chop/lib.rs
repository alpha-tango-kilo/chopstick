use crate::{ChopError, Result};
use std::path::{Path, PathBuf};

const EXTENSION_PREFIX: &str = "p";

#[derive(Debug, Copy, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Split {
    pub part_size: u64,
    pub num_parts: u64,
}

impl Split {
    pub const fn from_part_size(
        file_size: u64,
        part_size: u64,
    ) -> Result<Self> {
        if part_size >= file_size {
            Err(ChopError::PartSizeTooLarge)
        } else {
            Ok(Split {
                part_size,
                num_parts: round_up_div(file_size, part_size),
            })
        }
    }

    pub const fn from_num_parts(
        file_size: u64,
        num_parts: u64,
    ) -> Result<Self> {
        if num_parts >= file_size {
            Err(ChopError::NumPartsTooLarge)
        } else {
            Ok(Split {
                part_size: round_up_div(file_size, num_parts),
                num_parts,
            })
        }
    }
}

pub fn get_part_path_buf<P: AsRef<Path>>(
    original_path: P,
    index: u64,
) -> PathBuf {
    let mut os_str = original_path.as_ref().as_os_str().to_owned();
    os_str.push(format!(".{}{}", EXTENSION_PREFIX, index));
    PathBuf::from(os_str)
}

pub const fn round_up_div(a: u64, b: u64) -> u64 {
    a / b + (a % b != 0) as u64
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    const PART_SIZE_DATA: [(u64, u64, u64); 5] = [
        (1000, 50, 20),
        (1024, 50, 21),
        (12, 5, 3),
        (603, 50, 13),
        (156, 99, 2),
    ];

    const NUM_PARTS_DATA: [(u64, u64, u64); 5] = [
        (1000, 20, 50),
        (1024, 21, 49),
        (12, 3, 4),
        (603, 13, 47),
        (156, 2, 78),
    ];

    #[test]
    fn round_up_division() {
        assert_eq!(round_up_div(1, 2), 1);
        assert_eq!(round_up_div(7, 2), 4);
        assert_eq!(round_up_div(10, 3), 4);
        assert_eq!(round_up_div(76, 2), 38);
        assert_eq!(round_up_div(16, 7), 3);
        assert_eq!(round_up_div(7, 3), 3);
        assert_eq!(round_up_div(10, 20), 1);
    }

    #[test]
    fn split_from_part_size() {
        PART_SIZE_DATA.into_iter().for_each(
            |(file_size, part_size, num_parts)| {
                let split = Split::from_part_size(file_size, part_size)
                    .expect("Unexpected error");
                assert_eq!(
                    split,
                    Split {
                        part_size,
                        num_parts
                    },
                    "Split calculation mismatch for file size {}",
                    file_size,
                );
            },
        );
    }

    #[test]
    fn split_from_part_size_err() {
        let err = Split::from_part_size(10, 10).unwrap_err();
        assert!(matches!(err, ChopError::PartSizeTooLarge));
        let err = Split::from_part_size(10, 100).unwrap_err();
        assert!(matches!(err, ChopError::PartSizeTooLarge));
    }

    #[test]
    fn split_from_num_parts() {
        NUM_PARTS_DATA.into_iter().for_each(
            |(file_size, num_parts, part_size)| {
                let split = Split::from_num_parts(file_size, num_parts)
                    .expect("Unexpected error");
                assert_eq!(
                    split,
                    Split {
                        part_size,
                        num_parts
                    },
                    "Split calculation mismatch for file size {}",
                    file_size,
                );
            },
        );
    }

    #[test]
    fn split_from_num_parts_err() {
        let err = Split::from_num_parts(10, 10).unwrap_err();
        assert!(matches!(err, ChopError::NumPartsTooLarge));
        let err = Split::from_num_parts(10, 100).unwrap_err();
        assert!(matches!(err, ChopError::NumPartsTooLarge));
    }
}