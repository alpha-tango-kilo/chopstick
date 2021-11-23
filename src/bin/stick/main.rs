use crate::args::RunConfig;
pub use error::*;
use std::process;

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
    println!("{:#?}", config);
    Ok(())
}
