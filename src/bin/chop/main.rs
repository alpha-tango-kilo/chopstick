use crate::ChopError::*;
use args::RunConfig;
use chopstick::{
    digits, max_buffer_size, sufficient_disk_space, ChunkedReader,
};
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
                eprintln!(
                    "Sufficient disk space available ({} needed)",
                    bytesize::to_string(space_needed, true),
                );
            }
        }
        Ok(false) => return Err(InsufficientDiskSpace),
        Err(warn) => eprintln!("WARNING: {warn}"),
    }

    // Cast is saturating if part_size > usize::MAX
    let buffer_size = min(config.split.part_size, max_buffer_size()) as usize;
    let mut buffer = vec![0; buffer_size];
    if config.verbose {
        eprintln!(
            "Allocated {} buffer",
            bytesize::to_string(buffer_size as u64, true),
        );
    }

    let original_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&config.path)
        .map_err(FailedToReadPart)?;
    let mut reader =
        ChunkedReader::new(original_file, &mut buffer, config.verbose);
    let zero_pad_width = digits(config.split.num_parts) as usize;

    (0..config.split.num_parts)
        .into_iter()
        // Have to make parts backwards because we can only truncate the
        // original file
        .rev()
        .map(|part| {
            let start = part * config.split.part_size;
            let end = min(start + config.split.part_size, file_size);
            let part_path =
                get_part_path_buf(&config.path, part + 1, zero_pad_width);
            (start, end, part_path)
        })
        .try_for_each(|(start, end, part_path)| -> Result<()> {
            let mut part_file = if !config.dry_run {
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
                // Extra newline for some nice spacing
                eprintln!("\nCreated {}", part_path.to_string_lossy());
            }

            if !config.dry_run {
                reader.seek_to(start)?;
                while let Some(bytes) =
                    reader.read_up_to(end - 1).map_err(FailedToReadPart)?
                {
                    part_file.as_mut().unwrap().write_all(bytes).map_err(
                        |err| FailedToWritePart(part_path.clone(), err),
                    )?;
                    if config.verbose {
                        eprintln!("Wrote buffer to part file");
                    }
                }
            } else if config.verbose {
                eprintln!("[reading and writing happens]");
            }

            if !config.retain {
                if !config.dry_run {
                    reader.file.set_len(start).map_err(FailedToTruncate)?;
                }
                if config.verbose {
                    eprintln!(
                        "Truncated original file to {}",
                        bytesize::to_string(start, true),
                    );
                }
            }

            Ok(())
        })?;
    // Extra newline for some nice spacing
    if config.verbose {
        eprintln!();
    }

    // Drop isn't strictly necessary but saves me trying to use it after the
    // file is deleted
    mem::drop(reader);
    mem::drop(buffer);
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
