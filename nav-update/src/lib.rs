//! # Nav Update
//!
//! `nav-update` is a quick and dirty program to
//! update my site navigation without messing with
//! existing indentation or the like. It takes in
//! two arguments, the first being the file to use
//! as the source of truth as to what the `<header>`
//! data should be, and the second beind the path or
//! file to update according to the template.

use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Runs the navigation update for the given configuration
/// Note that the path being updated will be traversed recursively,
/// filtering the updates to any and all .html files that exist within
/// the specific directories.
/// Any HTML file with a `<header>` element will have the contents updated
/// to match the template file specified in the config.
///
/// ```
/// let config = nav_update::Config {
///  path_to_update: ".".to_string(),
///  template_file: "template.html".to_string(),
/// };
/// nav_update::run(config);
/// ```
///
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let template_file = fs::read_to_string(&config.template_file)?;
    let header_data = header_lines_from_template(&template_file);

    if config.path_is_directory() {
        update_files_in_dir(&config, &header_data)?;
    } else {
        let contents_to_update = fs::read_to_string(&config.path_to_update)?;
        let new_contents = get_updated_file_contents(&header_data, contents_to_update);
        fs::write(&config.path_to_update, new_contents)?;
    }
    Ok(())
}

fn header_lines_from_template(template: &str) -> Vec<&str> {
    // if we wanted to be fancy we could make this return a &[str]
    template
        .lines()
        .skip_while(|line| !line.contains("<header>"))
        .take_while(|line| !line.contains("</header>"))
        .skip(1)
        .collect()
}

fn get_updated_file_contents(
    template_header_lines: &Vec<&str>,
    contents_to_update: String,
) -> String {
    let mut iter = contents_to_update.lines();
    let expected_max_length = contents_to_update.len() + template_header_lines.len();
    let mut new_contents = String::with_capacity(expected_max_length);
    while let Some(line) = iter.next() {
        if line.contains("<header>") {
            new_contents.push_str(line);
            new_contents.push('\n');

            for templated_line in template_header_lines.iter() {
                new_contents.push_str(templated_line);
                new_contents.push('\n');
            }
            for line in iter.by_ref() {
                if line.contains("</header>") {
                    new_contents.push_str(line);
                    break;
                } else {
                    continue;
                }
            }
        } else {
            new_contents.push_str(line);
        }
        new_contents.push('\n');
    }
    new_contents
}

fn update_files_in_dir(
    config: &Config,
    template_header_lines: &Vec<&str>,
) -> Result<(), Box<dyn Error>> {
    let entries = RecursiveDirIterator::new(Path::new(&config.path_to_update))?;
    entries
        .filter(|dir_entry| dir_entry.path().extension().and_then(|e| e.to_str()) == Some("html"))
        .for_each(|dir_entry| {
            if let Ok(contents_to_update) = fs::read_to_string(dir_entry.path()) {
                let new_contents =
                    get_updated_file_contents(template_header_lines, contents_to_update);
                if let Err(e) = fs::write(dir_entry.path(), new_contents) {
                    eprintln!("{e}");
                }
            }
        });
    Ok(())
}

pub struct Config {
    pub template_file: String,
    pub path_to_update: String,
}

impl Config {
    ///
    ///```
    /// use nav_update::Config;
    /// // Note that your args would normally come via env::args()
    /// let args = ["program name", "template.html", "."].map(|s| s.to_string());
    /// let built = Config::build(args.into_iter());
    ///```
    ///
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        if args.next().is_none() {
            return Err("Didn't get the program name somehow.");
        };

        let template_file = match args.next() {
            Some(arg) => arg,
            None => return Err("First argument should be the template file"),
        };
        let path_to_update = match args.next() {
            Some(arg) => arg,
            None => return Err("Second argument should be file or path to update"),
        };

        Ok(Config {
            template_file,
            path_to_update,
        })
    }

    fn path_is_directory(&self) -> bool {
        match fs::metadata(&self.path_to_update) {
            Ok(metadata) => metadata.is_dir(),
            Err(_) => false,
        }
    }
}

/// Iterator across a path's contents if the given path is a directory.
///
/// If you pass a non directory file as the path, the iterator will be empty.
///
/// The directory entries are lazily loaded as each directory is processed by the
/// next method. Large amounts of folders in or files in a directory may be loaded
/// unless you take care to exclude hidden folders and the like. Be careful!
///
#[derive(Debug)]
pub struct RecursiveDirIterator {
    q: VecDeque<fs::DirEntry>,
}

