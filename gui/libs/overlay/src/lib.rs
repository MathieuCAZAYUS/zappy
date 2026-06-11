wit_bindgen::generate!({
    path: "../../wit",
    world: "ui-world",
});

use std::sync::Mutex;

use crate::local::zappy::{
    graphic::{Color, TextAlign, TextCmd},
    host_api::host_subscribe,
};
static OVERLAY_ACTIVE: Mutex<bool> = Mutex::new(false);
static DYNAMIC_METRICS: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
static INITIALIZED: Mutex<bool> = Mutex::new(false);

struct Module;

fn init_module() {
    let mut initialized = INITIALIZED.lock().unwrap();
    if *initialized {
        return;
    }

    host_subscribe("overlay:update_metric");

    *initialized = true;
}

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

    fn handle_event(event_name: String, payload: String) {
        if event_name == "overlay:update_metric" {
            let mut metrics = DYNAMIC_METRICS.lock().unwrap();
            if let Some((key, value)) = payload.split_once(':') {
                metrics.retain(|(k, _)| k != key);
                metrics.push((key.to_string(), value.to_string()));
            } else {
                metrics.retain(|(k, _)| k != "flop");
                metrics.push(("flop".to_string(), "big flop".to_string()));
            }
        }
    }

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
        init_module();
        let active = *OVERLAY_ACTIVE.lock().unwrap();
        if !active {
            return Vec::new();
        }

        let metrics = DYNAMIC_METRICS.lock().unwrap();

        let font_size = 15.0;
        let padding_right = 2.0;
        let padding_top = 20.0;
        let line_height = 20.0;

        let mut cmds = vec![RenderCommand::Text(TextCmd {
            text: format!("FPS: {}", if dt > 0.0 { (1.0 / dt) as u32 } else { 0 }),
            x: w - padding_right,
            y: padding_top,
            size: font_size,
            color: Color {
                r: 200,
                g: 200,
                b: 200,
                a: 255,
            },
            align: TextAlign::Right,
        })];

        for (i, (key, value)) in metrics.iter().enumerate() {
            cmds.push(RenderCommand::Text(TextCmd {
                text: format!("{key}: {value}"),
                x: w - padding_right,
                y: padding_top + ((i + 1) as f32 * line_height),
                size: font_size,
                color: Color {
                    r: 100,
                    g: 200,
                    b: 255,
                    a: 255,
                },
                align: TextAlign::Right,
            }));
        }

        cmds
    }

    fn get_commands() -> Vec<CommandDesc> {
        vec![CommandDesc {
            module: "overlay".to_string(),
            name: "display_info".to_string(),
            options: "<0|1>".to_string(),
            help: "Show / hide overlay".to_string(),
        }]
    }

    fn accept_log(_segments: Vec<TextSegment>) {}
}

export!(Module);
