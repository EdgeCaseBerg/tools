use raylib::prelude::*;

const XBOX_ALIAS_1: &str = "xbox";
const XBOX_ALIAS_2: &str = "xbox";
const PS_ALIAS: &str = "playstation";

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 450;


fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Game Pad output")
        .msaa_4x()
        .build();

    let xbox_texture = rl.load_texture(&thread, "resources/xbox.png");
    let ps3_texture  = rl.load_texture(&thread, "resources/ps3.png");
     
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
         
        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}