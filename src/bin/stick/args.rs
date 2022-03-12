use crate::Result;
use crate::StickError::*;
use chopstick::EXTENSION_PREFIX;
use clap::{Arg, ArgMatches};
use os_str_bytes::RawOsStr;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct RunConfig {
    pub original_file: PathBuf,
    // Ordered list of parts
    pub part_paths: Vec<PathBuf>,
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
            .about("Reconstruct files from parts efficiently")
            .trailing_var_arg(true)
            .arg(
                Arg::new("retain")
                    .short('r')
                    .long("retain")
                    .visible_aliases(&["no-delete", "preserve"])
                    .help("Don't delete the part files")
                    .long_help("Don't delete the part files (requires more disk space)"),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Makes stick tell you what it's doing"),
            )
            .arg(
                Arg::new("dry-run")
                    .long("dry-run")
                    .visible_alias("dry")
                    .help("Don't actually do anything, just tell me about it")
                    .long_help("Don't actually do anything, just tell me about it (implies --verbose)"),
            )
            .arg(
                Arg::new("file_name")
                    .help("The file to reconstruct")
                    .long_help(
                        "The file to reconstruct. \
                        You only need to specify one part, providing the \
                        extension is optional",
                    )
                    .required(true)
                    .takes_value(true)
                    .allow_invalid_utf8(true),
            )
    }

    fn process_matches(clap_matches: &ArgMatches) -> Result<Self> {
        let retain = clap_matches.is_present("retain");
        let dry_run = clap_matches.is_present("dry-run");
        let verbose = dry_run || clap_matches.is_present("verbose");

        // Unwrap is assured by "file_name" being a required argument taking
        // a value
        let path_ref: &Path =
            clap_matches.value_of_os("file_name").unwrap().as_ref();

        let search_stem = path_ref.remove_chopstick_extension();
        // Try and use parent folder from given path, failing that use the
        // working directory
        let mut parent_folder = match path_ref
            .parent()
            // .parent() can just return an empty string which is annoying
            .filter(|p| !p.as_os_str().is_empty())
        {
            Some(parent) => parent.to_owned(),
            None => env::current_dir().map_err(BadParent)?,
        };

        let discovered_paths = find_parts_in(&parent_folder, &search_stem);

        if discovered_paths.is_empty() {
            Err(NoParts)
        } else if discovered_paths.len() > 1
            && verify_discovered_parts(&discovered_paths)
        {
            // Add file name onto parent folder to reconstruct file into
            // If we don't use parent_folder here, the file will be recreated
            // in the working directory, instead of the file's directory
            parent_folder.push(search_stem);
            Ok(RunConfig {
                original_file: parent_folder,
                part_paths: discovered_paths,
                retain,
                verbose,
                dry_run,
            })
        } else {
            // Pretty up format a bit to make life easier for StickError
            let files_found = discovered_paths
                .into_iter()
                .map(|pb| pb.file_name().unwrap().to_owned())
                .collect::<Vec<_>>();
            Err(IncompleteParts(files_found))
        }
    }
}

fn find_parts_in<P: AsRef<Path>>(root: P, search_stem: &OsStr) -> Vec<PathBuf> {
    WalkDir::new(root)
        .min_depth(1)
        .max_depth(1) // Search same folder
        .follow_links(true)
        // Zero padding file names in chop means sorting by file name here
        // lets us get the parts in numerical order, which is useful later for
        // verifying we have a full run of them
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            // Check file extension...
            e.path()
                .extension()
                .and_then(OsStr::to_str)
                .map(|ext_str| {
                    ext_str.starts_with(EXTENSION_PREFIX)
                })
                .unwrap_or(false)
                // ...and file name
                && e.path().file_name().unwrap().remove_chopstick_extension() == search_stem
        })
        .filter_map(|rde| match rde {
            Ok(de) => Some(de.into_path()),
            Err(why) => {
                let path =
                    why.path().expect("Read error not associated with a path");
                eprintln!("Failed to read {:?}: {}", path, why);
                None
            }
        })
        .collect()
}

// Check extensions indicate a complete set of parts
// i.e. .p1, .p2, .p3 instead of .p2, .p4, .p5
fn verify_discovered_parts(part_paths: &[PathBuf]) -> bool {
    part_paths
        .iter()
        .enumerate()
        .map(|(index, path)| ((index + 1).to_string(), path))
        .all(|(index, path)| {
            path.extension()
                .and_then(OsStr::to_str)
                // ends_with handily ignores the zero padding
                .map(|ext| ext.ends_with(&index))
                .unwrap_or(false)
        })
}

