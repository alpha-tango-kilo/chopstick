use std::cmp::min;
use std::path::Path;
use sysinfo::{DiskExt, System, SystemExt};

pub const EXTENSION_PREFIX: &str = "p";
const DEFAULT_MAX_BUFFER_SIZE: u64 = 512 * 1024 * 1024; // 512 MiB

pub const fn digits(num: u64) -> usize {
    if num < 10 {
        1
    } else if num < 100 {
        2
    } else if num < 1000 {
        3
    } else {
        let mut count = 4;
        let mut current = num / 10u64.pow(4);
        while current > 0 {
            count += 1;
            current /= 10;
        }
        count
    }
}

pub const fn round_up_div(a: u64, b: u64) -> u64 {
    a / b + (a % b != 0) as u64
}

//noinspection RsRedundantElse
pub fn sufficient_disk_space(
    directory: &Path,
    space_needed: u64,
) -> Result<bool, &'static str> {
    if System::IS_SUPPORTED {
        let directory = directory
            .canonicalize()
            .map_err(|_| "couldn't canonicalise file path")?;
        let mut err = "unable to determine disk being used to check space";
        let mut system = System::new();
        system.refresh_disks_list();

        system
            .disks()
            .iter()
            .find(|disk| match disk.mount_point().canonicalize() {
                Ok(disk_path) => directory.starts_with(&disk_path),
                Err(_) => {
                    err = "unable to check space in appropriate disk";
                    false
                }
            })
            .map(|disk| disk.available_space() > space_needed)
            .ok_or(err)
    } else {
        Err("unable to check if there is enough free disk space for this operation")
    }
}

pub fn max_buffer_size() -> u64 {
    if System::IS_SUPPORTED {
        let mut system = System::new();
        system.refresh_memory();
        let total = system.total_memory() * 1000;
        let available = system.available_memory() * 1000;
        // Use at most either an eighth of total memory, or half the available,
        // whichever's smallest
        min(total / 8, available / 2)
    } else {
        DEFAULT_MAX_BUFFER_SIZE
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::*;

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
    fn digit_counting() {
        let input = vec![
            (1, 1),
            (50, 2),
            (12, 2),
            (9, 1),
            (123, 3),
            (41231, 5),
            (1234, 4),
            (123123, 6),
            (1234567890, 10),
            (u64::MAX, 20),
        ];
        input
            .into_iter()
            .for_each(|(n, d)| assert_eq!(digits(n), d));
    }
}
