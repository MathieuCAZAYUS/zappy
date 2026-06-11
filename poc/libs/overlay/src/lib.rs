wit_bindgen::generate!({
    path: "../../wit",
    world: "ui-world",
});

use std::sync::Mutex;

use crate::local::zappy::{
    graphic::{Color, RectCmd, TextCmd},
    host_api::host_get_state,
};
static OVERLAY_ACTIVE: Mutex<bool> = Mutex::new(false);

struct Module;

impl Guest for Module {
    fn serialize() -> Vec<u8> {
        let active = OVERLAY_ACTIVE.lock().unwrap();
        bincode::serialize(&*active).unwrap_or_default()
    }

    fn deserialize(state_bytes: Vec<u8>) {
        if let Ok(decoded) = bincode::deserialize::<bool>(&state_bytes) {
            let mut active = OVERLAY_ACTIVE.lock().unwrap();
            *active = decoded;
        }
    }

    fn handle_event(_event_name: String, _payload: String) {}

    fn handle_input(_state: InputState) {}

    fn run_command(cmd: String, args: Vec<String>) -> ResponseCommand {
        match cmd.as_str() {
            "display_info" => {
                if let Some(arg) = args.first()
                    && let Ok(n) = arg.parse::<u8>()
                    && n <= 1
                {
                    let mut active = OVERLAY_ACTIVE.lock().unwrap();
                    *active = n == 1;

                    ResponseCommand::Ok
                } else {
                    ResponseCommand::BadArgument
                }
            }
            _ => ResponseCommand::Unknown,
        }
    }

    fn update_module(_time: f32, dt: f32, w: f32, _h: f32) -> Vec<RenderCommand> {
        let active = *OVERLAY_ACTIVE.lock().unwrap();
        if !active {
            return Vec::new();
        }

        let fps = if dt > 0.0 { (1.0 / dt) as u32 } else { 0 };
        let frametime = dt * 1000.0;
        let ping = 0.0; // TODO:

        let bg_name = if let Some(bytes) = host_get_state("bg:current") {
            String::from_utf8_lossy(&bytes).into_owned()
        } else {
            "Unknown".to_string()
        };

        let time_mode = if let Some(ts) = host_get_state("sys:timescale")
            && ts.len() == 4
        {
            f32::from_le_bytes(ts[..4].try_into().unwrap()).to_string()
        } else {
            "1.0".to_string()
        };

        let rect_w = 200.0;
        let rect_h = 105.0;
        let x = w - rect_w - 20.0;
        let y = 20.0;

        vec![
            RenderCommand::Rect(RectCmd {
                x,
                y,
                w: rect_w,
                h: rect_h,
                color: Color {
                    r: 15,
                    g: 15,
                    b: 20,
                    a: 220,
                },
                rotation: 0.0,
            }),
            RenderCommand::Text(TextCmd {
                text: format!("FPS: {fps}"),
                x: x + 15.0,
                y: y + 30.0,
                size: 24.0,
                color: Color {
                    r: 200,
                    g: 200,
                    b: 200,
                    a: 255,
                },
            }),
            RenderCommand::Text(TextCmd {
                text: format!("Frame: {frametime:.1}ms"),
                x: x + 15.0,
                y: y + 60.0,
                size: 20.0,
                color: Color {
                    r: 200,
                    g: 200,
                    b: 200,
                    a: 255,
                },
            }),
            RenderCommand::Text(TextCmd {
                text: format!("Ping: {ping}ms"),
                x: x + 15.0,
                y: y + 90.0,
                size: 20.0,
                color: Color {
                    r: 100,
                    g: 200,
                    b: 255,
                    a: 255,
                },
            }),
            RenderCommand::Text(TextCmd {
                text: format!("Background: {bg_name}"),
                x: x + 15.0,
                y: y + 120.0,
                size: 20.0,
                color: Color {
                    r: 200,
                    g: 200,
                    b: 200,
                    a: 255,
                },
            }),
            RenderCommand::Text(TextCmd {
                text: format!("Time Scale: {time_mode}"),
                x: x + 15.0,
                y: y + 150.0,
                size: 20.0,
                color: Color {
                    r: 255,
                    g: 100,
                    b: 100,
                    a: 255,
                },
            }),
        ]
    }

    fn get_commands() -> Vec<CommandDesc> {
        vec![CommandDesc {
            module: "overlay".to_string(),
            name: "display_info".to_string(),
            options: "<0|1>".to_string(),
            help: "Show / Hide performance overlay".to_string(),
        }]
    }

    fn accept_log(_segments: Vec<TextSegment>) {}
}

export!(Module);
