use notify::{Event, RecursiveMode, Result, Watcher};
use std::sync::mpsc;
use std::path::Path;
use std::fs;
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


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO get this from the arguments
    let path = Path::new("./data/text");
    let analyzer = vader_sentiment::SentimentIntensityAnalyzer::new();

    let state_to_image_file = get_emotion_to_image_map();

    let (ip, password) = get_ip_and_obs_password();
    let client = Client::connect(ip, 4455, Some(password)).await?;

    // pre-fetch the image container we'll be tweaking
    let scene_list = client.scenes().list().await?;
    println!("{:#?}", scene_list);

    let image_source_id = get_image_scene_item(&client).await?;

    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(path, RecursiveMode::Recursive)?;
    // Block forever, printing out events as they come in
    for res in rx {
        match res {
            Ok(event) => {
                let s = get_data_from_file(path);
                // we'll do something with the score later.
                let emotional_state = get_emotional_state(&s, &analyzer);
                let image_to_show = state_to_image_file.get(&emotional_state).unwrap();
                println!("Should show {:?}", image_to_show);
                swap_obs_image_to(&image_source_id, &image_to_show, &client).await?;
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn get_data_from_file(path: &Path) -> String {
    let s = fs::read_to_string(path).expect("this is a bad idea");
    return s
}

fn get_ip_and_obs_password() -> (String, String) {
    // TODO take this from cli like the path or something I suppose.
    return (String::from("10.0.0.182"), String::from("password"));
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