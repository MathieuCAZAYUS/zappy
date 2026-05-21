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
    logs: Vec<Vec<TextSegment>>,
}

fn split_segments_by_lines(segments: Vec<TextSegment>) -> Vec<Vec<TextSegment>> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();

    for segment in segments {
        let mut current_text = String::new();

        for c in segment.text.chars() {
            match c {
                '\n' => {
                    if !current_text.is_empty() {
                        current_line.push(TextSegment {
                            text: current_text.clone(),
                            color: segment.color,
                        });
                        current_text.clear();
                    }
                    lines.push(current_line);
                    current_line = Vec::new();
                }
                '\t' => {
                    current_text.push_str("    ");
                }
                '\r' => {
                    continue;
                }
                _ => {
                    current_text.push(c);
                }
            }
        }

        if !current_text.is_empty() {
            current_line.push(TextSegment {
                text: current_text,
                color: segment.color,
            });
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
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
                                state.logs.push(vec![TextSegment {
                                    text: format!("> {trimmed}"),
                                    color: Color {
                                        r: 100,
                                        g: 100,
                                        b: 100,
                                        a: 255,
                                    },
                                }]);

                                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                                let cmd = parts[0].to_string();
                                let args: Vec<String> =
                                    parts[1..].iter().map(|s| s.to_string()).collect();

                                let response = host_system_command(&cmd, &args);
                                let split_lines = split_segments_by_lines(response);
                                state.logs.extend(split_lines);
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

    fn update_plugin(_time: f32, w: f32, h: f32) -> Vec<RenderCommand> {
        let state = CONSOLE.lock().unwrap();
        if !state.opened {
            return Vec::new();
        }

        let mut cmds = Vec::new();

        let console_height = h / 3.0;

        cmds.push(RenderCommand::Rect(RectCmd {
            x: 0.0,
            y: 0.0,
            w,
            h: console_height,
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
        let max_lines = (console_height - start_y) / line_height - 1.0;
        let display_logs = if state.logs.len() > max_lines as usize {
            &state.logs[state.logs.len() - max_lines as usize..]
        } else {
            &state.logs
        };

        for (i, line) in display_logs.iter().enumerate() {
            let mut current_x = 15.0;

            for segment in line {
                cmds.push(RenderCommand::Text(TextCmd {
                    text: segment.text.clone(),
                    x: current_x,
                    y: start_y + (i as f32) * line_height,
                    size: 18.0,
                    color: Color {
                        r: segment.color.r,
                        g: segment.color.g,
                        b: segment.color.b,
                        a: segment.color.a,
                    },
                }));
                current_x += segment.text.len() as f32 * 8.0;
            }
        }

        let input_y = start_y + (max_lines) * line_height + 10.0;
        let current_x = 15.0;
        let user = "testing".to_string();
        cmds.push(RenderCommand::Text(TextCmd {
            text: user.clone(),
            x: current_x,
            y: input_y,
            size: 20.0,
            color: Color {
                r: 50,
                g: 240,
                b: 100,
                a: 255,
            },
        }));
        cmds.push(RenderCommand::Text(TextCmd {
            text: format!(" > {}", state.input),
            x: current_x + user.len() as f32 * 8.0,
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

    fn accept_log(segments: Vec<TextSegment>) {
        let mut state = CONSOLE.lock().unwrap();
        let split_lines = split_segments_by_lines(segments);
        state.logs.extend(split_lines);
    }
}

export!(Plugin);
