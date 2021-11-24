use crate::args::RunConfig;
use crate::StickError::CreateOriginal;
pub use error::*;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::{fs, process};

mod args;
mod error;

fn main() {
    if let Err(why) = _main() {
        eprintln!("{}", why);
        process::exit(why.exit_code());
    }
}

fn _main() -> Result<()> {
    let config = RunConfig::new()?;

    let mut original_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&config.original_file)
        .map_err(|err| CreateOriginal(config.original_file.clone(), err))?;
    let mut buffer = Vec::new();

    config
        .part_paths
        .iter()
        .try_for_each(|part_path| -> Result<()> {
            use StickError::*;

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
            fs::remove_file(part_path)
                .map_err(|err| DeletePart(part_path.clone(), err))?;

            Ok(())
        })?;
    Ok(())
}
