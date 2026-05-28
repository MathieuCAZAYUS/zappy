wit_bindgen::generate!({
    path: "../../wit",
    world: "ui-world",
});

use std::sync::Mutex;

use crate::local::zappy::{
    graphic::{Color, RectCmd},
    host_api::{host_log, host_subscribe},
};
static CROSSHAIR_ACTIVE: Mutex<bool> = Mutex::new(false);
static INITIALIZED: Mutex<bool> = Mutex::new(false);

struct Module;

fn init_module() {
    let mut initialized = INITIALIZED.lock().unwrap();
    if *initialized {
        return;
    }

    host_subscribe("crosshair:update_crosshair");

    *initialized = true;
}

impl Guest for Module {
    fn serialize() -> Vec<u8> {
        let active = CROSSHAIR_ACTIVE.lock().unwrap();
        bincode::serialize(&*active).unwrap_or_default()
    }

    fn deserialize(state_bytes: Vec<u8>) {
        if let Ok(decoded) = bincode::deserialize::<bool>(&state_bytes) {
            let mut active = CROSSHAIR_ACTIVE.lock().unwrap();
            *active = decoded;
        }
    }

    fn handle_event(event_name: String, payload: String) {
        if event_name == "crosshair:update_crosshair" {
            host_log(format!("crosshair updated: {payload}").as_str());
        }
    }

    fn handle_input(_state: InputState) {}

    fn run_command(cmd: String, args: Vec<String>) -> ResponseCommand {
        match cmd.as_str() {
            "display_crosshair" => {
                if let Some(arg) = args.first()
                    && let Ok(n) = arg.parse::<u8>()
                    && n <= 1
                {
                    let mut active = CROSSHAIR_ACTIVE.lock().unwrap();
                    *active = n == 1;

                    ResponseCommand::Ok
                } else {
                    ResponseCommand::BadArgument
                }
            }
            _ => ResponseCommand::Unknown,
        }
    }

    fn update_module(_time: f32, _dt: f32, w: f32, h: f32) -> Vec<RenderCommand> {
        init_module();
        let active = *CROSSHAIR_ACTIVE.lock().unwrap();
        if !active {
            return Vec::new();
        }

        let size = (w / 100.0, h / 100.0);

        let cmds = vec![RenderCommand::Rect(RectCmd {
            x: (w / 2.0) - (size.0 / 2.0),
            y: (h / 2.0) - (size.1 / 2.0),
            w: size.0,
            h: size.1,
            color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 50,
            },
            rotation: 0.0,
        })];

        cmds
    }

    fn get_commands() -> Vec<CommandDesc> {
        vec![CommandDesc {
            module: "crosshair".to_string(),
            name: "display_crosshair".to_string(),
            options: "<0|1>".to_string(),
            help: "Show / hide crosshair".to_string(),
        }]
    }

    fn accept_log(_segments: Vec<TextSegment>) {}
}

export!(Module);