trait RemoveChopstickExtension {
    fn remove_chopstick_extension(&self) -> OsString;
}

impl RemoveChopstickExtension for OsStr {
    fn remove_chopstick_extension(&self) -> OsString {
        let haystack = RawOsStr::new(self);
        let extension_start_index: Option<usize> = haystack.rfind('.');
        match haystack.as_ref().rsplit_once(EXTENSION_PREFIX) {
            Some((file_stem, _))
                if extension_start_index
                    .map(|index| index + 1 == file_stem.raw_len())
                    .unwrap_or(false) =>
            {
                file_stem
                    .strip_suffix('.')
                    .unwrap()
                    .to_os_str()
                    .into_owned()
            }
            _ => self.to_owned(),
        }
    }
}

impl RemoveChopstickExtension for Path {
    fn remove_chopstick_extension(&self) -> OsString {
        self.as_os_str().remove_chopstick_extension()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use std::ffi::OsString;

    #[test]
    fn path_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let mut expected_parts = Vec::with_capacity(9);
        (0..9).into_iter().for_each(|n| {
            let part = temp_dir.child(&format!("foo.p{}", n + 1));
            part.touch().expect("Failed to create file");
            expected_parts.push(part.to_path_buf());
        });
        let actual_parts = find_parts_in(&temp_dir, &OsString::from("foo"));
        assert_eq!(actual_parts, expected_parts);
    }

    #[test]
    fn part_verification_good() {
        let temp_dir = TempDir::new().unwrap();
        let mut parts = Vec::with_capacity(9);
        (0..9).into_iter().for_each(|n| {
            let part = temp_dir.child(&format!("foo.p{}", n + 1));
            part.touch().expect("Failed to create file");
            parts.push(part.to_path_buf());
        });
        assert!(verify_discovered_parts(&parts), "Simple case");

        let temp_dir = TempDir::new().unwrap();
        let mut parts = Vec::with_capacity(9);
        (0..9).into_iter().for_each(|n| {
            let part = temp_dir.child(&format!("foo.tar.gz.p{}", n + 1));
            part.touch().expect("Failed to create file");
            parts.push(part.to_path_buf());
        });
        assert!(verify_discovered_parts(&parts), "Long extension");

        let temp_dir = TempDir::new().unwrap();
        let mut parts = Vec::with_capacity(9);
        (0..9).into_iter().for_each(|n| {
            let part = temp_dir.child(&format!("foo.p.bar.p{}", n + 1));
            part.touch().expect("Failed to create file");
            parts.push(part.to_path_buf());
        });
        assert!(
            verify_discovered_parts(&parts),
            "False positive extension prefix"
        );
    }

    #[test]
    fn part_verification_bad() {
        let temp_dir = TempDir::new().unwrap();
        let mut parts = Vec::with_capacity(9);
        (0..9).into_iter().for_each(|n| {
            if n != 2 {
                let part = temp_dir.child(&format!("foo.p{}", n + 1));
                part.touch().expect("Failed to create file");
                parts.push(part.to_path_buf());
            }
        });
        assert!(!verify_discovered_parts(&parts), "One missing");

        let temp_dir = TempDir::new().unwrap();
        let mut parts = Vec::with_capacity(9);
        (0..9).into_iter().for_each(|n| {
            if n > 1 {
                let part = temp_dir.child(&format!("foo.p{}", n + 1));
                part.touch().expect("Failed to create file");
                parts.push(part.to_path_buf());
            }
        });
        assert!(!verify_discovered_parts(&parts), "Two missing");
    }

    fn extension_removal_test_runner(test_data: &[(&str, &str)]) {
        test_data
            .iter()
            .map(|(inp, out)| (Path::new(inp), OsString::from(out)))
            .for_each(|(path, expected)| {
                assert_eq!(path.remove_chopstick_extension(), expected)
            });
    }

    #[test]
    fn extension_removal() {
        let data = vec![
            ("bar.p01", "bar"),
            ("foo.tgz.p01", "foo.tgz"),
            ("../foo/bar/../foo.p999999", "../foo/bar/../foo"),
            ("barmy.hber.afv.p00.asdf.p10", "barmy.hber.afv.p00.asdf"),
        ];
        extension_removal_test_runner(&data);
    }

    #[test]
    fn extension_removal_noop() {
        let data = vec![
            ("bar", "bar"),
            ("foo.tgz", "foo.tgz"),
            ("../foo/bar/../foo", "../foo/bar/../foo"),
            ("barmy.hber.afv.p00.asdf", "barmy.hber.afv.p00.asdf"),
        ];
        extension_removal_test_runner(&data);
    }
}
