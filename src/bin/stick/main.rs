use crate::args::RunConfig;
use crate::StickError::*;
use chopstick::sufficient_disk_space;
pub use error::*;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::{fs, io, process};

mod args;
mod error;

fn main() {
    if let Err(why) = _main() {
        eprintln!("{}", why);
        process::exit(why.exit_code());
    }
}

fn _main() -> Result<()> {
    let mut config = RunConfig::new()?;
    let mut buffer = Vec::new();

    // Disk space check (only applies for retain)
    if config.retain {
        match total_part_size(&config.part_paths) {
            Ok(required_space) => {
                match sufficient_disk_space(&config.original_file, required_space) {
                    Ok(true) => {
                        if config.verbose {
                            eprintln!("Enough disk space available for operation");
                        }
                    }
                    Ok(false) => return Err(InsufficientDiskSpace),
                    Err(warn) => eprintln!("WARNING: {}", warn),
                }
            }
            Err(why) => eprintln!("WARNING: unable to read part file sizes to check if space is available ({})", why),
        }
    }

    let mut original_file = if !config.retain {
        // Check the original file doesn't already exist, so as not to
        // overwrite it if it does
        if config.original_file.exists() {
            return Err(CreateOriginal(
                config.original_file.clone(),
                io::Error::new(io::ErrorKind::AlreadyExists, "The file exists"),
            ));
        }
        // Rename first part to the original file and append to it from there
        // First part is removed from config.part_paths
        let first_part = config.part_paths.remove(0);
        if !config.dry_run {
            fs::rename(&first_part, &config.original_file).map_err(|why| {
                CreateOriginal(config.original_file.clone(), why)
            })?;
        }
        if config.verbose {
            eprintln!(
                "Renamed {} to {}",
                first_part.to_string_lossy(),
                config.original_file.to_string_lossy(),
            );
        }

        if !config.dry_run {
            OpenOptions::new()
                .append(true)
                .open(&config.original_file)
                .map_err(WriteOriginal)?
                .into()
        } else {
            None
        }
    } else {
        let of = if !config.dry_run {
            // Just create a new file to store the original in
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&config.original_file)
                .map_err(|why| {
                    CreateOriginal(config.original_file.clone(), why)
                })?
                .into()
        } else {
            None
        };
        if config.verbose {
            eprintln!(
                "Created empty file {}",
                config.original_file.to_string_lossy()
            );
        }
        of
    };

    config
        .part_paths
        .iter()
        .try_for_each(|part_path| -> Result<()> {
            // Step 1: read part into memory
            if !config.dry_run {
                let mut part = OpenOptions::new()
                    .read(true)
                    .open(part_path)
                    .map_err(|err| ReadPart(part_path.clone(), err))?;
                part.read_to_end(&mut buffer)
                    .map_err(|err| ReadPart(part_path.clone(), err))?;
            }
            if config.verbose {
                eprintln!("Read {} into buffer", part_path.to_string_lossy());
            }

            // Step 2: write buffer to original file
            if !config.dry_run {
                original_file
                    .as_mut()
                    .unwrap()
                    .write_all(&buffer)
                    .map_err(WriteOriginal)?;
            }
            if config.verbose {
                eprintln!("Appended buffer to original file");
            }

            // Step 3: clear buffer
            buffer.clear();

            // Step 4: delete part file
            if !config.retain {
                if !config.dry_run {
                    fs::remove_file(part_path)
                        .map_err(|err| DeletePart(part_path.clone(), err))?;
                }
                if config.verbose {
                    eprintln!("Deleted {}", part_path.to_string_lossy());
                }
            }

            Ok(())
        })?;

    if config.verbose && !config.dry_run {
        eprintln!("Finished without error!");
    }

    Ok(())
}

fn total_part_size<P: AsRef<Path>>(paths: &[P]) -> Result<u64, io::Error> {
    // All parts are the same size but the last one, so just multiply the size
    // of the first part by paths.len() - 1, then add the size of the last part
    debug_assert!(paths.len() >= 2);
    let first = fs::metadata(&paths[0])?.len();
    let last = fs::metadata(paths.last().unwrap())?.len();
    Ok(first * (paths.len() as u64 - 1) + last)
}

#[cfg(test)]
mod test {
    use super::total_part_size;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    #[test]
    fn required_size_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let file_one = temp_dir.child("one");
        file_one.write_binary(&[12, 45, 51, 12, 34]).unwrap();
        let file_two = temp_dir.child("two");
        file_two.write_binary(&[32, 34, 22, 34, 11]).unwrap();
        let file_three = temp_dir.child("three");
        file_three.write_binary(&[4, 120, 54]).unwrap();
        let paths = vec![file_one.path(), file_two.path(), file_three.path()];
        assert_eq!(total_part_size(&paths).unwrap(), 13);
    }
}