impl RecursiveDirIterator {
    pub fn new(d: &Path) -> Result<RecursiveDirIterator, Box<dyn Error>> {
        let mut q = VecDeque::new();
        if d.is_dir() {
            let entries = fs::read_dir(d)?;
            for entry in entries.flatten() {
                q.push_back(entry);
            }
        }

        Ok(RecursiveDirIterator { q })
    }
}

impl Iterator for RecursiveDirIterator {
    type Item = fs::DirEntry;

    /// Returns the next entry for the path being iterated on.
    ///
    /// Note that this _does_ include the directories and the individual files.
    /// We do not skip directory entries while iterator, so be prepared to process them.
    ///
    /// The iterator will traverse the files breadth first, so given a folder structure of
    /// ```markdown
    /// nav-update/
    ///     src/
    ///         lib.rs
    ///         main.rs
    ///     Cargo.toml
    /// ```
    ///
    /// When you first call .next() you'll see the contents like this:
    ///
    /// ```
    /// use std::path::Path;
    /// let mut iter = nav_update::RecursiveDirIterator::new(Path::new(".")).unwrap();
    /// assert_eq!("Cargo.toml", iter.next().unwrap().file_name());
    /// assert_eq!("src", iter.next().unwrap().file_name());
    /// assert_eq!("lib.rs", iter.next().unwrap().file_name());
    /// assert_eq!("main.rs", iter.next().unwrap().file_name());
    /// assert!(iter.next().is_none());
    /// ```
    ///
    /// Note that the exact sorting of the files at each level is platform dependent.
    fn next(&mut self) -> Option<fs::DirEntry> {
        let n = self.q.pop_front();
        match n {
            None => n,
            Some(dir_entry) => {
                let path = dir_entry.path();
                if path.is_dir() {
                    if let Ok(entries) = fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            self.q.push_back(entry);
                        }
                    } else {
                        eprintln!("Could not read entry in path {}", path.display());
                    }
                }
                Some(dir_entry)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_is_empty() {
        let args = [];
        let built = Config::build(args.into_iter());
        assert!(built.is_err());
    }

    #[test]
    fn fails_to_build_config_on_no_args() {
        let args = ["programname"];
        let built = Config::build(args.into_iter().map(|s| String::from(s)));
        assert!(built.is_err());
    }

    #[test]
    fn fails_to_build_config_on_one_arg() {
        let args = ["programname", "templatefile"];
        let built = Config::build(args.into_iter().map(|s| String::from(s)));
        assert!(built.is_err());
    }

    #[test]
    fn successfully_builds_with_two_args() {
        let args = ["programname", "templatefile", "path"];
        let built = Config::build(args.into_iter().map(|s| String::from(s)));
        assert!(built.is_ok());
    }

    #[test]
    fn reads_lines_after_headers() {
        let contents = "<header>\n<nav>\n<li>hi</li>\n</nav>\n</header>";
        let lines = header_lines_from_template(contents);
        assert_eq!(lines, vec!["<nav>", "<li>hi</li>", "</nav>"]);
    }

    #[test]
    fn splices_correctly() {
        let template = vec!["<nav>", "<li>hi</li>", "</nav>"];
        let to_replace_in = "Wont be touched at all\n<header>\n<nav>\n<li>bye</li>\n</nav>\n</header>\nWont be touched";
        let new_contents = get_updated_file_contents(&template, to_replace_in.to_string());
        assert_eq!(
            "Wont be touched at all\n<header>\n<nav>\n<li>hi</li>\n</nav>\n</header>\nWont be touched\n", 
            new_contents
        );
    }

    #[test]
    fn can_iterate_dir() {
        let dir = Path::new(".");
        let iter = RecursiveDirIterator::new(dir).expect("Could not make iterator");
        let mut rust_files_in_nav_update_dir: Vec<_> = iter
            .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        rust_files_in_nav_update_dir.sort_by_key(|d| d.file_name());

        assert_eq!(
            "lib.rs",
            rust_files_in_nav_update_dir[0]
                .file_name()
                .to_str()
                .unwrap()
        );
        assert_eq!(
            "main.rs",
            rust_files_in_nav_update_dir[1]
                .file_name()
                .to_str()
                .unwrap()
        );
    }
}
