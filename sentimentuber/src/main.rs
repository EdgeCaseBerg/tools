//! # SentimentTuber
//!
//! `sentimenttuber` is a tool to read in a text file produced by 
//! OBS's localvocal plugin and determine an image to show for an
//! avatar based on keywords and sentiment.

mod cli;

use cli::Config;

use notify::{Event, RecursiveMode, Result, Watcher};
use std::sync::mpsc;
use std::path::Path;
use std::fs;
use std::process;
use std::env;

use vader_sentiment;
use obws::Client;
use obws::responses::scene_items::SceneItem;
use obws::requests::scene_items::Source;
use obws::responses::sources::SourceId;
use obws::requests::inputs::SetSettings;

use tokio;
use anyhow;
use std::collections::HashMap;
use serde_json::json;

// TODO: Cite https://github.com/ckw017/vader-sentiment-rust?tab=readme-ov-file#citation-information

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse_env();
    println!("{:?}", config);

    let path = config.input_text_file_path.as_path();
    let ip = config.obs_ip;
    let password = config.obs_password;
    let port = config.obs_port;

    let analyzer = vader_sentiment::SentimentIntensityAnalyzer::new();

    let state_to_image_file = get_emotion_to_image_map();

    let client = Client::connect(ip, port, Some(password)).await?;

    // pre-fetch the image container we'll be tweaking
    let image_source_id = get_image_scene_item(&client).await?;

    let (sender, receiver) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(sender)?;
    watcher.watch(path, RecursiveMode::Recursive)?;
    // Block forever, printing out events as they come in
    for res in receiver {
        match res {
            Ok(event) => {
                // TODO:
                // probably take a bit of file at a time, append to a buffer
                // and then analyze the buffer rather than do one bit at a time
                // Also we should keep track of how long its been since data 
                // came in, and then we could use that to do mouth flaps
                let s = get_data_from_file(path);
                // we'll do something with the score later.
                let emotional_state = get_emotional_state(&s, &analyzer);
                let image_to_show = state_to_image_file.get(&emotional_state).unwrap();
                swap_obs_image_to(&image_source_id, &image_to_show, &client).await?;
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn get_data_from_file(path: &Path) -> String {
    let s = fs::read_to_string(path).expect("could not get text data from file shared with localvocal");
    return s
}

async fn get_image_scene_item(client: &Client) -> anyhow::Result<SourceId> {
    let scenes_struct = client.scenes().list().await?;
    let test_scene = scenes_struct.scenes.iter().find(|scene| {
        scene.id.name.contains("SentimentTuber")
    }).expect("Could not find OBS scene by name");

    let items_in_scene = client.scene_items().list(test_scene.id.clone().into()).await?;
    let image_source = items_in_scene.iter().find(|item| {
        // TODO: use a better name than "Image" obviously.
        item.source_name.contains("Image")
    }).expect("No image source found in OBS scene for the avatar");

    let source_id = client.scene_items().source(
        Source {
            scene: test_scene.id.clone().into(),
            item_id: image_source.id.clone().into()
        }
    ).await?;

    Ok(source_id)
}

async fn swap_obs_image_to(source_id: &SourceId, new_file_path: &str, client: &Client) -> anyhow::Result<()> {
    let path = Path::new(new_file_path);
    let absolute = Path::canonicalize(path)?;
    let setting = json!({"file": absolute});
    client.inputs().set_settings(SetSettings {
        input: (&*source_id.name).into(),
        settings: &setting,
        overlay: Some(true)
    }).await?;

    Ok(())
}

#[derive(Debug)]
#[derive(Hash, Eq, PartialEq)]
enum EmotionalState {
    Neutral,
    Mad,
    MakingAPromise,
    Sad,
    Smug,
    ThumbsUp
}

fn get_emotion_to_image_map() -> HashMap<EmotionalState, &'static str> {
    HashMap::from([
        (EmotionalState::Neutral, "./data/neutral.png"),
        (EmotionalState::Mad, "./data/mad.png"),
        (EmotionalState::MakingAPromise, "./data/promise.png"),
        (EmotionalState::Sad, "./data/sad.png"),
        (EmotionalState::Smug, "./data/smug.png"),
        (EmotionalState::ThumbsUp, "./data/thumbsup.png")
    ])
}

// TODO:
// probably make this file based rather than compile code based to some extent
// define things like keyword foo -> Bla and letting rules cascade would be good
// not to mention we'll need to think about blinking or similar things one should do regularly.
fn get_emotional_state(sentence: &String, analyzer: &vader_sentiment::SentimentIntensityAnalyzer) -> EmotionalState {
    if sentence.contains("good job") {
        return EmotionalState::ThumbsUp
    }
    if sentence.contains("promise") {
        return EmotionalState::MakingAPromise
    }
    if sentence.contains("I'm the best") || sentence.contains("I am the best") {
        return EmotionalState::Smug
    }

    if sentence.contains("bummer") {
        return EmotionalState::Sad
    }

    let scores = analyzer.polarity_scores(&sentence);
    // we'll tweak these later once we know more about the library.
    let positive = scores.get("pos").unwrap_or_else(|| &0.0);
    let negative = scores.get("neg").unwrap_or_else(|| &0.0);
    let neutral = scores.get("neu").unwrap_or_else(|| &0.0);

    if positive < negative && neutral < negative {
        return EmotionalState::Mad
    }

    EmotionalState::Neutral
}