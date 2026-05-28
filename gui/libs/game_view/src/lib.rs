wit_bindgen::generate!({
    path: "../../wit",
    world: "cube-world",
});

use serde::{Deserialize, Serialize};

use crate::local::zappy::{
    graphic::{CameraCmd, Color, CubeCmd, Grid3dCmd, Line3dCmd, Vec3},
    host_api::emit_event,
};
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct SerializableCameraState {
    position: (f32, f32, f32),
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
    speed: f32,
}

struct CameraState {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
    speed: f32,
    velocity: Vec3,
    wish_dir: Vec3,
}

#[derive(Serialize, Deserialize)]
struct SerializableCubeEntity {
    pos: (f32, f32, f32),
    target_pos: (f32, f32, f32),
    vel: (f32, f32, f32),
    color: (u8, u8, u8, u8),
}

#[derive(Clone)]
struct CubeEntity {
    pos: Vec3,
    target_pos: Vec3,
    vel: Vec3,
    color: Color,
}

#[derive(Serialize, Deserialize)]
struct SerializableState {
    camera: SerializableCameraState,
    cubes: Vec<SerializableCubeEntity>,
}

const GOLDEN_ANGLE: f32 = 2.39996;
const MAX_PITCH: f32 = 89.0f32.to_radians();
const FRICTION: f32 = 8.0;
const ACCELERATION: f32 = 12.0;
const CUBE_OFFSET_RADIUS_XZ: f32 = 0.28;
const CUBE_OFFSET_Y: f32 = 0.12;
const SPRING_STIFFNESS: f32 = 150.0;
const DAMPING: f32 = 10.0;
const MAP_CENTER_OFFSET: f32 = 5.5;

const FULL_ROTATION_DEGREES: f32 = 360.0;
const MIN_GRAB_DIST: f32 = 1.0;
const MAX_GRAB_DIST: f32 = 30.0;
const SPEED_SCALE_FACTOR: f32 = 1.20;
const MIN_CAMERA_SPEED: f32 = 0.1;
const MAX_CAMERA_SPEED: f32 = 10.0;
const SPHERE_INTERSECT_RADIUS: f32 = 0.25;
const MAX_WISH_DIR_LEN: f32 = 1.0;

const CAMERA_FOVY: f32 = 70.0;
const GRID_SPACING: f32 = 0.5;
const CUBE_SIZE: f32 = 0.12;
const PITCH_DISPLAY_OFFSET: f32 = 89.0;
const LASER_OFFSET_FORWARD: f32 = 0.5;
const LASER_OFFSET_SIDE: f32 = 0.2;
const LASER_OFFSET_DOWN: f32 = 0.2;

const COLOR_RED: Color = Color {
    r: 255,
    g: 80,
    b: 80,
    a: 255,
};
const COLOR_GREEN: Color = Color {
    r: 80,
    g: 255,
    b: 150,
    a: 255,
};
const COLOR_BLUE: Color = Color {
    r: 80,
    g: 150,
    b: 255,
    a: 255,
};
const COLOR_YELLOW: Color = Color {
    r: 200,
    g: 200,
    b: 80,
    a: 255,
};
const COLOR_WHITE: Color = Color {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
};
const COLOR_LASER: Color = Color {
    r: 50,
    g: 200,
    b: 255,
    a: 200,
};
const COLOR_GRID1: Color = Color {
    r: 60,
    g: 60,
    b: 70,
    a: 255,
};
const COLOR_GRID2: Color = Color {
    r: 60,
    g: 100,
    b: 190,
    a: 255,
};

const METRIC_EVENT_NAME: &str = "overlay:update_metric";

static CAMERA: Mutex<CameraState> = Mutex::new(CameraState {
    position: Vec3 {
        x: 0.0,
        y: 10.0,
        z: 0.0,
    },
    yaw: 0.0f32.to_radians(),
    pitch: -45.0f32.to_radians(),
    sensitivity: 0.5,
    speed: 0.5,
    velocity: Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    wish_dir: Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
});

static CUBES: Mutex<Vec<CubeEntity>> = Mutex::new(Vec::new());
static INITIALIZED: Mutex<bool> = Mutex::new(false);
static GRAB_STATE: Mutex<Option<(usize, f32)>> = Mutex::new(None);

static MAP_DIMENSIONS: (u32, u32) = (12, 12);

struct Module;

fn intersect_sphere(ro: &Vec3, rd: &Vec3, center: &Vec3, radius: f32) -> Option<f32> {
    let oc = Vec3 {
        x: ro.x - center.x,
        y: ro.y - center.y,
        z: ro.z - center.z,
    };
    let b = oc.x * rd.x + oc.y * rd.y + oc.z * rd.z;
    let c = (oc.x * oc.x + oc.y * oc.y + oc.z * oc.z) - radius * radius;
    let discriminant = b * b - c;
    if discriminant > 0.0 {
        let t = -b - discriminant.sqrt();
        if t > 0.0 {
            return Some(t);
        }
    }
    None
}

