//! # SentimentTuber
//!
//! `sentimenttuber` is a tool to read in a text file produced by
//! OBS's localvocal plugin and determine an image to show for an
//! avatar based on keywords and sentiment.

mod cli;
use cli::Config;

mod obs;
use obs::OBSController;

mod rules;
use rules::load_from_file;

mod sentiment;
use sentiment::SentimentEngine;
use sentiment::get_context_polarity;

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::fs;
use std::sync::mpsc;
use std::time::Duration;
use std::thread;

// TODO: Cite https://github.com/ckw017/vader-sentiment-rust?tab=readme-ov-file#citation-information
fn main() -> anyhow::Result<()> {
    let config = Config::parse_env();
    let rules = load_from_file(&config.rules_file).unwrap_or_else(|e| {
        panic!(
            "Could not load rules file [{0}] {1}", 
            config.rules_file.to_string_lossy(),
            e
        )
    });

    let obs_control = OBSController::new(&config)?;
    let (obs_sender, obs_receiver) = mpsc::channel::<String>();
    thread::spawn(move || {
        for image_to_show in obs_receiver {
            obs_control.swap_image_to(&image_to_show).expect("OBS failed to swap images");
        }
    });

    let analyzer = vader_sentiment::SentimentIntensityAnalyzer::new();
    let mut polarity_engine = SentimentEngine::new(|sentence| {
        get_context_polarity(sentence, &analyzer)
    });
    polarity_engine.set_rules(rules);

    let (sender, receiver) = mpsc::channel();
    let debounce_milli = config.event_debouncing_duration_ms;
    let mut debouncer = new_debouncer(Duration::from_millis(debounce_milli), sender).unwrap();
    let path = config.input_text_file_path.as_path();
    debouncer.watcher().watch(path, RecursiveMode::Recursive).unwrap();

    // Blocks forever
    for res in receiver {
        match res {
            Ok(_) => {
                let s = fs::read_to_string(path).expect("could not get text data from file shared with localvocal");
                polarity_engine.add_context(s);
                let sentiment_action = polarity_engine.get_action();
                obs_sender.send(sentiment_action.show.to_string()).unwrap();
            }
            Err(e) => {
                eprintln!("watch error: {:?}", e);
            }
        };
    }
    Ok(())
}
