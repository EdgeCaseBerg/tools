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
use rules::SentimentRule;
use rules::SentimentField;
use rules::Relation;

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

                let polarity = get_context_polarity(&current_context, &analyzer);
                let sentiment_action = get_action_for_sentiment(&current_context, &polarity, &rules);
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

/// Returns None if no rule defined, Some(T|F) for if there was a match otherwise
fn context_contains_words(rule: &SentimentRule, sentence: &str) -> Option<bool> {
    let condition = &rule.condition;
    if let Some(words) = &condition.contains_words {
        let contains_words = words.iter().any(|word| {
            sentence.contains(word)
        });
        return Some(contains_words);
    }
    None
}

struct ContextPolarity {
    positive: f64,
    negative: f64,
    neutral: f64
}

impl ContextPolarity {
    fn for_field(&self, field: &SentimentField) -> f64 {
         match field {
            SentimentField::Positive => self.positive,
            SentimentField::Negative => self.negative,
            SentimentField::Neutral => self.neutral ,
        }
    }
}

fn context_in_polarity_range(rule: &SentimentRule, polarity: &ContextPolarity) -> Option<bool> {
    let condition = &rule.condition;
    if let Some(ranges) = &condition.polarity_ranges {
        let is_in_range = ranges.iter().all(|range| {
            let field = polarity.for_field(&range.field);
            range.low <= field && field <= range.high
        });
        return Some(is_in_range)
    }
    None
}

fn context_has_polarity_relations(rule: &SentimentRule, polarity: &ContextPolarity) -> Option<bool> {
    let condition = &rule.condition;
    if let Some(relations) = &condition.polarity_relations {
        let relation_is_true = relations.iter().all(|relation| {
            let left = polarity.for_field(&relation.left);
            let right = polarity.for_field(&relation.right);
            match &relation.relation {
                Relation::GT => left > right,
                Relation::LT => left < right,
                Relation::EQ => left == right,
            }
        });
        return Some(relation_is_true);
    }
    None
}

fn get_context_polarity(sentence: &str, analyzer: &vader_sentiment::SentimentIntensityAnalyzer) -> ContextPolarity {
    let scores = analyzer.polarity_scores(sentence);
    // we'll tweak these later once we know more about the library. 
    let positive = scores.get("pos").unwrap_or(&0.0);
    let negative = scores.get("neg").unwrap_or(&0.0);
    let neutral = scores.get("neu").unwrap_or(&0.0);

    ContextPolarity {
        positive: *positive,
        negative: *negative,
        neutral: *neutral
    }
}

// TODO for p3 post, discrete to continuous emotion to support blink etc?
fn get_action_for_sentiment(
    sentence: &str,
    polarity: &ContextPolarity,
    current_rules: &[SentimentRule]
) -> SentimentAction {
    let maybe_action = current_rules.iter().find(|&rule| {
        let maybe_contextual_word_match = context_contains_words(rule, sentence);
        if let Some(is_word_matched) = maybe_contextual_word_match {
            // Early return until we have a need to support multiple matches
            // across rule types.
            if is_word_matched {
                return true;
            }
        }

        let maybe_polarity_in_range = context_in_polarity_range(rule, polarity);
        if let Some(is_in_range) = maybe_polarity_in_range {
            if is_in_range {
                return true;
            }
        }
        
        let maybe_polarity_relation_applies = context_has_polarity_relations(rule, polarity);
        if let Some(relation_is_true) = maybe_polarity_relation_applies {
            if relation_is_true {
                return true;
            }
        }

        false
    });

    match maybe_action {
        Some(rule_based_action) => rule_based_action.action.clone(),
        None => SentimentAction {
            show: "./data/neutral.png".to_string()
        }
    }
}
