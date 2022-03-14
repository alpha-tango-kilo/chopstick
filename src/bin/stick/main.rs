use crate::args::RunConfig;
use crate::StickError::*;
use chopstick::{max_buffer_size, sufficient_disk_space};
pub use error::*;
use std::cmp::min;
use std::fs::{File, OpenOptions};
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
    let buffer_size = min(config.part_size, max_buffer_size()) as usize;
    // Buffer must be filled in order to be used in a PartialReader
    let mut buffer: Vec<u8> = vec![0; buffer_size];

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
            // TODO: verbosity & dry runs
            // Step 1: read & write in chunks, controlled by PartialReader
            let mut reader = PartialReader::new(part_path, &mut buffer)
                .map_err(|err| ReadPart(part_path.clone(), err))?;
            while let Some(bytes) = reader
                .read()
                .map_err(|err| ReadPart(part_path.clone(), err))?
            {
                original_file
                    .as_mut()
                    .unwrap()
                    .write_all(bytes)
                    .map_err(WriteOriginal)?;
            }

            // Step 2: delete part file
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

struct PartialReader<'a> {
    file: File,
    buffer: &'a mut Vec<u8>,
    file_size: usize,
    bytes_read: usize,
}

impl<'a> PartialReader<'a> {
    fn new<P: AsRef<Path>>(path: P, buffer: &'a mut Vec<u8>) -> io::Result<Self> {
        debug_assert_eq!(
            buffer.len(),
            buffer.capacity(),
            "PartialReader's buffer must come initialised",
        );
        let file = File::open(path)?;
        let file_size = file.metadata()?.len() as usize;
        Ok(PartialReader {
            file,
            buffer,
            file_size,
            bytes_read: 0,
        })
    }

    fn read(&mut self) -> io::Result<Option<&[u8]>> {
        if self.done() {
            return Ok(None);
        } else if self.buffer.capacity() > self.file_size - self.bytes_read {
            // Clear the buffer when we're going to use read_to_end as it's
            // Vec-aware
            self.buffer.clear();
            // Going to reach end of file, just read it all
            self.bytes_read += self.file.read_to_end(self.buffer)?;
        } else {
            // Assumes the buffer to be full. Read isn't Vec-aware so will only
            // read as many bytes as are already in the Vec. This is why we
            // ensure the buffer provided comes filled
            debug_assert_eq!(
                self.buffer.len(),
                self.buffer.capacity(),
                "Buffer should have data in at this point",
            );
            self.bytes_read += self.file.read(self.buffer)?;
        }
        Ok(Some(self.buffer))
    }

    fn done(&self) -> bool {
        self.bytes_read == self.file_size
    }
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
