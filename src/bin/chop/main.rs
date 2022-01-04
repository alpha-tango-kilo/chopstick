use crate::ChopError::*;
use args::RunConfig;
use chopstick::digits;
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

            if config.verbose {
                eprintln!(
                    "Created part file {}",
                    part_path.file_name().unwrap().to_string_lossy()
                );
            }

            // Step 3: read to end of source file into the buffer
            handle.read_to_end(&mut buffer).map_err(FailedToReadPart)?;

            if config.verbose {
                eprintln!("Read {} bytes into buffer", buffer.len());
            }

            // Step 4: write buffer to part file, then clear buffer
            part_file
                .write_all(&buffer)
                .map_err(|err| FailedToWritePart(part_path.clone(), err))?;
            buffer.clear();

            if config.verbose {
                eprintln!("Wrote buffer to part file");
            }

            // Step 5: truncate source file
            if !config.retain {
                handle.set_len(byte_offset).map_err(FailedToTruncate)?;

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
        fs::remove_file(&config.path).map_err(FailedToDeleteOriginal)?;

        if config.verbose {
            eprintln!("Deleted original file");
        }
    }

    if config.verbose {
        eprintln!("Finished without error!");
    }

    Ok(())
}
