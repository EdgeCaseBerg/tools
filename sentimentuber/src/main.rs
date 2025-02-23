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
use rules::SentimentAction;

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;
use std::thread;
use std::collections::VecDeque;

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
    println!("{:?}", rules);
    let obs_control = OBSController::new(&config)?;
    let (obs_sender, obs_receiver) = mpsc::channel::<String>();
    thread::spawn(move || {
        for res in obs_receiver {
            let image_to_show = res;
            obs_control.swap_image_to(&image_to_show).expect("OBS failed to swap images");
        }
    });

    let analyzer = vader_sentiment::SentimentIntensityAnalyzer::new();

    let (sender, receiver) = mpsc::channel();
    let debounce_milli = config.event_debouncing_duration_ms;
    let mut debouncer = new_debouncer(Duration::from_millis(debounce_milli), sender).unwrap();

    let path = config.input_text_file_path.as_path();
    debouncer.watcher().watch(path, RecursiveMode::Recursive).unwrap();

    let mut text_context: VecDeque<(Instant, String)> = VecDeque::new();
    // Blocks forever
    for res in receiver {
        match res {
            Ok(_) => {
                let s = get_data_from_file(path);
                let mut current_context = String::new();

                let right_now = Instant::now();
                let drop_time = right_now - Duration::from_secs(10); // TODO make configurable
                text_context.push_back((right_now, s));
                text_context.retain(|tuple| {
                     if tuple.0.ge(&drop_time) {
                         current_context.push_str(&tuple.1.clone());
                     }
                     tuple.0.ge(&drop_time)
                });

                println!("{:?}", current_context);

                let sentiment_action = get_emotional_state(&current_context, &analyzer);
                let image_to_show = sentiment_action.show;
                obs_sender.send(image_to_show.to_string()).unwrap();
            }
            Err(e) => {eprintln!("watch error: {:?}", e);}
        };
    }
    Ok(())
}

fn get_data_from_file(path: &Path) -> String {
    fs::read_to_string(path).expect("could not get text data from file shared with localvocal")
}

// TODO:
// probably make this file based rather than compile code based to some extent
// define things like keyword foo -> Bla and letting rules cascade would be good
// not to mention we'll need to think about blinking or similar things one should do regularly.
fn get_emotional_state(
    sentence: &str,
    analyzer: &vader_sentiment::SentimentIntensityAnalyzer,
) -> SentimentAction {
    if sentence.contains("good job") {
        return SentimentAction {
            show: "./data/thumbsup.png".to_string()
        };
    }
    if sentence.contains("promise") {
        return SentimentAction {
            show: "./data/promise.png".to_string()
        };
    }
    if sentence.contains("I'm the best") || sentence.contains("I am the best") {
        return SentimentAction {
            show: "./data/smug.png".to_string()
        };
    }

    if sentence.contains("bummer") {
        return SentimentAction{
            show: "./data/sad.png".to_string()
        };
    }

    let scores = analyzer.polarity_scores(sentence);
    // we'll tweak these later once we know more about the library.
    let positive = scores.get("pos").unwrap_or(&0.0);
    let negative = scores.get("neg").unwrap_or(&0.0);
    let neutral = scores.get("neu").unwrap_or(&0.0);

    if positive < negative && neutral < negative {
        return SentimentAction {
            show: "./data/mad.png".to_string()
        };
    }

    SentimentAction {
        show: "./data/neutral.png".to_string()
    }
}
