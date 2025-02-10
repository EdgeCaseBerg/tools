//! # SentimentTuber
//!
//! `sentimenttuber` is a tool to read in a text file produced by
//! OBS's localvocal plugin and determine an image to show for an
//! avatar based on keywords and sentiment.

use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Path to localvocal (or similar) transcript file
    #[arg(long = "file", value_name = "FILE")]
    pub input_text_file_path: PathBuf,

    /// IP address of OBS websocket
    #[arg(long = "ip")]
    pub obs_ip: String,

    /// Password of OBS websocket
    #[arg(short = 'p', long = "password")]
    pub obs_password: String,

    /// Port for OBS websocket
    #[arg(long = "port", default_value_t = 4455)]
    pub obs_port: u16,

    /// Milliseconds to aggregate events from the local file system for changes to the transcript file
    #[arg(short = 'd',  long = "debounce_milliseconds", default_value_t = 50)]
    pub event_debouncing_duration_ms: u64,
}

impl Config {
    pub fn parse_env() -> Config {
        Config::parse()
    }
}
