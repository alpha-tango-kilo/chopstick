use crate::{ChopError, Result};
use chopstick::{round_up_div, EXTENSION_PREFIX};
use std::path::{Path, PathBuf};

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
            let (part_size, num_parts) =
                Split::closest_factors_to(file_size, part_size);
            Ok(Split {
                part_size,
                num_parts,
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
            let (num_parts, part_size) =
                Split::closest_factors_to(file_size, num_parts);
            Ok(Split {
                part_size,
                num_parts,
            })
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
}

pub fn get_part_path_buf<P: AsRef<Path>>(
    original_path: P,
    index: u64,
    width: usize,
) -> PathBuf {
    let mut os_str = original_path.as_ref().as_os_str().to_owned();
    os_str.push(format!(
        ".{}{:0width$}",
        EXTENSION_PREFIX,
        index,
        width = width
    ));
    PathBuf::from(os_str)
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    const PART_SIZE_DATA: [(u64, u64, u64); 5] = [
        (1000, 50, 20),
        (1024, 49, 21),
        (12, 5, 3),
        (603, 47, 13),
        (156, 79, 2),
    ];

    const NUM_PARTS_DATA: [(u64, u64, u64); 5] = [
        (1000, 20, 50),
        (1024, 21, 49),
        (12, 3, 4),
        (603, 13, 47),
        (156, 2, 78),
    ];

    #[test]
    fn split_from_part_size() {
        PART_SIZE_DATA.into_iter().for_each(
            |(file_size, part_size, num_parts)| {
                let split =
                    Split::from_part_size(file_size, part_size).unwrap();
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

    #[test]
    fn closest_factors() {
        assert_eq!(Split::closest_factors_to(512000, 986), (985, 520));
        assert_eq!(Split::closest_factors_to(1024, 50), (49, 21));
    }

    #[test]
    fn disobey() {
        let split = Split::from_num_parts(512000, 986).unwrap();
        assert_eq!(
            split,
            Split {
                part_size: 520,
                num_parts: 985, // We did it, we disobeyed!
            },
        );

        let split = Split::from_part_size(1024, 50).unwrap();
        assert_eq!(
            split,
            Split {
                part_size: 49, // We did it, we disobeyed!
                num_parts: 21,
            },
        );
    }
}
