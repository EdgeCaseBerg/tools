use raylib::prelude::*;

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

    // Raylib supports up to 4 gamepads, if you just have one connected, it's ID 0.
    let gamepad_to_display = 0;
     
    while !rl.window_should_close() {
        // Acquire the sticks and triggers
        let mut left_stick_x = rl.get_gamepad_axis_movement(gamepad_to_display, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
        let mut left_stick_y = rl.get_gamepad_axis_movement(gamepad_to_display, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
        let mut right_stick_x = rl.get_gamepad_axis_movement(gamepad_to_display, GamepadAxis::GAMEPAD_AXIS_RIGHT_X);
        let mut right_stick_y = rl.get_gamepad_axis_movement(gamepad_to_display, GamepadAxis::GAMEPAD_AXIS_RIGHT_Y);
        let mut left_trigger = rl.get_gamepad_axis_movement(gamepad_to_display, GamepadAxis::GAMEPAD_AXIS_LEFT_TRIGGER);
        let mut right_trigger = rl.get_gamepad_axis_movement(gamepad_to_display, GamepadAxis::GAMEPAD_AXIS_RIGHT_TRIGGER);

        // Then filter out noise via deadzones
        if -DEADZONE_LEFT_STICK_X < left_stick_x && left_stick_x < DEADZONE_LEFT_STICK_X {
            left_stick_x = 0.0;
        }
        if -DEADZONE_LEFT_STICK_Y < left_stick_y && left_stick_y < DEADZONE_LEFT_STICK_Y {
            left_stick_y = 0.0;
        }
        if -DEADZONE_RIGHT_STICK_X < right_stick_x && right_stick_x < DEADZONE_RIGHT_STICK_X {
            right_stick_x = 0.0;
        }
        if -DEADZONE_RIGHT_STICK_Y < right_stick_y && right_stick_y < DEADZONE_RIGHT_STICK_Y {
            right_stick_y = 0.0;
        }
        if left_trigger < DEADZONE_LEFT_TRIGGER {
            left_trigger = -1.0;
        }
        if right_trigger < DEADZONE_RIGHT_TRIGGER {
            right_trigger = -1.0;
        }

        let left_gamepad_color: Color = if rl.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_LEFT_THUMB) {
            Color::RED
        } else {
            Color::BLACK
        };
        let right_gamepad_color: Color = if rl.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_RIGHT_THUMB) {
            Color::RED
        } else {
            Color::BLACK
        };

        let mut d = rl.begin_drawing(&thread);
        if d.is_gamepad_available(gamepad_to_display) { 
            d.draw_text(&format!("Gamepad: {gamepad_to_display}"), 60, 60, 12, Color::BLACK);            
        } else {
            d.draw_text(&format!("No Gamepad: {gamepad_to_display}"), 60, 60, 12, Color::BLACK);            
        }

        // The background texture
        d.draw_texture(&xbox_texture, 0, 0, Color::DARKGRAY);

        // select and start
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_MIDDLE_LEFT) {
            d.draw_circle(436, 150, 9.0, Color::RED);
        }
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_MIDDLE_RIGHT) {
            d.draw_circle(352, 150, 9.0, Color::RED);
        }

        // face buttons
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_LEFT) {
            d.draw_circle(501, 151, 15.0, Color::BLUE);
        }
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_DOWN) {
            d.draw_circle(536, 187, 15.0, Color::LIME);
        }
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_RIGHT) {
            d.draw_circle(572, 151, 15.0, Color::MAROON);
        }
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_RIGHT_FACE_UP) {
            d.draw_circle(536, 115, 15.0, Color::GOLD);
        }

        // Draw buttons: d-pad
        d.draw_rectangle(317, 202, 19, 71, Color::BLACK);
        d.draw_rectangle(293, 228, 69, 19, Color::BLACK);
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_UP) {
            d.draw_rectangle(317, 202, 19, 26, Color::RED);
        }
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_DOWN) {
            d.draw_rectangle(317, 202 + 45, 19, 26, Color::RED);
        }
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_LEFT) {
            d.draw_rectangle(292, 228, 25, 19, Color::RED);
        }
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_LEFT_FACE_RIGHT) {
            d.draw_rectangle(292 + 44, 228, 26, 19, Color::RED);
        }

        // Left Joystick
        d.draw_circle(259, 152, 39.0, Color::BLACK);
        d.draw_circle(259, 152, 34.0, Color::LIGHTGRAY);
        d.draw_circle(
            259 + (left_stick_x * 20.0) as i32,
            152 + (left_stick_y * 20.0) as i32, 
            25.0, left_gamepad_color
        );

        // Right Joystick
        d.draw_circle(461, 237, 38.0, Color::BLACK);
        d.draw_circle(461, 237, 33.0, Color::LIGHTGRAY);
        d.draw_circle(
            461 + (right_stick_x * 20.0) as i32,
            237 + (right_stick_y * 20.0) as i32, 
            25.0, right_gamepad_color
        );

        // left and right triggers
        d.draw_rectangle(170, 30, 15, 70, Color::GRAY);
        d.draw_rectangle(604, 30, 15, 70, Color::GRAY);
        d.draw_rectangle(170, 30, 15, ((1.0 + left_trigger) /2.0 * 70.0) as i32, Color::RED);
        d.draw_rectangle(604, 30, 15, ((1.0 + right_trigger) /2.0 * 70.0) as i32, Color::RED);

        // Draw buttons: left-right shoulder buttons
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_LEFT_TRIGGER_1) {
            d.draw_circle(239, 82, 20.0, Color::RED);
        }
        if d.is_gamepad_button_down(gamepad_to_display, GamepadButton::GAMEPAD_BUTTON_RIGHT_TRIGGER_1) {
            d.draw_circle(557, 82, 20.0, Color::RED);
        }

        d.clear_background(Color::WHITE);
    }
}
