wit_bindgen::generate!({
    path: "../../wit",
    world: "cube-world",
});

use std::sync::Mutex;

use crate::local::zappy::graphic::{Color, RectCmd};
static OFFSETS: Mutex<(f32, f32)> = Mutex::new((200.0, 200.0));
static CLOCKWISE: Mutex<bool> = Mutex::new(false);

struct Module;

impl Guest for Module {
    fn get_commands() -> Vec<CommandDesc> {
        vec![CommandDesc {
            module: "cube".to_string(),
            name: "invert_cube".to_string(),
            options: "".to_string(),
            help: "Invert cube direction".to_string(),
        }]
    }

    fn run_command(cmd: String, _args: Vec<String>) -> ResponseCommand {
        match cmd.as_str() {
            "invert_cube" => {
                let mut clockwise = CLOCKWISE.lock().unwrap();
                *clockwise = !*clockwise;
                ResponseCommand::Ok
            }
            _ => ResponseCommand::Unknown,
        }
    }

    fn handle_input(event: KeyEvent) -> bool {
        if let KeyEvent::Pressed(key) = event {
            let mut offsets = OFFSETS.lock().unwrap();
            match key.as_str() {
                "Left" => {
                    offsets.0 -= 10.0;
                    true
                }
                "Right" => {
                    offsets.0 += 10.0;
                    true
                }
                "Up" => {
                    offsets.1 -= 10.0;
                    true
                }
                "Down" => {
                    offsets.1 += 10.0;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn update_module(time: f32, _dt: f32, _w: f32, _h: f32) -> Vec<RenderCommand> {
        let offsets = OFFSETS.lock().unwrap();
        let speed = 2.0;
        let time = if *CLOCKWISE.lock().unwrap() {
            time
        } else {
            -time
        };
        let angle = time * speed;

        vec![RenderCommand::Rect(RectCmd {
            x: angle.cos() * 100.0 + offsets.0,
            y: angle.sin() * 100.0 + offsets.1,
            w: 25.0,
            h: 25.0,
            rotation: angle,
            color: Color {
                r: 135,
                g: 206,
                b: 235,
                a: 255,
            },
        })]
    }
}

export!(Module);
