use crate::ChopError::*;
use args::RunConfig;
use chopstick::{digits, max_buffer_size, sufficient_disk_space, ChunkedReader};
pub use error::*;
pub use lib::*;
use std::cmp::min;
use std::fs::OpenOptions;
use std::io::Write;
use std::{fs, mem, process};

mod args;
mod error;
mod lib;

fn main() {
    if let Err(why) = _main() {
        eprintln!("{}", why);
        process::exit(why.exit_code());
    }
}

fn _main() -> Result<()> {
    let config = RunConfig::new()?;
    let file_size = fs::metadata(&config.path)?.len();

    // Check if there is sufficient disk space available
    let space_needed = if !config.retain {
        config.split.part_size
    } else {
        file_size
    };
    match sufficient_disk_space(&config.path, space_needed) {
        Ok(true) => {
            if config.verbose {
                eprintln!("Enough disk space available for operation");
            }
        }
        Ok(false) => return Err(InsufficientDiskSpace),
        Err(warn) => eprintln!("WARNING: {}", warn),
    }

    // Cast is saturating if part_size > usize::MAX
    let buffer_size = min(config.split.part_size, max_buffer_size()) as usize;
    if config.verbose {
        eprintln!(
            "Chose buffer size of {}",
            bytesize::to_string(buffer_size as u64, true),
        );
    }

    let original_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&config.path)
        .map_err(FailedToReadPart)?;
    let mut reader = ChunkedReader::new(original_file, buffer_size);
    let zero_pad_width = digits(config.split.num_parts) as usize;

    if config.verbose {
        eprintln!("File opened and buffer created");
    }

    (0..config.split.num_parts)
        .into_iter()
        // Have to make parts backwards because we can only truncate the
        // original file
        .rev()
        .map(|part| {
            let start = part * config.split.part_size;
            let part_path =
                get_part_path_buf(&config.path, part + 1, zero_pad_width);
            (start, part_path)
        })
        .try_for_each(|(start, part_path)| -> Result<()> {
            // TODO: verbose & dry-run
            let mut part_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&part_path)
                .map_err(|err| {
                    use std::io::ErrorKind::*;
                    match err.kind() {
                        AlreadyExists => {
                            PartFileAlreadyExists(part_path.clone())
                        }
                        _ => err.into(),
                    }
                })?;

            reader.seek_to(start)?;
            while let Some(bytes) = reader.read().map_err(FailedToReadPart)? {
                part_file
                    .write_all(bytes)
                    .map_err(|err| FailedToWritePart(part_path.clone(), err))?;
            }

            if !config.retain {
                reader.file.set_len(start).map_err(FailedToTruncate)?;
            }
            Ok(())
        })?;

    // Drop isn't strictly necessary but saves me trying to use it after the
    // file is deleted
    mem::drop(reader);
    if !config.retain {
        if !config.dry_run {
            fs::remove_file(&config.path).map_err(FailedToDeleteOriginal)?;
        }
        if config.verbose {
            eprintln!("Deleted original file");
        }
    }

    if config.verbose && !config.dry_run {
        eprintln!("Finished without error!");
    }

    Ok(())
}
