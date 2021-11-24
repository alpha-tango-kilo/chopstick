use crate::Result;
use crate::StickError::*;
use chopstick::EXTENSION_PREFIX;
use clap::{AppSettings, Arg, ArgMatches};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct RunConfig {
    pub original_file: PathBuf,
    // Ordered list of parts
    pub part_paths: Vec<PathBuf>,
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
                Arg::new("file_name")
                    .about("The file to reconstruct")
                    .long_about(
                        "The file to reconstruct. \
                        You only need to specify one part, providing the \
                        extension is optional",
                    )
                    .required(true)
                    .takes_value(true),
            )
    }

    fn process_matches(clap_matches: &ArgMatches) -> Result<Self> {
        // Unwrap is assured by "file_name" being a required argument taking values
        let path_ref: &Path =
            clap_matches.value_of_os("file_name").unwrap().as_ref();
        let path = match path_ref.extension() {
            Some(_) => fs::canonicalize(path_ref),
            None => {
                let mut extensionless = path_ref.as_os_str().to_owned();
                extensionless.push(".p1");
                fs::canonicalize(extensionless)
            }
        };
        let path =
            path.map_err(|err| Canonicalise(path_ref.to_path_buf(), err))?;

        let search_stem = path
            .file_stem()
            .ok_or_else(|| NotRecognised(path_ref.to_path_buf()))?;
        let parent_folder =
            path.parent().ok_or_else(|| NotRecognised(path_ref.to_path_buf()))?;
        let ext = path
            .extension()
            .and_then(OsStr::to_str)
            .ok_or_else(|| NotRecognised(path_ref.to_path_buf()))?;
        if !ext.starts_with(EXTENSION_PREFIX) {
            return Err(NotRecognised(path_ref.to_path_buf()));
        }

        let discovered_paths = WalkDir::new(parent_folder)
            .min_depth(1)
            .max_depth(1) // Search same folder
            .follow_links(true)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                e.path()
                    .file_stem()
                    .map(|stem| stem == search_stem)
                    .unwrap_or(false)
            })
            .filter_map(|e| match e {
                Ok(de) => Some(de.into_path()),
                Err(why) => {
                    let path = why
                        .path()
                        .expect("Read error not associated with a path");
                    // TODO: check this reads nicely
                    eprintln!(
                        "Failed to read {}: {}",
                        path.to_string_lossy(),
                        why
                    );
                    None
                }
            })
            .collect::<Vec<_>>();

        // Check extensions indicate a complete set of parts
        // i.e. .p1, .p2, .p3 instead of .p2, .p4, .p5
        let we_good = discovered_paths
            .iter()
            .enumerate()
            .map(|(index, path)| (format!("{}", index + 1), path))
            .all(|(index, path)| {
                path.extension()
                    .and_then(OsStr::to_str)
                    .map(|ext| ext.ends_with(&index))
                    .unwrap_or(false)
            });

        if we_good {
            Ok(RunConfig {
                original_file: PathBuf::from(search_stem),
                part_paths: discovered_paths,
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