fn apply_rotation_input(camera: &mut CameraState, dx: f32, dy: f32) {
    camera.yaw += dx * camera.sensitivity;
    camera.yaw %= f32::to_radians(FULL_ROTATION_DEGREES);
    camera.pitch += dy * camera.sensitivity;
    camera.pitch = camera.pitch.clamp(-MAX_PITCH, MAX_PITCH);
}

fn process_input_actions(
    actions: Vec<InputAction>,
    f: (f32, f32, f32),
    sx: f32,
    sz: f32,
    camera_speed: &mut f32,
    grab_state: &mut Option<(usize, f32)>,
) -> (Vec3, bool) {
    let mut wd = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut primary_pressed = false;

    for action in actions {
        match action {
            InputAction::MoveForward => {
                wd.x += f.0;
                wd.y += f.1;
                wd.z += f.2;
            }
            InputAction::MoveBackward => {
                wd.x -= f.0;
                wd.y -= f.1;
                wd.z -= f.2;
            }
            InputAction::MoveLeft => {
                wd.x -= sx;
                wd.z -= sz;
            }
            InputAction::MoveRight => {
                wd.x += sx;
                wd.z += sz;
            }
            InputAction::PrimaryAction => {
                primary_pressed = true;
            }
            InputAction::ScrollUp => {
                if let Some((_, dist)) = grab_state {
                    *dist = (*dist + 1.0).clamp(MIN_GRAB_DIST, MAX_GRAB_DIST);
                } else {
                    *camera_speed = (*camera_speed * SPEED_SCALE_FACTOR)
                        .clamp(MIN_CAMERA_SPEED, MAX_CAMERA_SPEED);
                }
            }
            InputAction::ScrollDown => {
                if let Some((_, dist)) = grab_state {
                    *dist = (*dist - 1.0).clamp(MIN_GRAB_DIST, MAX_GRAB_DIST);
                } else {
                    *camera_speed = (*camera_speed / SPEED_SCALE_FACTOR)
                        .clamp(MIN_CAMERA_SPEED, MAX_CAMERA_SPEED);
                }
            }
            _ => {}
        }
    }
    (wd, primary_pressed)
}

fn handle_primary_action(
    grab_state: &mut Option<(usize, f32)>,
    camera_pos: &Vec3,
    rd: &Vec3,
    cubes: &[CubeEntity],
) {
    if grab_state.is_some() {
        *grab_state = None;
    } else {
        let mut closest_t = f32::MAX;
        let mut closest_idx = None;

        for (i, cube) in cubes.iter().enumerate() {
            if let Some(t) = intersect_sphere(camera_pos, rd, &cube.pos, SPHERE_INTERSECT_RADIUS)
                && t < closest_t
            {
                closest_t = t;
                closest_idx = Some(i);
            }
        }
        if let Some(idx) = closest_idx {
            *grab_state = Some((idx, closest_t));
        }
    }
}

fn get_procedural_color(idx: u32) -> Color {
    match idx {
        0 => COLOR_RED,
        1 => COLOR_GREEN,
        2 => COLOR_BLUE,
        _ => COLOR_YELLOW,
    }
}

fn init_procedural_cubes(cubes: &mut Vec<CubeEntity>) {
    let mut init = INITIALIZED.lock().unwrap();
    if *init {
        return;
    }
    for x in 0..MAP_DIMENSIONS.0 {
        for z in 0..MAP_DIMENSIONS.1 {
            let (fx, fz) = (x as f32 - MAP_CENTER_OFFSET, z as f32 - MAP_CENTER_OFFSET);
            for idx in 0..4 {
                let angle = (idx as f32) * GOLDEN_ANGLE;
                let pos = Vec3 {
                    x: fx + (CUBE_OFFSET_RADIUS_XZ * angle.cos()),
                    y: CUBE_OFFSET_Y,
                    z: fz + (CUBE_OFFSET_RADIUS_XZ * angle.sin()),
                };
                cubes.push(CubeEntity {
                    pos,
                    target_pos: pos,
                    vel: Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    color: get_procedural_color(idx),
                });
            }
        }
    }
    *init = true;
}

