//! # SentimentTuber
//!
//! `sentimenttuber` is a tool to read in a text file produced by
//! OBS's localvocal plugin and determine an image to show for an
//! avatar based on keywords and sentiment.

use eframe::egui;
use std::sync::mpsc;
use sentimentuber::cli::Config;
use sentimentuber::regularly_send_tick_with;
use sentimentuber::emit_file_contents_on_change_forever;
use sentimentuber::emit_action_on_sentiment;
use sentimentuber::get_full_path;
use sentimentuber::gui::AvatarGreenScreen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse_env();
    let (context_sender, context_receiver) = mpsc::channel();
    let starting_image = get_full_path(&config.default_action)?;
    let app = AvatarGreenScreen::new(starting_image);
    let to_gui_sender = app.new_image_sender.clone();
    regularly_send_tick_with(context_sender.clone(), config.context_retention_seconds);
    emit_file_contents_on_change_forever(config.clone(), context_sender);
    emit_action_on_sentiment(&config, context_receiver, to_gui_sender);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 1000.0]),
        ..Default::default()
    };
    let handle = eframe::run_native(
        "PNGTuber",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    );
    Ok(handle?)
}

