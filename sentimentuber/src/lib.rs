//! # SentimentTuber
//!
//! `sentimenttuber` is a tool to read in a text file produced by
//! OBS's localvocal plugin and determine an image to show for an
//! avatar based on keywords and sentiment.

pub mod cli;
use cli::Config;

mod obs;
use obs::OBSController;

pub mod rules;
use rules::load_from_file;
use rules::ContextPolarity;
use rules::SentimentAction;

mod sentiment;
use sentiment::SentimentEngine;

pub mod gui;

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::fs;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::thread;
use std::path::Path;

pub fn get_context_polarity(sentence: &str, analyzer: &vader_sentiment::SentimentIntensityAnalyzer) -> ContextPolarity {
    let scores = analyzer.polarity_scores(sentence);
    let positive = scores.get("pos").unwrap_or(&0.0);
    let negative = scores.get("neg").unwrap_or(&0.0);
    let neutral = scores.get("neu").unwrap_or(&0.0);

    ContextPolarity {
        positive: *positive,
        negative: *negative,
        neutral: *neutral
    }
}

pub fn start_obs_controller_on_thread(config: &Config) -> Result<Sender<SentimentAction>, obws::error::Error> {
    let obs_control = OBSController::new(&config)?;
    let (obs_sender, obs_receiver) = mpsc::channel::<SentimentAction>();
    thread::spawn(move || {
        for image_to_show in obs_receiver {
            match obs_control.swap_image_to(&image_to_show.show) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("OBS failed to swap images {e:?}");
                }
            }
        }
    });
    Ok(obs_sender)
}

pub fn regularly_send_tick_with(sender: Sender<String>, every_t_seconds: u64) {
    thread::spawn(move || {
        loop {
            let sleep_for_seconds = Duration::from_secs(every_t_seconds);
            thread::sleep(sleep_for_seconds);
            if let Err(send_error) = sender.send(String::new()) {
                eprintln!("Error on sending tick {send_error:?}");
            }
        }
    });
}

// TODO: Cite https://github.com/ckw017/vader-sentiment-rust?tab=readme-ov-file#citation-information
pub fn emit_action_on_sentiment(config: &Config, context_receiver: Receiver<String>, obs_sender: Sender<SentimentAction>) {
    let rules = load_from_file(&config.rules_file).unwrap_or_else(|e| {
        panic!(
            "Could not load rules file [{0}] {1}", 
            config.rules_file.to_string_lossy(),
            e
        )
    });

    let analyzer = vader_sentiment::SentimentIntensityAnalyzer::new();
    let default_action = SentimentAction {
        show: config.default_action.clone()
    };
    let mut polarity_engine = SentimentEngine::new(default_action, move |sentence| {
        get_context_polarity(sentence, &analyzer)
    });
    polarity_engine.set_context_duration(config.context_retention_seconds);
    polarity_engine.set_rules(rules);
    thread::spawn(move || {
        for new_context in context_receiver {
            polarity_engine.add_context(new_context);
            let sentiment_action = polarity_engine.get_action();
            if let Err(send_error) = obs_sender.send(sentiment_action) {
                eprintln!("{:?}", send_error);
            }
        }
    });
}

pub fn emit_file_contents_on_change_forever(config: Config, context_sender: Sender<String>) {
    thread::spawn(move || {
        let (sender, receiver) = mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(config.event_debouncing_duration_ms), sender).unwrap();
        let path = config.input_text_file_path.as_path();
        debouncer.watcher().watch(path, RecursiveMode::Recursive).unwrap();
        for res in receiver {
            if let Err(error) = res {
                eprintln!("Watch error {error:?}");
                continue;
            }

            match fs::read_to_string(path) {
                Err(file_error) => {
                    eprintln!("could not get text data from file shared with localvocal: {file_error:?}");
                    continue;
                },
                Ok(new_context) => {
                    if let Err(send_error) = context_sender.send(new_context) {
                        eprintln!("{:?}", send_error);
                    }
                }
            }
        }
    });
}


pub fn get_full_path(relative_path: &str) -> Result<String, std::io::Error> {
    let path = Path::new(relative_path);
    let path_buf = Path::canonicalize(path)?;
    Ok(path_buf.display().to_string())
}