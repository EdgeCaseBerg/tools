//! # Nav Update
//!
//! `nav-update` is a quick and dirty program to
//! update my site navigation without messing with
//! existing indentation or the like. It takes in
//! two arguments, the first being the file to use
//! as the source of truth as to what the `<header>`
//! data should be, and the second beind the path or
//! file to update according to the template.

use std::error::Error;
use std::fs;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let template_file = fs::read_to_string(&config.template_file)?;
    let header_data = header_lines_from_template(&template_file);

    if config.path_is_directory() {
        update_files_in_dir(&config, header_data);
    } else {
        update_file_in_dir(&config, header_data);
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

fn update_files_in_dir(config: &Config, template_header_lines: Vec<&str>) {}

fn update_file_in_dir(config: &Config, template_header_lines: Vec<&str>) {}

pub struct Config {
    pub template_file: String,
    pub path_to_update: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        if let None = args.next() {
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

    pub fn path_is_directory(&self) -> bool {
        match fs::metadata(&self.path_to_update) {
            Ok(metadata) => metadata.is_dir(),
            Err(_) => false,
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
}
