use macroquad::texture::Texture2D;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const INITIAL_ID: u32 = 1;
const ID_INCREMENT: u32 = 1;

const MODELS_ROOT_PATH: &str = "assets/models";
const OBJ_FILE: &str = "Island.obj";
const MTLLIB_KEY: &str = "mtllib";
const MAP_KD_KEY: &str = "map_Kd";
const MAP_BUMP_KEY: &str = "map_Bump";
const MAP_NS_KEY: &str = "map_Ns";
const DEFAULT_FLOAT: f32 = 0.0;
const DEFAULT_HIGHT_FLOAT: f32 = -19.5;
const DEFAULT_SCALE: f32 = 2.0;

pub struct TextureRegistry {
    pub loaded_textures: HashMap<u32, Texture2D>,
    next_id: u32,
}

impl TextureRegistry {
    pub fn new() -> Self {
        Self {
            loaded_textures: HashMap::new(),
            next_id: INITIAL_ID,
        }
    }

    pub fn register(&mut self, texture: Texture2D) -> u32 {
        let id = self.next_id;
        self.loaded_textures.insert(id, texture);
        self.next_id += ID_INCREMENT;
        id
    }

    pub fn get(&self, id: u32) -> Option<&Texture2D> {
        self.loaded_textures.get(&id)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TextureMapping {
    pub file_name: String,
    pub texture_id: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FullModelPayload {
    pub obj_data: String,
    pub mtl_data: String,
    pub textures: Vec<TextureMapping>,
    pub scale: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub rotation_speed: f32,
}

fn extract_kw(line: &str, keyword: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    if parts.next() == Some(keyword) {
        parts.next().map(String::from)
    } else {
        None
    }
}

fn find_mtl_file(obj_content: &str) -> Option<String> {
    obj_content.lines().find_map(|l| extract_kw(l, MTLLIB_KEY))
}

fn extract_textures(mtl_content: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in mtl_content.lines() {
        if let Some(f) = extract_kw(line, MAP_KD_KEY) {
            files.push(f);
        }
        if let Some(f) = extract_kw(line, MAP_BUMP_KEY) {
            files.push(f);
        }
        if let Some(f) = extract_kw(line, MAP_NS_KEY) {
            files.push(f);
        }
    }
    files
}

fn load_texture(path: &Path) -> Option<Texture2D> {
    let bytes = fs::read(path).ok()?;
    let dyn_img = image::load_from_memory(&bytes).ok()?;
    let rgba = dyn_img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some(Texture2D::from_rgba8(w as u16, h as u16, rgba.as_raw()))
}

pub fn load_folder(dir: &Path, reg: &mut TextureRegistry) -> Option<FullModelPayload> {
    let obj_data = fs::read_to_string(dir.join(OBJ_FILE)).ok()?;
    let mtl_name = find_mtl_file(&obj_data).unwrap_or_default();
    let mtl_data = fs::read_to_string(dir.join(&mtl_name)).unwrap_or_default();

    let mut textures = Vec::new();
    for fname in extract_textures(&mtl_data) {
        if let Some(tex) = load_texture(&dir.join(&fname)) {
            let texture_id = reg.register(tex);
            textures.push(TextureMapping {
                file_name: fname,
                texture_id,
            });
        }
    }

    Some(FullModelPayload {
        obj_data,
        mtl_data,
        textures,
        scale: DEFAULT_SCALE,
        pos_x: DEFAULT_FLOAT,
        pos_y: DEFAULT_HIGHT_FLOAT,
        pos_z: DEFAULT_FLOAT,
        rotation_speed: DEFAULT_FLOAT,
    })
}

pub fn discover_models(reg: &mut TextureRegistry) -> HashMap<String, FullModelPayload> {
    let mut models = HashMap::new();
    let entries = match fs::read_dir(MODELS_ROOT_PATH) {
        Ok(e) => e,
        Err(_) => return models,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir()
            && let Some(name) = p.file_name().and_then(|s| s.to_str())
            && let Some(payload) = load_folder(&p, reg)
        {
            models.insert(name.to_string(), payload);
        }
    }
    models
}
