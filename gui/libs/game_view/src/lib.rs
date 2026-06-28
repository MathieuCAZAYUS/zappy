wit_bindgen::generate!({
    path: "../../wit",
    world: "cube-world",
});

use serde::{Deserialize, Serialize};

use crate::local::zappy::{
    graphic::{CameraCmd, Color, CubeCmd, Grid3dCmd, Line3dCmd, Mesh3dCmd, Vec2, Vec3, Vertex},
    host_api::emit_event,
};
use std::{
    collections::HashMap,
    f32::consts::{FRAC_PI_2, PI},
    sync::{LazyLock, Mutex},
};

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
const RENDER_DISTANCE: f32 = 100.0;

const FULL_ROTATION_DEGREES: f32 = 360.0;
const MIN_GRAB_DIST: f32 = 1.0;
const MAX_GRAB_DIST: f32 = 30.0;
const SPEED_SCALE_FACTOR: f32 = 1.20;
const MIN_CAMERA_SPEED: f32 = 0.1;
const MAX_CAMERA_SPEED: f32 = 10.0;
const SPHERE_INTERSECT_RADIUS: f32 = 0.25;
const MAX_WISH_DIR_LEN: f32 = 1.0;
const MAX_CUBES_PER_RESOURCE: u32 = 5;
const GOLEM_OBJ: &str = include_str!("../../../assets/models/giant/model.obj");
const GOLEM_SCALE: f32 = 0.08;
const GOLEM_Y_OFFSET: f32 = 0.02;
const GOLEM_COLOR: Color = Color {
    r: 122,
    g: 128,
    b: 122,
    a: 255,
};
const GOLEM_EYE_COLOR: Color = Color {
    r: 235,
    g: 245,
    b: 255,
    a: 255,
};
const GOLEM_MAX_VERTICES: usize = u8::MAX as usize;
const GOLEM_MAX_INDICES: usize = u8::MAX as usize;

const CAMERA_FOVY: f32 = 70.0;
const GRID_SPACING: f32 = 0.5;
const CUBE_SIZE: f32 = 0.12;
const PITCH_DISPLAY_OFFSET: f32 = 89.0;
const LASER_OFFSET_FORWARD: f32 = 0.5;
const LASER_OFFSET_SIDE: f32 = 0.2;
const LASER_OFFSET_DOWN: f32 = 0.2;
const DEFAULT_ASPECT_RATIO: f32 = 1.33;
const TILE_UPDATE_FIELD_COUNT: usize = 9;
const PLAYER_NEW_FIELD_COUNT: usize = 5;
const PLAYER_MOVE_FIELD_COUNT: usize = 4;

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

const MAX_DT: f32 = 50.0;

struct Chunk {
    center: Vec3,
    bounding_radius: f32,
    cubes: Vec<CubeEntity>,
}

static CHUNKS: LazyLock<Mutex<HashMap<usize, Chunk>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_or_create_chunk(
    chunks: &mut HashMap<usize, Chunk>,
    cx: usize,
    cz: usize,
    chunks_z: usize,
    map_w: u32,
    map_h: u32,
) {
    let global_id = cx * chunks_z + cz;

    chunks.entry(global_id).or_insert_with(|| {
        let chunk_half_width = (CHUNK_SIZE as f32) * 0.5;
        let chunk_bounding_radius = (chunk_half_width * chunk_half_width * 2.0).sqrt() + 1.0;

        let chunk_world_x = (cx * CHUNK_SIZE) as f32 - (map_w as f32 / 2.0) + chunk_half_width;
        let chunk_world_z = (cz * CHUNK_SIZE) as f32 - (map_h as f32 / 2.0) + chunk_half_width;

        let mut chunk = Chunk {
            center: Vec3 {
                x: chunk_world_x,
                y: CUBE_OFFSET_Y,
                z: chunk_world_z,
            },
            bounding_radius: chunk_bounding_radius,
            cubes: Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * 4),
        };

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let fx = (cx * CHUNK_SIZE + x) as f32 - map_w as f32 / 2.0 + 0.5;
                let fz = (cz * CHUNK_SIZE + z) as f32 - map_h as f32 / 2.0 + 0.5;

                let tile_x = cx * CHUNK_SIZE + x;
                let tile_z = cz * CHUNK_SIZE + z;
                let tile_idx = tile_z * map_w as usize + tile_x;
                let resources = {
                    let tiles = TILE_RESOURCES.lock().unwrap();
                    tiles.get(tile_idx).copied().unwrap_or([0u32; 7])
                };
                let mut cube_idx = 0u32;
                for (res_idx, &count) in resources.iter().enumerate() {
                    for _ in 0..count.min(MAX_CUBES_PER_RESOURCE) {
                        let angle = (cube_idx as f32) * GOLDEN_ANGLE;
                        let pos = Vec3 {
                            x: fx + (CUBE_OFFSET_RADIUS_XZ * angle.cos()),
                            y: CUBE_OFFSET_Y,
                            z: fz + (CUBE_OFFSET_RADIUS_XZ * angle.sin()),
                        };
                        chunk.cubes.push(CubeEntity {
                            pos,
                            target_pos: pos,
                            vel: Vec3 {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            },
                            color: color_from_resource(res_idx),
                        });
                        cube_idx += 1;
                    }
                }
            }
        }
        chunk
    });
}

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
static _INITIALIZED: Mutex<bool> = Mutex::new(false);
static GRAB_STATE: Mutex<Option<(usize, usize, f32)>> = Mutex::new(None);

