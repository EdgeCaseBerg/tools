use raylib::prelude::*;

const XBOX_ALIAS_1: &str = "xbox";
const XBOX_ALIAS_2: &str = "xbox";
const PS_ALIAS: &str = "playstation";

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 450;

const DEADZONE_LEFT_STICK_X: f32 = 0.1;
const DEADZONE_LEFT_STICK_Y: f32 = 0.1;
const DEADZONE_LEFT_TRIGGER: f32 = -0.9;
const DEADZONE_RIGHT_STICK_X: f32 = 0.1;
const DEADZONE_RIGHT_STICK_Y: f32 = 0.1;
const DEADZONE_RIGHT_TRIGGER: f32 = -0.9;

const TARGET_FPS: u32 = 60;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Game Pad output")
        .msaa_4x()
        .build();

    rl.set_target_fps(TARGET_FPS);

    let xbox_texture = rl.load_texture(&thread, "resources/xbox.png").expect("Cannot run program if gamepad texture (xbox) missing");
    let ps3_texture  = rl.load_texture(&thread, "resources/ps3.png").expect("Cannot run program if gamepad texture (ps3) missing");

    // Original example uses left and right to swap pad to display using this.
    let gamepad_to_display = 0;
     
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
         
        if d.is_gamepad_available(gamepad_to_display) {
            d.draw_text(&format!("Gamepad: {gamepad_to_display}"), 60, 60, 20, Color::BLACK);            
        } else {
            d.draw_text(&format!("No Gamepad: {gamepad_to_display}"), 60, 60, 20, Color::BLACK);            
        }

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }

    // We don't need to do this because the Drop implementation for Texture2D will handle this
    // rl.unload_texture(&thread, xbox_texture.make_weak());
    // rl.unload_texture(&thread, ps3_texture.make_weak());
}