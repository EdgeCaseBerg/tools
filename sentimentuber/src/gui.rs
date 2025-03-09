use eframe::egui;
use std::sync::mpsc;
use crate::rules::SentimentAction;
use crate::get_full_path;

pub struct AvatarGreenScreen {
    current_file_path: String,
    new_image_receiver: mpsc::Receiver<SentimentAction>,
    pub new_image_sender: mpsc::Sender<SentimentAction>
}

impl AvatarGreenScreen {
    pub fn new(path: String) -> Self {
        let (new_image_sender, new_image_receiver) = mpsc::channel();
        Self {
            current_file_path: path,
            new_image_sender,
            new_image_receiver
        }
    }
}

impl eframe::App for AvatarGreenScreen {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Note that if we DONT request a repaint after some time,
        // then we'll never see any repaints unless we hover over the window
        // with our mouse or similar. Since we want this to change in the
        // background so that OBS can capture it, we need to do this.
        // Requesting 1s from now is better than calling request_paint()
        // every frame and results in CPU% of like 0.00001 versus 2.7% or so
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
        if let Ok(new_action) = self.new_image_receiver.try_recv() {
            if let Ok(new_file) = get_full_path(&new_action.show) {
                self.current_file_path =  new_file;
                // This may not be neccesary, but also know that it never 
                // worked to get around the issue that the request_repaint_after
                // is solving above.
                ctx.request_repaint();
            }
        }

        egui_extras::install_image_loaders(&ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            let image = egui::Image::from_uri(format!("file://{0}", self.current_file_path)).bg_fill(egui::Color32::GREEN);
            ui.add(image);
        });
    }
}
