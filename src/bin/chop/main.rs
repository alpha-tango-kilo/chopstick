use args::RunConfig;
pub use error::*;
pub use lib::*;
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
    Ok(())
}
