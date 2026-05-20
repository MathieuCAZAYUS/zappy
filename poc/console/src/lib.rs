wit_bindgen::generate!({
    path: "../wit",
    world: "plugin-world"
});

use std::sync::Mutex;
static CONSOLE: Mutex<ConsoleState> = Mutex::new(ConsoleState {
    opened: false,
    input: String::new(),
    logs: Vec::new(),
});

struct ConsoleState {
    opened: bool,
    input: String,
    logs: Vec<String>,
}

struct Plugin;

impl Guest for Plugin {
    fn handle_input(event: KeyEvent) -> bool {
        let mut state = CONSOLE.lock().unwrap();

        match event {
            KeyEvent::Pressed(key) => {
                if key == "F1" {
                    state.opened = !state.opened;
                    return true;
                }

                if state.opened {
                    match key.as_str() {
                        "Enter" => {
                            let trimmed = state.input.trim().to_string();
                            if !trimmed.is_empty() {
                                state.logs.push(format!("> {trimmed}"));

                                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                                let cmd = parts[0].to_string();
                                let args: Vec<String> =
                                    parts[1..].iter().map(|s| s.to_string()).collect();

                                let response = host_system_command(&cmd, &args);
                                for line in response.lines() {
                                    state.logs.push(line.to_string());
                                }
                                state.input.clear();
                            }
                        }
                        "Backspace" => {
                            state.input.pop();
                        }
                        _ => {}
                    }
                    return true;
                }
            }
            KeyEvent::CharInput(c) => {
                if state.opened {
                    state.input.push_str(&c);
                    return true;
                }
            }
        }
        state.opened
    }

    fn update_plugin(_time: f32) -> Vec<RenderCommand> {
        let state = CONSOLE.lock().unwrap();
        if !state.opened {
            return Vec::new();
        }

        let mut cmds = Vec::new();

        cmds.push(RenderCommand::Rect(RectCmd {
            x: 0.0,
            y: 0.0,
            w: 800.0,
            h: 320.0,
            color: Color {
                r: 10,
                g: 10,
                b: 15,
                a: 200,
            },
            rotation: 0.0,
        }));

        let start_y = 25.0;
        let line_height = 20.0;
        let max_lines = 12;
        let display_logs = if state.logs.len() > max_lines {
            &state.logs[state.logs.len() - max_lines..]
        } else {
            &state.logs
        };

        for (i, line) in display_logs.iter().enumerate() {
            cmds.push(RenderCommand::Text(TextCmd {
                text: line.clone(),
                x: 15.0,
                y: start_y + (i as f32) * line_height,
                size: 18.0,
                color: Color {
                    r: 220,
                    g: 220,
                    b: 225,
                    a: 255,
                },
            }));
        }

        let input_y = start_y + (max_lines as f32) * line_height + 10.0;
        cmds.push(RenderCommand::Text(TextCmd {
            text: format!("> {}", state.input),
            x: 15.0,
            y: input_y,
            size: 20.0,
            color: Color {
                r: 50,
                g: 240,
                b: 100,
                a: 255,
            },
        }));

        cmds
    }

    fn get_commands() -> Vec<CommandDesc> {
        Vec::new()
    }

    fn accept_log(msg: String) {
        let mut state = CONSOLE.lock().unwrap();
        for line in msg.lines() {
            state.logs.push(line.to_string());
        }
    }
}

export!(Plugin);
