use crate::Result;
use crate::StickError::*;
use chopstick::EXTENSION_PREFIX;
use clap::{AppSettings, Arg, ArgMatches};
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct RunConfig {
    pub original_file: PathBuf,
    // Ordered list of parts
    pub part_paths: Vec<PathBuf>,
    pub retain: bool,
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
            .about("Reconstruct files from parts efficiently")
            .setting(AppSettings::TrailingVarArg)
            .arg(
                Arg::new("retain")
                    .short('r')
                    .long("retain")
                    .visible_aliases(&["no-delete", "preserve"])
                    .help("Don't delete the part files")
                    .long_help("Don't delete the part files (requires more disk space)"),
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

        // Unwrap is assured by "file_name" being a required argument taking
        // a value
        let path_ref: &Path =
            clap_matches.value_of_os("file_name").unwrap().as_ref();

        let search_stem = path_ref
            .file_stem()
            .ok_or_else(|| NotRecognised(path_ref.to_path_buf()))?;
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

        let discovered_paths = WalkDir::new(&parent_folder)
            .min_depth(1)
            .max_depth(1) // Search same folder
            .follow_links(true)
            // Zero padding file names in chop means sorting by file name here
            // lets us get the parts in order, which is useful later for
            // verifying we have a full run of them
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                // Check file name...
                e.path()
                    .file_stem()
                    .map(|stem| stem == search_stem)
                    .unwrap_or(false)
                    // ...and file extension
                    && e.path()
                        .extension()
                        .and_then(OsStr::to_str)
                        .map(|ext_str| ext_str.starts_with(EXTENSION_PREFIX))
                        .unwrap_or(false)
            })
            .filter_map(|e| match e {
                Ok(de) => Some(de.into_path()),
                Err(why) => {
                    let path = why
                        .path()
                        .expect("Read error not associated with a path");
                    eprintln!("Failed to read {:?}: {}", path, why);
                    None
                }
            })
            .collect::<Vec<_>>();

        // Check extensions indicate a complete set of parts
        // i.e. .p1, .p2, .p3 instead of .p2, .p4, .p5
        let we_good = discovered_paths
            .iter()
            .enumerate()
            .map(|(index, path)| ((index + 1).to_string(), path))
            .all(|(index, path)| {
                path.extension()
                    .and_then(OsStr::to_str)
                    // ends_with handily ignores the zero padding
                    .map(|ext| ext.ends_with(&index))
                    .unwrap_or(false)
            });

        if we_good {
            // Add file name onto parent folder to reconstruct file into
            // If we don't use parent_folder here, the file will be recreated
            // in the working directory, instead of the file's directory
            parent_folder.push(search_stem);
            Ok(RunConfig {
                original_file: parent_folder,
                part_paths: discovered_paths,
                retain,
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
