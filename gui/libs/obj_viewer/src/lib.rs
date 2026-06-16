wit_bindgen::generate!({
    path: "../../wit",
    world: "model-viewer-world",
});

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::local::zappy::host_api::{host_log, host_subscribe};

const DEF_ALPHA: u8 = 255;
const DEF_FLT: f32 = 0.0;
const DEF_VEC: &str = "0";
const DEF_ONE: f32 = 1.0;
const FULL_RGB: f32 = 255.0;
const DEF_TEX: u32 = 0;
const KW_NEWMTL: &str = "newmtl";
const KW_KD: &str = "Kd";
const KW_MAP_KD: &str = "map_Kd";
const KW_V: &str = "v";
const KW_VT: &str = "vt";
const KW_USEMTL: &str = "usemtl";
const KW_F: &str = "f";

const CMD_SET_SCALE: &str = "set_scale";
const CMD_SET_POS: &str = "set_position";

const EVENT_LOAD: &str = "obj_viewer:load_full_scene";
const EVENT_SET_SCALE: &str = "obj_viewer:set_scale";
const EVENT_SET_POS: &str = "obj_viewer:set_position";
const EVENT_SET_ROT: &str = "obj_viewer:set_rotation";

#[derive(Serialize, Deserialize, Clone)]
struct TextureMapping {
    file_name: String,
    texture_id: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct FullModelPayload {
    obj_data: String,
    mtl_data: String,
    textures: Vec<TextureMapping>,
    scale: f32,
    pos_x: f32,
    pos_y: f32,
    pos_z: f32,
    rotation_speed: f32,
}

struct MaterialInfo {
    diff_col: [f32; 3],
    tex_id: u32,
}

struct ParsedMesh {
    verts: Vec<Vec3>,
    uvs: Vec<Vec2>,
    faces: Vec<Vec<String>>,
    mats: Vec<String>,
}

struct ModuleState {
    payload: FullModelPayload,
    mesh: ParsedMesh,
    materials: HashMap<String, MaterialInfo>,
}

static STATE: LazyLock<Mutex<Option<ModuleState>>> = LazyLock::new(|| Mutex::new(None));
static INITIALIZED: Mutex<bool> = Mutex::new(false);

struct Module;

fn init_module() {
    let mut initialized = INITIALIZED.lock().unwrap();
    if *initialized {
        return;
    }

    host_subscribe(EVENT_LOAD);
    host_subscribe(EVENT_SET_SCALE);
    host_subscribe(EVENT_SET_POS);
    host_subscribe(EVENT_SET_ROT);

    host_log("obj_viewer loaded");
    *initialized = true;
}

fn parse_f32(s: Option<String>, default: &str) -> f32 {
    s.unwrap_or_else(|| default.to_string())
        .parse()
        .unwrap_or(DEF_FLT)
}

fn parse_vec3(parts: &mut impl Iterator<Item = String>) -> Vec3 {
    Vec3 {
        x: parse_f32(parts.next(), DEF_VEC),
        y: parse_f32(parts.next(), DEF_VEC),
        z: parse_f32(parts.next(), DEF_VEC),
    }
}

fn parse_vec2(parts: &mut impl Iterator<Item = String>) -> Vec2 {
    Vec2 {
        x: parse_f32(parts.next(), DEF_VEC),
        y: parse_f32(parts.next(), DEF_VEC),
    }
}

fn parse_materials(mtl: &str, tex_map: &[TextureMapping]) -> HashMap<String, MaterialInfo> {
    let mut map = HashMap::new();
    let mut cur_name = String::new();
    let mut cur_col = [DEF_ONE, DEF_ONE, DEF_ONE];
    let mut cur_tex = DEF_TEX;

    for line in mtl.lines() {
        let mut p = line.split_whitespace().map(String::from);
        if let Some(kw) = p.next() {
            match kw.as_str() {
                KW_NEWMTL => {
                    if !cur_name.is_empty() {
                        map.insert(
                            cur_name.clone(),
                            MaterialInfo {
                                diff_col: cur_col,
                                tex_id: cur_tex,
                            },
                        );
                    }
                    cur_name = p.next().unwrap_or_default();
                    cur_tex = DEF_TEX;
                }
                KW_KD => {
                    cur_col = [
                        parse_f32(p.next(), DEF_VEC),
                        parse_f32(p.next(), DEF_VEC),
                        parse_f32(p.next(), DEF_VEC),
                    ];
                }
                KW_MAP_KD => {
                    let fname = p.next().unwrap_or_default();
                    cur_tex = tex_map
                        .iter()
                        .find(|t| t.file_name == fname)
                        .map(|t| t.texture_id)
                        .unwrap_or(DEF_TEX);
                }
                _ => {}
            }
        }
    }
    if !cur_name.is_empty() {
        map.insert(
            cur_name,
            MaterialInfo {
                diff_col: cur_col,
                tex_id: cur_tex,
            },
        );
    }
    map
}

fn parse_obj_data(obj_data: &str) -> ParsedMesh {
    let mut mesh = ParsedMesh {
        verts: Vec::new(),
        uvs: Vec::new(),
        faces: Vec::new(),
        mats: Vec::new(),
    };
    let mut act_mat = String::new();

    for line in obj_data.lines() {
        let mut p = line.split_whitespace().map(String::from);
        if let Some(kw) = p.next() {
            match kw.as_str() {
                KW_V => mesh.verts.push(parse_vec3(&mut p)),
                KW_VT => mesh.uvs.push(parse_vec2(&mut p)),
                KW_USEMTL => act_mat = p.next().unwrap_or_default(),
                KW_F => {
                    mesh.faces.push(p.collect());
                    mesh.mats.push(act_mat.clone());
                }
                _ => {}
            }
        }
    }
    mesh
}

fn build_verts(
    tokens: &[String],
    mesh: &ParsedMesh,
    mat: &MaterialInfo,
    cos_sin: (f32, f32),
    p: &FullModelPayload,
    out: &mut Vec<Vertex>,
) {
    let (cos_t, sin_t) = cos_sin;

    for token in tokens {
        let subs: Vec<&str> = token.split('/').collect();
        if subs.is_empty() {
            continue;
        }

        let v_idx = match subs[0].parse::<usize>() {
            Ok(idx) => idx.saturating_sub(1),
            Err(_) => continue,
        };
        if v_idx >= mesh.verts.len() {
            continue;
        }

        let orig = mesh.verts[v_idx];

        let vt_idx = subs
            .get(1)
            .and_then(|s| s.parse::<usize>().ok())
            .map(|i| i.saturating_sub(1))
            .and_then(|i| if i < mesh.uvs.len() { Some(i) } else { None });

        let uv = vt_idx
            .and_then(|i| mesh.uvs.get(i).copied())
            .unwrap_or(Vec2 {
                x: DEF_FLT,
                y: DEF_FLT,
            });

        let rx = orig.x * cos_t - orig.z * sin_t;
        let rz = orig.x * sin_t + orig.z * cos_t;

        out.push(Vertex {
            position: Vec3 {
                x: (rx * p.scale) + p.pos_x,
                y: (orig.y * p.scale) + p.pos_y,
                z: (rz * p.scale) + p.pos_z,
            },
            normal: Vec3 {
                x: DEF_FLT,
                y: DEF_ONE,
                z: DEF_FLT,
            },
            uv,
            color: Color {
                r: (mat.diff_col[0] * FULL_RGB) as u8,
                g: (mat.diff_col[1] * FULL_RGB) as u8,
                b: (mat.diff_col[2] * FULL_RGB) as u8,
                a: DEF_ALPHA,
            },
        });
    }
}

impl Guest for Module {
    fn update_module(time: f32, _dt: f32, _w: f32, _h: f32) -> Vec<RenderCommand> {
        init_module();
        let mut lock = STATE.lock().unwrap();
        let s = match lock.as_mut() {
            Some(x) => x,
            None => return vec![],
        };

        let angle = time * s.payload.rotation_speed;
        let cos_sin = (angle.cos(), angle.sin());
        let def_mat = MaterialInfo {
            diff_col: [DEF_ONE, DEF_ONE, DEF_ONE],
            tex_id: DEF_TEX,
        };

        const MAX_VERTICES: usize = u8::MAX as usize;
        const MAX_INDICES: usize = u8::MAX as usize;

        let mut commands = Vec::new();

        struct Chunk {
            vertices: Vec<Vertex>,
            indices: Vec<u16>,
        }

        let mut tex_chunks: HashMap<u32, Vec<Chunk>> = HashMap::new();

        for (i, face) in s.mesh.faces.iter().enumerate() {
            let mat = s.materials.get(&s.mesh.mats[i]).unwrap_or(&def_mat);
            let num_verts = face.len();
            if num_verts < 3 {
                continue;
            }

            let new_indices = 3 * (num_verts - 2);
            let chunks = tex_chunks.entry(mat.tex_id).or_default();

            let chunk = if let Some(last) = chunks.last_mut() {
                if last.vertices.len() + num_verts <= MAX_VERTICES
                    && last.indices.len() + new_indices <= MAX_INDICES
                {
                    last
                } else {
                    chunks.push(Chunk {
                        vertices: Vec::new(),
                        indices: Vec::new(),
                    });
                    chunks.last_mut().unwrap()
                }
            } else {
                chunks.push(Chunk {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                });
                chunks.last_mut().unwrap()
            };

            let start_idx = chunk.vertices.len() as u16;
            build_verts(face, &s.mesh, mat, cos_sin, &s.payload, &mut chunk.vertices);

            for j in 1..num_verts - 1 {
                chunk.indices.extend_from_slice(&[
                    start_idx,
                    start_idx + j as u16,
                    start_idx + (j + 1) as u16,
                ]);
            }
        }

        for (tex_id, chunks) in tex_chunks {
            for chunk in chunks {
                if !chunk.indices.is_empty() {
                    commands.push(RenderCommand::Mesh3d(Mesh3dCmd {
                        vertices: chunk.vertices,
                        indices: chunk.indices,
                        texture_id: tex_id,
                    }));
                }
            }
        }

        commands
    }

