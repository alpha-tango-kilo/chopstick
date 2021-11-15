use crate::ChopError::*;
use crate::*;
use bytesize::ByteSize;
use clap::{Arg, ArgMatches};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub struct RunConfig {
    pub path: PathBuf,
    pub part_size: u64,
    pub num_parts: u64,
}

impl RunConfig {
    pub fn new() -> Result<Self> {
        let matches = RunConfig::create_clap_app().get_matches();
        RunConfig::process_matches(&matches)
    }

    fn create_clap_app() -> clap::App<'static> {
        clap::App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author("alpha-tango-kilo <git@heyatk.com>")
            .about("Separate files into chunks quickly")
            .arg(
                Arg::new("part_size")
                    .short('s')
                    .long("size")
                    .about("The maximum size each part should be")
                    .long_about(
                        "The maximum size each part should be.\
                        Accepts units - e.g. 1GB, 20K, 128MiB",
                    )
                    .required_unless_present("num_parts")
                    .conflicts_with("num_parts")
                    .takes_value(true),
            )
            .arg(
                Arg::new("num_parts")
                    .short('n')
                    .long("parts")
                    .about("The number of parts to chop the file into")
                    .required_unless_present("part_size")
                    .conflicts_with("part_size")
                    .takes_value(true),
            )
            .arg(
                Arg::new("file")
                    .about("The file to split")
                    .required(true)
                    .takes_value(true),
            )
    }

    // TODO: separate out file size calculation and parsing for better testing
    fn process_matches(clap_matches: &ArgMatches) -> Result<Self> {
        let path: PathBuf = clap_matches.value_of_os("file").unwrap().into();
        let file_size = fs::metadata(&path)?.len();

        let num_parts;
        let part_size;
        if let Some(part_size_str) = clap_matches.value_of("part_size") {
            let ByteSize(part_size_bytes) = ByteSize::from_str(part_size_str)?;
            if part_size_bytes >= file_size {
                return Err(PartSizeTooLarge);
            }
            num_parts = file_size / part_size_bytes + 1;
            part_size = part_size_bytes;
        } else if let Some(num_parts_str) = clap_matches.value_of("num_parts") {
            num_parts = num_parts_str.parse().map_err(|_| InvalidNumParts)?;
            if num_parts >= file_size {
                return Err(NumPartsTooLarge);
            }
            part_size = round_up_div(file_size, num_parts);
        } else {
            unreachable!("Either num_parts or part_size should have been specified");
        }

        Ok(RunConfig {
            path,
            part_size,
            num_parts,
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use super::RunConfig;

    // TODO: part size calculations

    #[test]
    fn requires_file() {
        let clap = RunConfig::create_clap_app();
        let err = clap
            .try_get_matches_from(vec![env!("CARGO_PKG_NAME"), "-n", "5"])
            .unwrap_err();
        assert_eq!(err.kind, clap::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn must_give_part_size_or_num_parts() {
        // Neither
        let clap = RunConfig::create_clap_app();
        let err = clap
            .try_get_matches_from(vec![env!("CARGO_PKG_NAME"), "Cargo.toml"])
            .unwrap_err();
        assert_eq!(err.kind, clap::ErrorKind::MissingRequiredArgument);

        // One
        let clap = RunConfig::create_clap_app();
        let matches = clap
            .try_get_matches_from(vec![env!("CARGO_PKG_NAME"), "-n", "5", "Cargo.toml"])
            .unwrap();
        assert!(matches.is_present("num_parts"));
        assert!(RunConfig::process_matches(&matches).is_ok());

        let clap = RunConfig::create_clap_app();
        let matches = clap
            .try_get_matches_from(vec![env!("CARGO_PKG_NAME"), "-s", "512", "Cargo.toml"])
            .unwrap();
        assert!(matches.is_present("part_size"));
        assert!(RunConfig::process_matches(&matches).is_ok());

        // Both
        let clap = RunConfig::create_clap_app();
        let err = clap
            .try_get_matches_from(vec![
                env!("CARGO_PKG_NAME"),
                "-n",
                "5",
                "-s",
                "512",
                "Cargo.toml",
            ])
            .unwrap_err();
        assert_eq!(err.kind, clap::ErrorKind::ArgumentConflict);
    }
}
