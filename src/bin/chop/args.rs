use crate::*;
use bytesize::ByteSize;
use clap::{Arg, ArgGroup, ArgMatches};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub struct RunConfig {
    pub path: PathBuf,
    pub split: Split,
    pub retain: bool,
    pub verbose: bool,
    pub dry_run: bool,
}

impl RunConfig {
    pub fn new() -> Result<Self> {
        let matches = RunConfig::create_clap_app().get_matches();
        RunConfig::process_matches(&matches)
    }

    fn create_clap_app() -> clap::Command<'static> {
        clap::Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author("alpha-tango-kilo <git@heyatk.com>")
            .about("Separate files into parts quickly")
            .arg(
                Arg::new("part_size")
                    .short('s')
                    .long("size")
                    .help("The maximum size each part should be")
                    .long_help(
                        "The maximum size each part should be. \
                        Accepts units - e.g. 1GB, 20K, 128MiB. \
                        The last part may be smaller than the others",
                    )
                    .takes_value(true),
            )
            .arg(
                Arg::new("num_parts")
                    .short('n')
                    .long("parts")
                    .help("The number of parts to chop the file into")
                    .long_help(
                        "The number of parts to chop the file into. \
                        Parts will all be the same size (except the last one potentially)",
                    )
                    .takes_value(true),
            )
            .arg(
                Arg::new("retain")
                    .short('r')
                    .long("retain")
                    .visible_aliases(&["no-delete", "preserve"])
                    .help("Don't delete the original file")
                    .long_help("Don't delete the original file (requires more disk space)"),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Makes chop tell you what it's doing"),
            )
            .arg(
                Arg::new("dry-run")
                    .long("dry-run")
                    .visible_alias("dry")
                    .help("Don't actually do anything, just tell me about it")
                    .long_help("Don't actually do anything, just tell me about it (implies --verbose)"),
            )
            .arg(
                Arg::new("file")
                    .help("The file to split")
                    .required(true)
                    .takes_value(true)
                    .allow_invalid_utf8(true),
            )
            .group(
                ArgGroup::new("require_exactly_one")
                    .args(&["part_size", "num_parts"])
                    .required(true),
            )
    }

    fn process_matches(clap_matches: &ArgMatches) -> Result<Self> {
        let path: PathBuf = clap_matches.value_of_os("file").unwrap().into();
        let file_size = fs::metadata(&path)?.len();

        let split = if let Some(part_size_str) =
            clap_matches.value_of("part_size")
        {
            let ByteSize(part_size) = ByteSize::from_str(part_size_str)?;
            Split::from_part_size(file_size, part_size)?
        } else if let Some(num_parts_str) = clap_matches.value_of("num_parts") {
            let num_parts =
                num_parts_str.parse().map_err(|_| InvalidNumParts)?;
            Split::from_num_parts(file_size, num_parts)?
        } else {
            unreachable!(
                "Either num_parts or part_size should have been specified"
            );
        };

        let retain = clap_matches.is_present("retain");
        let dry_run = clap_matches.is_present("dry-run");
        let verbose = dry_run || clap_matches.is_present("verbose");

        Ok(RunConfig {
            path,
            split,
            retain,
            verbose,
            dry_run,
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use super::RunConfig;

    #[test]
    fn requires_file() {
        let clap = RunConfig::create_clap_app();
        let err = clap
            .try_get_matches_from(vec![env!("CARGO_PKG_NAME"), "-n", "5"])
            .unwrap_err();
        assert_eq!(err.kind(), clap::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn must_give_part_size_or_num_parts() {
        // Neither
        let clap = RunConfig::create_clap_app();
        let err = clap
            .try_get_matches_from(vec![env!("CARGO_PKG_NAME"), "Cargo.toml"])
            .unwrap_err();
        assert_eq!(err.kind(), clap::ErrorKind::MissingRequiredArgument);

        // One
        let clap = RunConfig::create_clap_app();
        let matches = clap
            .try_get_matches_from(vec![
                env!("CARGO_PKG_NAME"),
                "-n",
                "5",
                "Cargo.toml",
            ])
            .unwrap();
        assert!(matches.is_present("num_parts"));
        assert!(RunConfig::process_matches(&matches).is_ok());

        let clap = RunConfig::create_clap_app();
        let matches = clap
            .try_get_matches_from(vec![
                env!("CARGO_PKG_NAME"),
                "-s",
                "512",
                "Cargo.toml",
            ])
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
        assert_eq!(err.kind(), clap::ErrorKind::ArgumentConflict);
    }
}
