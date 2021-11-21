use args::RunConfig;
pub use error::*;
pub use lib::*;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process;

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
    println!("{:?}", &config);

    // Cast is saturating if part_size > usize::MAX
    let mut buffer = Vec::with_capacity(config.split.part_size as usize);
    let mut handle = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&config.path)?;

    let cut_off = config.split.part_size * (config.split.num_parts - 1);

    // Go to one part from the end
    handle
        .seek(SeekFrom::Start(cut_off))
        .expect("Arithmetic error, seek outside of file");
    // Read that part
    handle.read_to_end(&mut buffer)?;

    let part_path = get_part_path_buf(&config.path, config.split.num_parts);
    // TODO: Specific error variant for file already existing
    let mut part = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&part_path)?;
    part.write_all(&buffer)?;

    handle.set_len(cut_off)?;

    Ok(())
}
