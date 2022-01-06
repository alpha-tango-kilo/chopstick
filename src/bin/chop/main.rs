use crate::ChopError::*;
use args::RunConfig;
use chopstick::{digits, sufficient_disk_space};
pub use error::*;
pub use lib::*;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
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

    // Check if there is sufficient disk space available
    let space_needed = if !config.retain {
        config.split.part_size
    } else {
        fs::metadata(&config.path)?.len()
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
    let mut buffer = Vec::with_capacity(config.split.part_size as usize);
    let mut handle = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&config.path)?;
    let zero_pad_width = digits(config.split.num_parts) as usize;

    if config.verbose {
        eprintln!("File opened and buffer created");
    }

    (0..config.split.num_parts)
        .into_iter()
        .rev()
        .map(|part| {
            (
                part * config.split.part_size,
                get_part_path_buf(&config.path, part + 1, zero_pad_width),
            )
        })
        .try_for_each(|(byte_offset, part_path)| -> Result<()> {
            // Step 1: Get the source file handle pointed at the right place
            handle
                .seek(SeekFrom::Start(byte_offset))
                .expect("Arithmetic error, seek outside of file");

            // Step 2: Create part file
            let part_file = if !config.dry_run {
                OpenOptions::new()
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
                    })?
                    .into()
            } else {
                None
            };

            if config.verbose {
                eprintln!(
                    "Created part file {}",
                    part_path.file_name().unwrap().to_string_lossy()
                );
            }

            // Step 3: read to end of source file into the buffer
            if !config.dry_run {
                handle.read_to_end(&mut buffer).map_err(FailedToReadPart)?;
            }
            if config.verbose {
                eprintln!("Read {} bytes into buffer", buffer.len());
            }

            // Step 4: write buffer to part file, then clear buffer
            if !config.dry_run {
                part_file
                    .unwrap()
                    .write_all(&buffer)
                    .map_err(|err| FailedToWritePart(part_path.clone(), err))?;
                buffer.clear();
            }
            if config.verbose {
                eprintln!("Wrote buffer to part file");
            }

            // Step 5: truncate source file
            if !config.retain {
                if !config.dry_run {
                    handle.set_len(byte_offset).map_err(FailedToTruncate)?;
                }
                if config.verbose {
                    eprintln!("Truncated original file");
                }
            }

            Ok(())
        })?;

    // Drop isn't strictly necessary but saves me trying to use it on a
    // soon-to-be deleted file
    mem::drop(handle);
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
