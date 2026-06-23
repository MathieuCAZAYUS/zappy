wit_bindgen::generate!({
    path: "../../wit",
    world: "ui-world"
});

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::local::zappy::{
    graphic::{Color, RectCmd, TextAlign, TextCmd},
    host_api::host_system_command,
};

#[derive(Serialize, Deserialize)]
struct SerializableColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Serialize, Deserialize)]
struct SerializableSegment {
    text: String,
    color: SerializableColor,
}

#[derive(Serialize, Deserialize)]
struct ConsoleSerializableState {
    input: String,
    logs: Vec<Vec<SerializableSegment>>,
    history: Vec<String>,
}

static CONSOLE: Mutex<ConsoleState> = Mutex::new(ConsoleState {
    opened: false,
    input: String::new(),
    logs: Vec::new(),
    history: Vec::new(),
    history_index: None,
    scroll_offset: 0,
});

struct ConsoleState {
    opened: bool,
    input: String,
    logs: Vec<Vec<TextSegment>>,
    history: Vec<String>,
    history_index: Option<usize>,
    scroll_offset: usize,
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

struct Module;

impl Guest for Module {
    fn serialize() -> Vec<u8> {
        let state = CONSOLE.lock().unwrap();

        let serializable_logs = state
            .logs
            .iter()
            .map(|line| {
                line.iter()
                    .map(|seg| SerializableSegment {
                        text: seg.text.clone(),
                        color: SerializableColor {
                            r: seg.color.r,
                            g: seg.color.g,
                            b: seg.color.b,
                            a: seg.color.a,
                        },
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let dump = ConsoleSerializableState {
            input: state.input.clone(),
            logs: serializable_logs,
            history: state.history.clone(),
        };

        bincode::serialize(&dump).unwrap_or_default()
    }

    fn deserialize(state_bytes: Vec<u8>) {
        if let Ok(dump) = bincode::deserialize::<ConsoleSerializableState>(&state_bytes) {
            let mut state = CONSOLE.lock().unwrap();
            state.input = dump.input;
            state.history = dump.history;

            state.logs = dump
                .logs
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|seg| TextSegment {
                            text: seg.text.clone(),
                            color: Color {
                                r: seg.color.r,
                                g: seg.color.g,
                                b: seg.color.b,
                                a: seg.color.a,
                            },
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
        }
    }

    fn handle_event(event_name: String, payload: String) {
        if event_name == "console_log" {
            let mut console = CONSOLE.lock().unwrap();

            let colored_msg = vec![
                TextSegment {
                    text: "[BUS] ".to_string(),
                    color: Color {
                        r: 148,
                        g: 87,
                        b: 235,
                        a: 255,
                    },
                },
                TextSegment {
                    text: payload,
                    color: Color {
                        r: 100,
                        g: 100,
                        b: 100,
                        a: 255,
                    },
                },
            ];

            let split_lines = split_segments_by_lines(colored_msg);
            console.logs.extend(split_lines);
        }
    }

    fn handle_input(state: InputState) {
        let mut console = CONSOLE.lock().unwrap();

        console.opened = matches!(state.context, EngineContext::UiConsole);

        if !console.opened {
            return;
        }

        for c in state.raw_chars.chars() {
            console.input.push(c);
            console.scroll_offset = 0;
        }

        for action in state.actions {
            match action {
                InputAction::Confirm => {
                    let trimmed = console.input.trim().to_string();
                    if !trimmed.is_empty() {
                        console.history.push(trimmed.clone());
                        console.history_index = None;
                        console.scroll_offset = 0;

                        console.logs.push(vec![TextSegment {
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
                        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

                        let response = host_system_command(&cmd, &args);
                        let split_lines = split_segments_by_lines(response);
                        console.logs.extend(split_lines);
                        console.input.clear();
                    }
                }
                InputAction::Delete => {
                    console.input.pop();
                }
                InputAction::NavigateUp if !console.history.is_empty() => {
                    let idx = match console.history_index {
                        Some(i) => i.saturating_sub(1),
                        None => console.history.len() - 1,
                    };
                    console.history_index = Some(idx);
                    console.input = console.history[idx].clone();
                }
                InputAction::NavigateDown if console.history_index.is_some() => {
                    let idx = console.history_index.unwrap();
                    if idx + 1 < console.history.len() {
                        console.history_index = Some(idx + 1);
                        console.input = console.history[idx + 1].clone();
                    } else {
                        console.history_index = None;
                        console.input.clear();
                    }
                }
                InputAction::ScrollUp => {
                    console.scroll_offset += 1;
                }
                InputAction::ScrollDown => {
                    console.scroll_offset = console.scroll_offset.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    fn run_command(_cmd: String, _args: Vec<String>) -> ResponseCommand {
        ResponseCommand::Unknown
    }

    fn update_module(_time: f32, _dt: f32, w: f32, h: f32) -> Vec<RenderCommand> {
        let mut state = CONSOLE.lock().unwrap();
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

        let max_lines_f32 = (console_height - start_y) / line_height - 1.0;
        let max_lines = if max_lines_f32 > 0.0 {
            max_lines_f32 as usize
        } else {
            0
        };
        let total_logs = state.logs.len();

        let max_scroll = total_logs.saturating_sub(max_lines);
        if state.scroll_offset > max_scroll {
            state.scroll_offset = max_scroll;
        }

        let end_idx = total_logs.saturating_sub(state.scroll_offset);
        let start_idx = end_idx.saturating_sub(max_lines);

        let display_logs = &state.logs[start_idx..end_idx];

        for (i, line) in display_logs.iter().enumerate() {
            let mut current_x = 15.0;

            for segment in line {
                cmds.push(RenderCommand::Text(TextCmd {
                    text: segment.text.clone(),
                    x: current_x,
                    y: start_y + (i as f32) * line_height,
                    size: 20.0,
                    color: Color {
                        r: segment.color.r,
                        g: segment.color.g,
                        b: segment.color.b,
                        a: segment.color.a,
                    },
                    align: TextAlign::Left,
                }));
                current_x += segment.text.len() as f32 * 8.5;
            }
        }

        let input_y = start_y + (max_lines_f32) * line_height + 10.0;
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
            align: TextAlign::Left,
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
            align: TextAlign::Left,
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

export!(Module);
