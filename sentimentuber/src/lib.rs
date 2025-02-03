//! # SentimentTuber
//!
//! `sentimenttuber` is a tool to read in a text file produced by 
//! OBS's localvocal plugin and determine an image to show for an
//! avatar based on keywords and sentiment.

use std::error::Error;
use std::path::PathBuf;

use clap::Parser;

/// Run the program with the given configuration
///
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("{:?}", config);
    Ok(())
}

#[derive(Debug)]
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    #[arg(long = "file", value_name = "FILE")]
    pub input_text_file_path: PathBuf,

    #[arg(long = "ip")]
    pub obs_ip: String,

    #[arg(short = 'p', long = "password")]
    pub obs_password: String,

    #[arg(long = "port", default_value_t = 4455)]
    pub obs_port: u16
}

impl Config {
    pub fn parse_env() -> Config {
        Config::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_have_boilerplate() {
        assert!(true);
    }   
}
