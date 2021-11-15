use args::RunConfig;
pub use error::*;

mod args;
mod error;

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
