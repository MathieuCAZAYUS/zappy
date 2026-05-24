wit_bindgen::generate!({
    path: "../../wit",
    world: "cube-world",
});

use std::{f32::consts::PI, sync::Mutex};

use serde::{Deserialize, Serialize};

use crate::local::zappy::{
    graphic::{Color, RectCmd, SpriteCmd},
    host_api::emit_event,
};

#[derive(Serialize, Deserialize, Default)]
struct CubeData {
    offsets: (f32, f32),
    clockwise: bool,
}

static CUBE_STATE: Mutex<CubeData> = Mutex::new(CubeData {
    offsets: (200.0, 200.0),
    clockwise: false,
});

#[derive(Default)]
struct Rotation {
    center: (f32, f32),
    direction: f32,
    distance: f32,
    speed: f32,
    angle_offset: f32,
    angle: f32,
    x: f32,
    y: f32,
}

impl Rotation {
    fn new(
        center: (f32, f32),
        direction: f32,
        distance: f32,
        speed: f32,
        angle_offset: f32,
    ) -> Self {
        Rotation {
            center,
            direction,
            distance,
            speed,
            angle_offset,
            angle: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }

    fn calc_angle(&mut self, time: f32, speed_mult: f32) {
        self.angle = (time * self.speed * speed_mult * self.direction) + self.angle_offset;
    }

    fn calc_pos(&mut self, size: (f32, f32)) {
        self.x = self.center.0 + self.angle.cos() * self.distance
            - (size.0 / 2.0) * self.angle.cos()
            + (size.1 / 2.0) * self.angle.sin();
        self.y = self.center.1 + self.angle.sin() * self.distance
            - (size.0 / 2.0) * self.angle.sin()
            - (size.1 / 2.0) * self.angle.cos();
    }
}

struct Module;

impl Guest for Module {
    fn serialize() -> Vec<u8> {
        let state = CUBE_STATE.lock().unwrap();
        bincode::serialize(&*state).unwrap_or_default()
    }

    fn deserialize(state_bytes: Vec<u8>) {
        if let Ok(decoded) = bincode::deserialize::<CubeData>(&state_bytes) {
            let mut state = CUBE_STATE.lock().unwrap();
            *state = decoded;
        }
    }

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
                let mut state = CUBE_STATE.lock().unwrap();
                state.clockwise = !state.clockwise;

                let msg = format!("Cube changed direction! Clockwise: {}", state.clockwise);
                emit_event("console_log", &msg);

                ResponseCommand::Ok
            }
            _ => ResponseCommand::Unknown,
        }
    }

    fn handle_event(event_name: String, payload: String) {
        if event_name == "teleport_cube" {
            let mut data = CUBE_STATE.lock().unwrap();
            if payload == "reset" {
                data.offsets = (200.0, 200.0);
                emit_event(
                    "console_log",
                    "Cube intercepted 'teleport_cube': Reset to center.",
                );
            }
        }
    }

    fn handle_input(state: InputState) {
        if matches!(state.context, EngineContext::Gameplay) {
            let mut data = CUBE_STATE.lock().unwrap();

            for action in state.actions {
                match action {
                    InputAction::MoveLeft => data.offsets.0 -= 10.0,
                    InputAction::MoveRight => data.offsets.0 += 10.0,
                    InputAction::MoveUp => data.offsets.1 -= 10.0,
                    InputAction::MoveDown => data.offsets.1 += 10.0,
                    _ => {}
                }
            }
        }
    }

    fn update_module(time: f32, _dt: f32, _w: f32, _h: f32) -> Vec<RenderCommand> {
        let data = CUBE_STATE.lock().unwrap();
        let center = data.offsets;
        let direction = if data.clockwise { 1.0 } else { -1.0 };
        let base_speed = 25.0;

        let distance = 100.0;
        let speed_mult = 0.5;

        let size = (5.0, 25.0);

        let mut rot_1 = Rotation::new(center, direction, distance, base_speed, 0.0);
        let mut rot_2 = Rotation::new(center, direction, distance, base_speed, PI);

        rot_1.calc_angle(time, speed_mult);
        rot_2.calc_angle(time, speed_mult);

        rot_1.calc_pos(size);
        rot_2.calc_pos(size);

        let mut rot_3 = Rotation::new(
            center,
            -direction,
            distance / 2.0,
            base_speed * 2.0,
            PI / 2.0,
        );
        let mut rot_4 = Rotation::new(
            center,
            -direction,
            distance / 2.0,
            base_speed * 2.0,
            PI * 1.5,
        );

        rot_3.calc_angle(time, speed_mult);
        rot_4.calc_angle(time, speed_mult);

        rot_3.calc_pos((size.1, size.0));
        rot_4.calc_pos((size.1, size.0));

        let skyblue = Color {
            r: 135,
            g: 206,
            b: 235,
            a: 255,
        };

        let neon_purple = Color {
            r: 148,
            g: 87,
            b: 235,
            a: 255,
        };

        vec![
            RenderCommand::Rect(RectCmd {
                x: rot_1.x,
                y: rot_1.y,
                w: size.0,
                h: size.1,
                rotation: rot_1.angle,
                color: skyblue,
            }),
            RenderCommand::Rect(RectCmd {
                x: rot_2.x,
                y: rot_2.y,
                w: size.0,
                h: size.1,
                rotation: rot_2.angle,
                color: skyblue,
            }),
            RenderCommand::Rect(RectCmd {
                x: rot_3.x,
                y: rot_3.y,
                w: size.1,
                h: size.0,
                rotation: rot_3.angle,
                color: neon_purple,
            }),
            RenderCommand::Rect(RectCmd {
                x: rot_4.x,
                y: rot_4.y,
                w: size.1,
                h: size.0,
                rotation: rot_4.angle,
                color: neon_purple,
            }),
            RenderCommand::Rect(RectCmd {
                x: data.offsets.0 - 2.5,
                y: data.offsets.1 - 2.5,
                w: 5.0,
                h: 5.0,
                color: Color {
                    r: 140,
                    g: 219,
                    b: 26,
                    a: 255,
                },
                rotation: 0.0,
            }),
            RenderCommand::Sprite(SpriteCmd {
                path: "player.png".to_string(),
                x: data.offsets.0 + 40.0,
                y: data.offsets.1 + 40.0,
                w: 200.0,
                h: 300.0,
                source: None,
                color: Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
            }),
        ]
    }
}

export!(Module);
