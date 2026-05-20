wit_bindgen::generate!({
    path: "../wit",
    world: "plugin-world"
});

use std::sync::Mutex;
static CONSOLE_OPENED: Mutex<bool> = Mutex::new(false);

struct Plugin;

impl Guest for Plugin {
    fn handle_input(event: KeyEvent) -> bool {
        let mut opened = CONSOLE_OPENED.lock().unwrap();

        match event {
            KeyEvent::Pressed(key) => {
                if key == "F1" {
                    *opened = !*opened;
                    return true;
                }

                if *opened {
                    if key == "Enter" { true } else { true }
                } else {
                    false
                }
            }
            KeyEvent::Released(_) => *opened,
        }
    }

    fn update_cube(_state: CubeState) -> RenderCommand {
        RenderCommand {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
        }
    }
}

export!(Plugin);