static MAP_DIMENSIONS: Mutex<(u32, u32)> = Mutex::new((0, 0));
static SUBSCRIBED: Mutex<bool> = Mutex::new(false);

struct PlayerEntity {
    id: u32,
    x: u32,
    y: u32,
    direction: u32,
    level: u32,
}

static PLAYERS: LazyLock<Mutex<Vec<PlayerEntity>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static GOLEM_MESH: LazyLock<GolemMesh> = LazyLock::new(|| parse_golem_mesh(GOLEM_OBJ));

static TILE_RESOURCES: LazyLock<Mutex<Vec<[u32; 7]>>> = LazyLock::new(|| Mutex::new(Vec::new()));

const CHUNK_SIZE: usize = 2;

struct Plane {
    normal: Vec3,
    d: f32,
}

struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    fn from_camera(pos: &Vec3, forward: &Vec3, fovy_rad: f32, aspect: f32, far_dist: f32) -> Self {
        let half_v = (fovy_rad * 0.5).tan();
        let half_h = half_v * aspect;

        let right = if forward.x.abs() < 0.001 && forward.z.abs() < 0.001 {
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            }
        } else {
            Self::normalize(&Self::cross(
                forward,
                &Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            ))
        };
        let up = Self::cross(&right, forward);

        let make_plane = |n: Vec3| {
            let norm = Self::normalize(&n);
            let d = -(norm.x * pos.x + norm.y * pos.y + norm.z * pos.z);
            Plane { normal: norm, d }
        };

        let left_v = Vec3 {
            x: forward.x - right.x * half_h,
            y: forward.y - right.y * half_h,
            z: forward.z - right.z * half_h,
        };
        let right_v = Vec3 {
            x: forward.x + right.x * half_h,
            y: forward.y + right.y * half_h,
            z: forward.z + right.z * half_h,
        };
        let down_v = Vec3 {
            x: forward.x - up.x * half_v,
            y: forward.y - up.y * half_v,
            z: forward.z - up.z * half_v,
        };
        let up_v = Vec3 {
            x: forward.x + up.x * half_v,
            y: forward.y + up.y * half_v,
            z: forward.z + up.z * half_v,
        };

        let far_normal = Vec3 {
            x: -forward.x,
            y: -forward.y,
            z: -forward.z,
        };
        let far_d = forward.x * pos.x + forward.y * pos.y + forward.z * pos.z + far_dist;
        let far_plane = Plane {
            normal: far_normal,
            d: far_d,
        };

        Self {
            planes: [
                make_plane(Self::cross(&up, &left_v)),
                make_plane(Self::cross(&right_v, &up)),
                make_plane(Self::cross(&down_v, &right)),
                make_plane(Self::cross(&right, &up_v)),
                make_plane(*forward),
                far_plane,
            ],
        }
    }

    fn is_point_visible(&self, p: &Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let dist =
                -(plane.normal.x * p.x + plane.normal.y * p.y + plane.normal.z * p.z + plane.d);
            if dist < -(RENDER_DISTANCE + radius) {
                return false;
            }
        }
        true
    }

    fn cross(a: &Vec3, b: &Vec3) -> Vec3 {
        Vec3 {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }
    }

    fn normalize(v: &Vec3) -> Vec3 {
        let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
        if len > 0.0 {
            Vec3 {
                x: v.x / len,
                y: v.y / len,
                z: v.z / len,
            }
        } else {
            *v
        }
    }
}

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
    grab_state: &mut Option<(usize, usize, f32)>,
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
                if let Some((_, _, dist)) = grab_state {
                    *dist = (*dist + 1.0).clamp(MIN_GRAB_DIST, MAX_GRAB_DIST);
                } else {
                    *camera_speed = (*camera_speed * SPEED_SCALE_FACTOR)
                        .clamp(MIN_CAMERA_SPEED, MAX_CAMERA_SPEED);
                }
            }
            InputAction::ScrollDown => {
                if let Some((_, _, dist)) = grab_state {
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
    grab_state: &mut Option<(usize, usize, f32)>,
    camera_pos: &Vec3,
    rd: &Vec3,
    chunks: &HashMap<usize, Chunk>,
) {
    if grab_state.is_some() {
        *grab_state = None;
    } else {
        let mut closest_t = f32::MAX;
        let mut closest_idx = None;

        let mut valid_chunks = Vec::with_capacity(chunks.len());

        for (&c_idx, chunk) in chunks.iter() {
            let oc = Vec3 {
                x: camera_pos.x - chunk.center.x,
                y: camera_pos.y - chunk.center.y,
                z: camera_pos.z - chunk.center.z,
            };
            let b = oc.x * rd.x + oc.y * rd.y + oc.z * rd.z;
            let sq_dist = oc.x * oc.x + oc.y * oc.y + oc.z * oc.z;
            let c = sq_dist - chunk.bounding_radius * chunk.bounding_radius;

            if c <= 0.0 {
                valid_chunks.push((c_idx, chunk, 0.0));
            } else {
                let discriminant = b * b - c;
                if discriminant >= 0.0 && b < 0.0 {
                    let t = -b - discriminant.sqrt();
                    if t > 0.0 {
                        valid_chunks.push((c_idx, chunk, t));
                    }
                }
            }
        }

        valid_chunks.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        for (c_idx, chunk, chunk_t) in valid_chunks {
            if chunk_t >= closest_t {
                break;
            }

            for (cube_idx, cube) in chunk.cubes.iter().enumerate() {
                if let Some(t) =
                    intersect_sphere(camera_pos, rd, &cube.pos, SPHERE_INTERSECT_RADIUS)
                    && t < closest_t
                {
                    closest_t = t;
                    closest_idx = Some((c_idx, cube_idx));
                }
            }
        }

        if let Some((c_idx, cube_idx)) = closest_idx {
            *grab_state = Some((c_idx, cube_idx, closest_t));
        }
    }
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

fn render_camera_and_grid(
    camera: &CameraState,
    ray_dir: &Vec3,
    cmds: &mut Vec<RenderCommand>,
    map_w: u32,
    map_h: u32,
) {
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
        slices: map_w.max(map_h) * 2,
        spacing: GRID_SPACING,
        color1: COLOR_GRID1,
        color2: COLOR_GRID2,
    }));
}

