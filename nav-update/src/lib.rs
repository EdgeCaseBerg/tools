//! # Nav Update
//! 
//! `nav-update` is a quick and dirty program to 
//! update my site navigation without messing with
//! existing indentation or the like. It takes in
//! two arguments, the first being the file to use
//! as the source of truth as to what the `<header>`
//! data should be, and the second beind the path or
//! file to update according to the template.

use std::fs;
use std::error::Error;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let _ = fs::read_to_string(config.template_file)?;

    println!("{:?}", config.path_to_update);
    
    Ok(())
}

pub struct Config {
    pub template_file: String,
    pub path_to_update: String,
}

impl Config {
    pub fn build(
        mut args: impl Iterator<Item = String>
    ) -> Result<Config, &'static str> {
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

}