    fn handle_event(name: String, payload: String) {
        match name.as_str() {
            EVENT_LOAD => match serde_json::from_str::<FullModelPayload>(&payload) {
                Ok(p) => {
                    host_log(
                        format!(
                            "Successfully parsed JSON. obj_data len: {}",
                            p.obj_data.len()
                        )
                        .as_str(),
                    );
                    let materials = parse_materials(&p.mtl_data, &p.textures);
                    let mesh = parse_obj_data(&p.obj_data);
                    *STATE.lock().unwrap() = Some(ModuleState {
                        payload: p,
                        mesh,
                        materials,
                    });
                }
                Err(e) => {
                    host_log(format!("Error parsing JSON: {:?}", e).as_str());
                }
            },
            EVENT_SET_SCALE => {
                if let Some((i_str, scale_str)) = payload.split_once(' ')
                    && let Ok(_idx) = i_str.parse::<u8>()
                    && let Ok(scale) = scale_str.parse::<f32>()
                {
                    let mut lock = STATE.lock().unwrap();
                    let s = match lock.as_mut() {
                        Some(x) => x,
                        None => return,
                    };

                    s.payload.scale = scale;
                }
            }
            _ => {}
        }
    }

    fn serialize() -> Vec<u8> {
        Vec::new()
    }
    fn deserialize(_: Vec<u8>) {}
    fn handle_input(_: InputState) {}
    fn get_commands() -> Vec<CommandDesc> {
        Vec::new()
    }
    fn run_command(cmd: String, args: Vec<String>) -> ResponseCommand {
        match cmd.as_str() {
            CMD_SET_SCALE => {
                if let Some(arg) = args.first()
                    && let Ok(scale) = arg.parse::<f32>()
                {
                    let mut lock = STATE.lock().unwrap();
                    let s = match lock.as_mut() {
                        Some(x) => x,
                        None => return ResponseCommand::Ok,
                    };

                    s.payload.scale = scale;

                    ResponseCommand::Ok
                } else {
                    ResponseCommand::BadArgument
                }
            }
            CMD_SET_POS => {
                if args.len() >= 3
                    && let Ok(pos_x) = args[0].parse::<f32>()
                    && let Ok(pos_y) = args[1].parse::<f32>()
                    && let Ok(pos_z) = args[2].parse::<f32>()
                {
                    let mut lock = STATE.lock().unwrap();
                    let s = match lock.as_mut() {
                        Some(x) => x,
                        None => return ResponseCommand::Ok,
                    };

                    s.payload.pos_x = pos_x;
                    s.payload.pos_y = pos_y;
                    s.payload.pos_z = pos_z;

                    ResponseCommand::Ok
                } else {
                    ResponseCommand::BadArgument
                }
            }
            _ => ResponseCommand::Unknown,
        }
    }
}

export!(Module);