fn apply_camera_physics(camera: &mut CameraState, dt: f32) {
    camera.velocity.x -= camera.velocity.x * FRICTION * dt;
    camera.velocity.y -= camera.velocity.y * FRICTION * dt;
    camera.velocity.z -= camera.velocity.z * FRICTION * dt;

    camera.velocity.x += camera.wish_dir.x * camera.speed * ACCELERATION * dt;
    camera.velocity.y += camera.wish_dir.y * camera.speed * ACCELERATION * dt;
    camera.velocity.z += camera.wish_dir.z * camera.speed * ACCELERATION * dt;

    camera.position.x += camera.velocity.x * dt;
    camera.position.y += camera.velocity.y * dt;
    camera.position.z += camera.velocity.z * dt;
}

fn apply_cube_physics(
    cubes: &mut [CubeEntity],
    grab_state: Option<(usize, f32)>,
    camera_pos: &Vec3,
    ray_dir: &Vec3,
    dt: f32,
) {
    if let Some((idx, dist)) = grab_state
        && let Some(cube) = cubes.get_mut(idx)
    {
        cube.target_pos = Vec3 {
            x: camera_pos.x + ray_dir.x * dist,
            y: camera_pos.y + ray_dir.y * dist,
            z: camera_pos.z + ray_dir.z * dist,
        };
    }
    for cube in cubes.iter_mut() {
        cube.vel.x += (cube.target_pos.x - cube.pos.x) * SPRING_STIFFNESS * dt;
        cube.vel.y += (cube.target_pos.y - cube.pos.y) * SPRING_STIFFNESS * dt;
        cube.vel.z += (cube.target_pos.z - cube.pos.z) * SPRING_STIFFNESS * dt;
        cube.vel.x -= cube.vel.x * DAMPING * dt;
        cube.vel.y -= cube.vel.y * DAMPING * dt;
        cube.vel.z -= cube.vel.z * DAMPING * dt;
        cube.pos.x += cube.vel.x * dt;
        cube.pos.y += cube.vel.y * dt;
        cube.pos.z += cube.vel.z * dt;
    }
}

fn render_camera_and_grid(camera: &CameraState, ray_dir: &Vec3, cmds: &mut Vec<RenderCommand>) {
    let target = Vec3 {
        x: camera.position.x + ray_dir.x,
        y: camera.position.y + ray_dir.y,
        z: camera.position.z + ray_dir.z,
    };
    cmds.push(RenderCommand::Camera(CameraCmd {
        position: camera.position,
        target,
        up: Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        fovy: CAMERA_FOVY,
    }));
    cmds.push(RenderCommand::Grid3d(Grid3dCmd {
        slices: MAP_DIMENSIONS.0 * 2,
        spacing: GRID_SPACING,
        color1: COLOR_GRID1,
        color2: COLOR_GRID2,
    }));
}

fn render_cubes(
    cubes: &[CubeEntity],
    grab_state: Option<(usize, f32)>,
    camera_pos: &Vec3,
    ray_dir: &Vec3,
    cmds: &mut Vec<RenderCommand>,
) {
    for (i, cube) in cubes.iter().enumerate() {
        let mut draw_color = cube.color;

        if let Some((grab_idx, _)) = grab_state
            && grab_idx == i
        {
            draw_color = COLOR_WHITE;
            let laser_start = Vec3 {
                x: camera_pos.x + ray_dir.x * LASER_OFFSET_FORWARD - ray_dir.z * LASER_OFFSET_SIDE,
                y: camera_pos.y - LASER_OFFSET_DOWN,
                z: camera_pos.z + ray_dir.z * LASER_OFFSET_FORWARD + ray_dir.x * LASER_OFFSET_SIDE,
            };
            cmds.push(RenderCommand::Line3d(Line3dCmd {
                start: laser_start,
                end: cube.pos,
                color: COLOR_LASER,
            }));
        }
        cmds.push(RenderCommand::Cube(CubeCmd {
            position: cube.pos,
            size: Vec3 {
                x: CUBE_SIZE,
                y: CUBE_SIZE,
                z: CUBE_SIZE,
            },
            color: draw_color,
        }));
    }
}

fn send_overlay_metrics(camera: &CameraState) {
    emit_event(
        METRIC_EVENT_NAME,
        &format!("Boxes:{}", MAP_DIMENSIONS.0 * MAP_DIMENSIONS.1),
    );
    emit_event(
        METRIC_EVENT_NAME,
        &format!(
            "Pitch:{:.2}° Yaw: {:.2}°",
            camera.pitch.to_degrees() + PITCH_DISPLAY_OFFSET,
            camera.yaw.to_degrees().abs()
        ),
    );
    emit_event(
        METRIC_EVENT_NAME,
        &format!(
            "X:{:.2} Y: {:.2} Z: {:.2}",
            camera.position.x, camera.position.y, camera.position.z
        ),
    );
}

