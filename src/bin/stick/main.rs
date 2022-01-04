use crate::args::RunConfig;
use crate::StickError::*;
pub use error::*;
use std::fs::OpenOptions;
use std::io::{Read, Write};
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
        fs::rename(&config.part_paths.remove(0), &config.original_file)
            .map_err(|why| CreateOriginal(config.original_file.clone(), why))?;
        OpenOptions::new()
            .append(true)
            .open(&config.original_file)
            .map_err(WriteOriginal)?
    } else {
        // Just create a new file to store the original in
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&config.original_file)
            .map_err(|why| CreateOriginal(config.original_file.clone(), why))?
    };

    config
        .part_paths
        .iter()
        .try_for_each(|part_path| -> Result<()> {
            // Step 1: read part into memory
            let mut part = OpenOptions::new()
                .read(true)
                .open(part_path)
                .map_err(|err| ReadPart(part_path.clone(), err))?;
            part.read_to_end(&mut buffer)
                .map_err(|err| ReadPart(part_path.clone(), err))?;

            // Step 2: write buffer to original file
            original_file.write_all(&buffer).map_err(WriteOriginal)?;

            // Step 3: clear buffer
            buffer.clear();

            // Step 4: delete part file
            if !config.retain {
                fs::remove_file(part_path)
                    .map_err(|err| DeletePart(part_path.clone(), err))?;
            }

            Ok(())
        })?;
    Ok(())
}
