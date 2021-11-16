use args::RunConfig;
pub use error::*;
pub use lib::*;

mod args;
mod error;
mod lib;

fn main() {
    if let Err(why) = _main() {
        eprintln!("{}", why);
    }
}

fn _main() -> Result<()> {
    let config = RunConfig::new()?;
    println!("{:?}", &config);
    Ok(())
}
