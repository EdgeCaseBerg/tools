use raylib::prelude::*;

const XBOX_ALIAS_1: &str = "xbox";
const XBOX_ALIAS_2: &str = "xbox";
const PS_ALIAS: &str = "playstation";

const screen_width: u32 = 800;
const screen_height: u32 = 450;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Game Pad output")
        .build();
     
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
         
        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}