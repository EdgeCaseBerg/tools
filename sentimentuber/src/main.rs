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

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;
use std::thread;
use std::collections::VecDeque;
use std::collections::HashMap;


// TODO: Cite https://github.com/ckw017/vader-sentiment-rust?tab=readme-ov-file#citation-information
fn main() -> anyhow::Result<()> {
    let config = Config::parse_env();
    let rules = load_from_file(&config.rules_file).unwrap_or_else(|_| {
        panic!(
            "Could not load rules file [{0}]", 
            config.rules_file.to_string_lossy()
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
    let state_to_image_file = get_emotion_to_image_map();

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

                let emotional_state = get_emotional_state(&current_context, &analyzer);
                let image_to_show = state_to_image_file.get(&emotional_state).unwrap();
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

#[derive(Debug, Hash, Eq, PartialEq)]
enum EmotionalState {
    Neutral,
    Mad,
    MakingAPromise,
    Sad,
    Smug,
    ThumbsUp,
}

fn get_emotion_to_image_map() -> HashMap<EmotionalState, &'static str> {
    HashMap::from([
        (EmotionalState::Neutral, "./data/neutral.png"),
        (EmotionalState::Mad, "./data/mad.png"),
        (EmotionalState::MakingAPromise, "./data/promise.png"),
        (EmotionalState::Sad, "./data/sad.png"),
        (EmotionalState::Smug, "./data/smug.png"),
        (EmotionalState::ThumbsUp, "./data/thumbsup.png"),
    ])
}

// TODO:
// probably make this file based rather than compile code based to some extent
// define things like keyword foo -> Bla and letting rules cascade would be good
// not to mention we'll need to think about blinking or similar things one should do regularly.
fn get_emotional_state(
    sentence: &str,
    analyzer: &vader_sentiment::SentimentIntensityAnalyzer,
) -> EmotionalState {
    if sentence.contains("good job") {
        return EmotionalState::ThumbsUp;
    }
    if sentence.contains("promise") {
        return EmotionalState::MakingAPromise;
    }
    if sentence.contains("I'm the best") || sentence.contains("I am the best") {
        return EmotionalState::Smug;
    }

    if sentence.contains("bummer") {
        return EmotionalState::Sad;
    }

    let scores = analyzer.polarity_scores(sentence);
    // we'll tweak these later once we know more about the library.
    let positive = scores.get("pos").unwrap_or(&0.0);
    let negative = scores.get("neg").unwrap_or(&0.0);
    let neutral = scores.get("neu").unwrap_or(&0.0);

    if positive < negative && neutral < negative {
        return EmotionalState::Mad;
    }

    EmotionalState::Neutral
}