impl Guest for Module {
    fn serialize() -> Vec<u8> {
        let camera = CAMERA.lock().unwrap();
        let cubes = CUBES.lock().unwrap();

        let serialize = SerializableState {
            camera: SerializableCameraState {
                position: (camera.position.x, camera.position.y, camera.position.z),
                yaw: camera.yaw,
                pitch: camera.pitch,
                sensitivity: camera.sensitivity,
                speed: camera.speed,
            },
            cubes: cubes
                .iter()
                .map(|c| SerializableCubeEntity {
                    pos: (c.pos.x, c.pos.y, c.pos.z),
                    target_pos: (c.target_pos.x, c.target_pos.y, c.target_pos.z),
                    vel: (c.vel.x, c.vel.y, c.vel.z),
                    color: (c.color.r, c.color.g, c.color.b, c.color.a),
                })
                .collect(),
        };
        bincode::serialize(&serialize).unwrap_or_default()
    }

    fn deserialize(state_bytes: Vec<u8>) {
        if let Ok(decoded) = bincode::deserialize::<SerializableState>(&state_bytes) {
            let mut state = CAMERA.lock().unwrap();
            *state = CameraState {
                position: Vec3 {
                    x: decoded.camera.position.0,
                    y: decoded.camera.position.1,
                    z: decoded.camera.position.2,
                },
                yaw: decoded.camera.yaw,
                pitch: decoded.camera.pitch,
                sensitivity: decoded.camera.sensitivity,
                speed: decoded.camera.speed,
                velocity: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                wish_dir: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            };

            let mut cubes = CUBES.lock().unwrap();
            *cubes = decoded
                .cubes
                .into_iter()
                .map(|c| CubeEntity {
                    pos: Vec3 {
                        x: c.pos.0,
                        y: c.pos.1,
                        z: c.pos.2,
                    },
                    target_pos: Vec3 {
                        x: c.target_pos.0,
                        y: c.target_pos.1,
                        z: c.target_pos.2,
                    },
                    vel: Vec3 {
                        x: c.vel.0,
                        y: c.vel.1,
                        z: c.vel.2,
                    },
                    color: Color {
                        r: c.color.0,
                        g: c.color.1,
                        b: c.color.2,
                        a: c.color.3,
                    },
                })
                .collect();
        }
    }

    fn handle_input(state: InputState) {
        if state.context != EngineContext::Gameplay {
            return;
        }
        let mut c = CAMERA.lock().unwrap();
        let mut grab_state = GRAB_STATE.lock().unwrap();
        let cubes = CUBES.lock().unwrap();

        apply_rotation_input(&mut c, state.mouse_delta.x, state.mouse_delta.y);

        let fx = c.yaw.sin() * c.pitch.cos();
        let fy = c.pitch.sin();
        let fz = c.yaw.cos() * c.pitch.cos();
        let rd = Vec3 {
            x: fx,
            y: fy,
            z: fz,
        };
        let (sx, sz) = (-fz, fx);

        let (mut wd, primary_pressed) = process_input_actions(
            state.actions,
            (fx, fy, fz),
            sx,
            sz,
            &mut c.speed,
            &mut grab_state,
        );

        if primary_pressed {
            handle_primary_action(&mut grab_state, &c.position, &rd, &cubes);
        }

        let len = (wd.x * wd.x + wd.y * wd.y + wd.z * wd.z).sqrt();
        if len > MAX_WISH_DIR_LEN {
            wd.x /= len;
            wd.y /= len;
            wd.z /= len;
        }
        c.wish_dir = wd;
    }

    fn update_module(_time: f32, dt: f32, _w: f32, _h: f32) -> Vec<RenderCommand> {
        let mut cmds = Vec::new();
        let mut camera = CAMERA.lock().unwrap();
        let grab_state = GRAB_STATE.lock().unwrap();
        let mut cubes = CUBES.lock().unwrap();

        init_procedural_cubes(&mut cubes);
        apply_camera_physics(&mut camera, dt);

        let rd = Vec3 {
            x: camera.yaw.sin() * camera.pitch.cos(),
            y: camera.pitch.sin(),
            z: camera.yaw.cos() * camera.pitch.cos(),
        };

        apply_cube_physics(&mut cubes, *grab_state, &camera.position, &rd, dt);
        render_camera_and_grid(&camera, &rd, &mut cmds);
        render_cubes(&cubes, *grab_state, &camera.position, &rd, &mut cmds);
        send_overlay_metrics(&camera);

        cmds
    }

    fn get_commands() -> Vec<CommandDesc> {
        Vec::new()
    }
    fn run_command(_cmd: String, _args: Vec<String>) -> ResponseCommand {
        ResponseCommand::Unknown
    }
    fn handle_event(_name: String, _payload: String) {}
}

export!(Module);
