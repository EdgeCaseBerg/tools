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
use rules::ContextPolarity;
use rules::SentimentAction;

mod sentiment;
use sentiment::SentimentEngine;

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::fs;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::thread;

// TODO: Cite https://github.com/ckw017/vader-sentiment-rust?tab=readme-ov-file#citation-information
fn main() -> anyhow::Result<()> {
    let config = Config::parse_env();
    let obs_sender = start_obs_controller_on_thread(&config)?;
    let (sender, receiver) = mpsc::channel();
    let (context_sender, context_receiver) = mpsc::channel();
    let debounce_milli = config.event_debouncing_duration_ms;
    
    regularly_send_tick_with(context_sender.clone(), config.context_retention_seconds);
    emit_action_on_sentiment(&config, context_receiver, obs_sender);

    // Blocks forever
    let mut debouncer = new_debouncer(Duration::from_millis(debounce_milli), sender).unwrap();
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
    Ok(())
}

fn get_context_polarity(sentence: &str, analyzer: &vader_sentiment::SentimentIntensityAnalyzer) -> ContextPolarity {
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

fn start_obs_controller_on_thread(config: &Config) -> Result<Sender<SentimentAction>, obws::error::Error> {
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

fn regularly_send_tick_with(sender: Sender<String>, every_t_seconds: u64) {
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

fn emit_action_on_sentiment(config: &Config, context_receiver: Receiver<String>, obs_sender: Sender<SentimentAction>) {
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