fn send_overlay_metrics(
    rendered_cubes: usize,
    camera: &CameraState,
    grab_state: Option<(usize, usize, f32)>,
) {
    emit_event(
        METRIC_EVENT_NAME,
        &format!("Rendered Cubes:{}", rendered_cubes),
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
    emit_event(METRIC_EVENT_NAME, &format!("Grab State:{grab_state:?}"));
}

fn color_from_resource(resource_idx: usize) -> Color {
    match resource_idx {
        0 => Color {
            r: 210,
            g: 140,
            b: 50,
            a: 255,
        },
        1 => Color {
            r: 160,
            g: 160,
            b: 160,
            a: 255,
        },
        2 => Color {
            r: 210,
            g: 190,
            b: 50,
            a: 255,
        },
        3 => Color {
            r: 80,
            g: 200,
            b: 100,
            a: 255,
        },
        4 => Color {
            r: 180,
            g: 80,
            b: 210,
            a: 255,
        },
        5 => Color {
            r: 80,
            g: 180,
            b: 220,
            a: 255,
        },
        _ => Color {
            r: 220,
            g: 50,
            b: 50,
            a: 255,
        },
    }
}

struct GolemFace {
    indices: Vec<usize>,
    color: Color,
}

struct GolemMesh {
    vertices: Vec<Vec3>,
    faces: Vec<GolemFace>,
    center_x: f32,
    center_z: f32,
    min_y: f32,
}

fn parse_golem_mesh(obj: &str) -> GolemMesh {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    let mut current_color = GOLEM_COLOR;

    for line in obj.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                let x = parts
                    .next()
                    .and_then(|part| part.parse().ok())
                    .unwrap_or(0.0);
                let y = parts
                    .next()
                    .and_then(|part| part.parse().ok())
                    .unwrap_or(0.0);
                let z = parts
                    .next()
                    .and_then(|part| part.parse().ok())
                    .unwrap_or(0.0);
                vertices.push(Vec3 { x, y, z });
            }
            Some("usemtl") => {
                current_color = match parts.next() {
                    Some("EYES") => GOLEM_EYE_COLOR,
                    _ => GOLEM_COLOR,
                };
            }
            Some("f") => {
                let indices = parts
                    .filter_map(|part| part.split('/').next())
                    .filter_map(|idx| idx.parse::<usize>().ok())
                    .filter_map(|idx| idx.checked_sub(1))
                    .collect::<Vec<_>>();
                if indices.len() >= 3 {
                    faces.push(GolemFace {
                        indices,
                        color: current_color,
                    });
                }
            }
            _ => {}
        }
    }

    let (mut min_x, mut max_x) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut min_y, mut min_z, mut max_z) = (f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY);
    for vertex in &vertices {
        min_x = min_x.min(vertex.x);
        max_x = max_x.max(vertex.x);
        min_y = min_y.min(vertex.y);
        min_z = min_z.min(vertex.z);
        max_z = max_z.max(vertex.z);
    }

    GolemMesh {
        vertices,
        faces,
        center_x: (min_x + max_x) * 0.5,
        center_z: (min_z + max_z) * 0.5,
        min_y,
    }
}

