use raylib::prelude::*;

const XBOX_ALIAS_1: &str = "xbox";
const XBOX_ALIAS_2: &str = "xbox";
const PS_ALIAS: &str = "playstation";

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .build();
     
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
         
        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}