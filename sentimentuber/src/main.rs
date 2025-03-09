//! # SentimentTuber
//!
//! `sentimenttuber` is a tool to read in a text file produced by
//! OBS's localvocal plugin and determine an image to show for an
//! avatar based on keywords and sentiment.

use std::sync::mpsc;
use std::thread;
use sentimentuber::cli::Config;
use sentimentuber::start_obs_controller_on_thread;
use sentimentuber::regularly_send_tick_with;
use sentimentuber::emit_file_contents_on_change_forever;
use sentimentuber::emit_action_on_sentiment;

mod fltk_app;

fn main() -> anyhow::Result<()> {
    let config = Config::parse_env();
    let app = fltk_app::MyApp::new(&config.default_action);

    let (context_sender, context_receiver) = mpsc::channel();
    // let (obs_sender, sentiment_reciever) = mpsc::channel();
    // let obs_sender = start_obs_controller_on_thread(&config)?;
    regularly_send_tick_with(context_sender.clone(), config.context_retention_seconds);
    emit_action_on_sentiment(&config, context_receiver, app.sender.clone());
    
    thread::spawn(move || {
       emit_file_contents_on_change_forever(&config, context_sender); 
    });
    app.run();
    
    Ok(())
}