fn player_rotation(direction: u32) -> f32 {
    match direction {
        1 => PI,
        2 => FRAC_PI_2,
        3 => 0.0,
        4 => -FRAC_PI_2,
        _ => 0.0,
    }
}

fn transform_golem_vertex(vertex: &Vec3, mesh: &GolemMesh, origin: Vec3, rotation: f32) -> Vec3 {
    let local_x = (vertex.x - mesh.center_x) * GOLEM_SCALE;
    let local_y = (vertex.y - mesh.min_y) * GOLEM_SCALE;
    let local_z = (vertex.z - mesh.center_z) * GOLEM_SCALE;
    let (sin, cos) = rotation.sin_cos();

    Vec3 {
        x: origin.x + local_x * cos - local_z * sin,
        y: origin.y + local_y,
        z: origin.z + local_x * sin + local_z * cos,
    }
}

fn render_players_as_golems(
    players: &[PlayerEntity],
    cmds: &mut Vec<RenderCommand>,
    map_w: u32,
    map_h: u32,
) {
    let mesh = &*GOLEM_MESH;

    for player in players {
        let origin = Vec3 {
            x: player.x as f32 - map_w as f32 / 2.0 + 0.5,
            y: GOLEM_Y_OFFSET,
            z: player.y as f32 - map_h as f32 / 2.0 + 0.5,
        };
        let rotation = player_rotation(player.direction);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for face in &mesh.faces {
            if face.indices.len() < 3 {
                continue;
            }

            let new_indices = 3 * (face.indices.len() - 2);
            if vertices.len() + face.indices.len() > GOLEM_MAX_VERTICES
                || indices.len() + new_indices > GOLEM_MAX_INDICES
            {
                push_golem_mesh(&mut vertices, &mut indices, cmds);
            }

            let start_idx = vertices.len() as u16;
            for &idx in &face.indices {
                let Some(vertex) = mesh.vertices.get(idx) else {
                    continue;
                };
                vertices.push(Vertex {
                    position: transform_golem_vertex(vertex, mesh, origin, rotation),
                    normal: Vec3 {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                    },
                    uv: Vec2 { x: 0.0, y: 0.0 },
                    color: face.color,
                });
            }

            for idx in 1..face.indices.len() - 1 {
                indices.extend_from_slice(&[
                    start_idx,
                    start_idx + idx as u16,
                    start_idx + (idx + 1) as u16,
                ]);
            }
        }

        push_golem_mesh(&mut vertices, &mut indices, cmds);
    }
}

fn push_golem_mesh(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    cmds: &mut Vec<RenderCommand>,
) {
    if indices.is_empty() {
        vertices.clear();
        return;
    }

    cmds.push(RenderCommand::Mesh3d(Mesh3dCmd {
        vertices: std::mem::take(vertices),
        indices: std::mem::take(indices),
        texture_id: 0,
    }));
}

fn render_chunks(
    chunks: &mut HashMap<usize, Chunk>,
    active_ids: &[usize],
    camera_pos: &Vec3,
    ray_dir: &Vec3,
    grab_state: &mut Option<(usize, usize, f32)>,
    frustum: &Frustum,
    cmds: &mut Vec<RenderCommand>,
    dt: f32,
    map_w: u32,
    map_h: u32,
) -> usize {
    let mut batch_cubes = Vec::new();
    let mut rendered_cubes = 0;

    struct PendingTransfer {
        from_chunk: usize,
        cube_idx: usize,
        to_chunk: usize,
    }
    let mut pending_transfer: Option<PendingTransfer> = None;

    for &c_idx in active_ids {
        if let Some(chunk) = chunks.get_mut(&c_idx) {
            if let Some((grab_c_idx, grab_cube_idx, dist)) = *grab_state
                && grab_c_idx == c_idx
                && let Some(cube) = chunk.cubes.get_mut(grab_cube_idx)
            {
                cube.target_pos = Vec3 {
                    x: camera_pos.x + ray_dir.x * dist,
                    y: camera_pos.y + ray_dir.y * dist,
                    z: camera_pos.z + ray_dir.z * dist,
                };
            }

            for cube in chunk.cubes.iter_mut() {
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

            if let Some((grab_c_idx, grab_cube_idx, _)) = *grab_state
                && grab_c_idx == c_idx
                && let Some(cube) = chunk.cubes.get(grab_cube_idx)
            {
                let chunk_x_f = (cube.pos.x + (map_w as f32 / 2.0)) / (CHUNK_SIZE as f32);
                let chunk_z_f = (cube.pos.z + (map_h as f32 / 2.0)) / (CHUNK_SIZE as f32);

                let chunks_x = (map_w as usize) / CHUNK_SIZE;
                let chunks_z = (map_h as usize) / CHUNK_SIZE;

                if chunk_x_f >= 0.0 && chunk_z_f >= 0.0 {
                    let target_cx = (chunk_x_f as usize).min(chunks_x.saturating_sub(1));
                    let target_cz = (chunk_z_f as usize).min(chunks_z.saturating_sub(1));
                    let new_chunk_idx = target_cx * chunks_z + target_cz;

                    if new_chunk_idx != grab_c_idx {
                        pending_transfer = Some(PendingTransfer {
                            from_chunk: grab_c_idx,
                            cube_idx: grab_cube_idx,
                            to_chunk: new_chunk_idx,
                        });
                    }
                }
            }

            if !frustum.is_point_visible(&chunk.center, chunk.bounding_radius) {
                continue;
            }

            for (i, cube) in chunk.cubes.iter().enumerate() {
                if !frustum.is_point_visible(&cube.pos, CUBE_SIZE) {
                    continue;
                }

                rendered_cubes += 1;
                let mut draw_color = cube.color;

                if let Some((grab_c_idx, grab_cube_idx, _)) = *grab_state
                    && grab_c_idx == c_idx
                    && grab_cube_idx == i
                {
                    draw_color = COLOR_WHITE;
                    let laser_start = Vec3 {
                        x: camera_pos.x + ray_dir.x * LASER_OFFSET_FORWARD
                            - ray_dir.z * LASER_OFFSET_SIDE,
                        y: camera_pos.y - LASER_OFFSET_DOWN,
                        z: camera_pos.z
                            + ray_dir.z * LASER_OFFSET_FORWARD
                            + ray_dir.x * LASER_OFFSET_SIDE,
                    };
                    cmds.push(RenderCommand::Line3d(Line3dCmd {
                        start: laser_start,
                        end: cube.pos,
                        color: COLOR_LASER,
                    }));
                }

                batch_cubes.push(CubeCmd {
                    position: cube.pos,
                    size: Vec3 {
                        x: CUBE_SIZE,
                        y: CUBE_SIZE,
                        z: CUBE_SIZE,
                    },
                    color: draw_color,
                });
            }
        }
    }

    if let Some(transfer) = pending_transfer
        && chunks.contains_key(&transfer.from_chunk)
        && chunks.contains_key(&transfer.to_chunk)
    {
        let moving_cube = chunks
            .get_mut(&transfer.from_chunk)
            .unwrap()
            .cubes
            .remove(transfer.cube_idx);
        chunks
            .get_mut(&transfer.to_chunk)
            .unwrap()
            .cubes
            .push(moving_cube);

        if let Some((_, _, dist)) = *grab_state {
            let new_cube_idx = chunks.get(&transfer.to_chunk).unwrap().cubes.len() - 1;
            *grab_state = Some((transfer.to_chunk, new_cube_idx, dist));
        }
    }

    cmds.push(RenderCommand::InstancedCubes(InstancedCubesCmd {
        cubes: batch_cubes,
    }));
    rendered_cubes
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
        let chunks = CHUNKS.lock().unwrap();

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
            handle_primary_action(&mut grab_state, &c.position, &rd, &chunks);
        }

        let len = (wd.x * wd.x + wd.y * wd.y + wd.z * wd.z).sqrt();
        if len > MAX_WISH_DIR_LEN {
            wd.x /= len;
            wd.y /= len;
            wd.z /= len;
        }
        c.wish_dir = wd;
    }

    fn update_module(_time: f32, dt: f32, w: f32, h: f32) -> Vec<RenderCommand> {
        let mut sub = SUBSCRIBED.lock().unwrap();
        if !*sub {
            crate::local::zappy::host_api::host_subscribe("zappy:map_size");
            crate::local::zappy::host_api::host_subscribe("zappy:tile_update");
            crate::local::zappy::host_api::host_subscribe("zappy:player_new");
            crate::local::zappy::host_api::host_subscribe("zappy:player_move");
            crate::local::zappy::host_api::host_subscribe("zappy:player_death");
            *sub = true;
        }
        let (map_w, map_h) = *MAP_DIMENSIONS.lock().unwrap();
        if map_w == 0 || map_h == 0 {
            return Vec::new();
        }
        let mut cmds = Vec::new();
        let mut camera = CAMERA.lock().unwrap();
        let mut grab_state = GRAB_STATE.lock().unwrap();
        let mut chunks = CHUNKS.lock().unwrap();
        let dt = dt.min(MAX_DT);

        let chunks_x = map_w as usize / CHUNK_SIZE.min(map_w as usize);
        let chunks_z = map_h as usize / CHUNK_SIZE.min(map_h as usize);

        let cam_cx = (((camera.position.x + (map_w as f32 / 2.0)) / CHUNK_SIZE as f32).floor()
            as i32)
            .clamp(0, chunks_x as i32 - 1);
        let cam_cz = (((camera.position.z + (map_h as f32 / 2.0)) / CHUNK_SIZE as f32).floor()
            as i32)
            .clamp(0, chunks_z as i32 - 1);

        let chunk_range = (RENDER_DISTANCE / CHUNK_SIZE as f32).ceil() as i32 + 1;

        let mut active_ids = Vec::new();
        for dx in -chunk_range..=chunk_range {
            for dz in -chunk_range..=chunk_range {
                let cx = cam_cx + dx;
                let cz = cam_cz + dz;
                if cx >= 0 && cx < chunks_x as i32 && cz >= 0 && cz < chunks_z as i32 {
                    let global_id = (cx as usize) * chunks_z + (cz as usize);
                    active_ids.push(global_id);
                    get_or_create_chunk(
                        &mut chunks,
                        cx as usize,
                        cz as usize,
                        chunks_z,
                        map_w,
                        map_h,
                    );
                }
            }
        }

        apply_camera_physics(&mut camera, dt);

        let rd = Vec3 {
            x: camera.yaw.sin() * camera.pitch.cos(),
            y: camera.pitch.sin(),
            z: camera.yaw.cos() * camera.pitch.cos(),
        };

        let aspect = if h > 1.0 { w / h } else { DEFAULT_ASPECT_RATIO };
        let frustum = Frustum::from_camera(
            &camera.position,
            &rd,
            CAMERA_FOVY.to_radians(),
            aspect,
            RENDER_DISTANCE,
        );

        render_camera_and_grid(&camera, &rd, &mut cmds, map_w, map_h);

        let rendered_cubes = render_chunks(
            &mut chunks,
            &active_ids,
            &camera.position,
            &rd,
            &mut grab_state,
            &frustum,
            &mut cmds,
            dt,
            map_w,
            map_h,
        );
        {
            let players = PLAYERS.lock().unwrap();
            render_players_as_golems(&players, &mut cmds, map_w, map_h);
        }
        send_overlay_metrics(rendered_cubes, &camera, *grab_state);
        cmds
    }

    fn get_commands() -> Vec<CommandDesc> {
        Vec::new()
    }
    fn run_command(_cmd: String, _args: Vec<String>) -> ResponseCommand {
        ResponseCommand::Unknown
    }
    fn handle_event(name: String, payload: String) {
        match name.as_str() {
            "zappy:map_size" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() == 2 {
                    let w: u32 = parts[0].parse().unwrap_or(0);
                    let h: u32 = parts[1].parse().unwrap_or(0);
                    if w > 0 && h > 0 {
                        *MAP_DIMENSIONS.lock().unwrap() = (w, h);
                        *TILE_RESOURCES.lock().unwrap() = vec![[0u32; 7]; (w * h) as usize];
                        CHUNKS.lock().unwrap().clear();
                        PLAYERS.lock().unwrap().clear();
                    }
                }
            }
            "zappy:tile_update" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() == TILE_UPDATE_FIELD_COUNT {
                    let x: u32 = parts[0].parse().unwrap_or(0);
                    let y: u32 = parts[1].parse().unwrap_or(0);
                    let (map_w, _) = *MAP_DIMENSIONS.lock().unwrap();
                    if map_w > 0 {
                        let idx = (y * map_w + x) as usize;
                        let mut tiles = TILE_RESOURCES.lock().unwrap();
                        if idx < tiles.len() {
                            tiles[idx] = [
                                parts[2].parse().unwrap_or(0),
                                parts[3].parse().unwrap_or(0),
                                parts[4].parse().unwrap_or(0),
                                parts[5].parse().unwrap_or(0),
                                parts[6].parse().unwrap_or(0),
                                parts[7].parse().unwrap_or(0),
                                parts[8].parse().unwrap_or(0),
                            ];
                            let chunk_x = x as usize / CHUNK_SIZE;
                            let chunk_z = y as usize / CHUNK_SIZE;
                            let chunks_z = (map_w as usize).div_ceil(CHUNK_SIZE);
                            let chunk_id = chunk_x * chunks_z + chunk_z;
                            CHUNKS.lock().unwrap().remove(&chunk_id);
                        }
                    }
                }
            }
            "zappy:player_new" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() == PLAYER_NEW_FIELD_COUNT {
                    let id: u32 = parts[0].parse().unwrap_or(0);
                    let x: u32 = parts[1].parse().unwrap_or(0);
                    let y: u32 = parts[2].parse().unwrap_or(0);
                    let dir: u32 = parts[3].parse().unwrap_or(1);
                    let lvl: u32 = parts[4].parse().unwrap_or(1);
                    let mut players = PLAYERS.lock().unwrap();
                    players.retain(|p| p.id != id);
                    players.push(PlayerEntity {
                        id,
                        x,
                        y,
                        direction: dir,
                        level: lvl,
                    });
                }
            }
            "zappy:player_move" => {
                let parts: Vec<&str> = payload.split_whitespace().collect();
                if parts.len() == PLAYER_MOVE_FIELD_COUNT {
                    let id: u32 = parts[0].parse().unwrap_or(0);
                    let x: u32 = parts[1].parse().unwrap_or(0);
                    let y: u32 = parts[2].parse().unwrap_or(0);
                    let dir: u32 = parts[3].parse().unwrap_or(1);
                    let mut players = PLAYERS.lock().unwrap();
                    if let Some(p) = players.iter_mut().find(|p| p.id == id) {
                        p.x = x;
                        p.y = y;
                        p.direction = dir;
                    }
                }
            }
            "zappy:player_death" => {
                if let Ok(id) = payload.trim().parse::<u32>() {
                    PLAYERS.lock().unwrap().retain(|p| p.id != id);
                }
            }
            _ => {}
        }
    }
}

export!(Module);
