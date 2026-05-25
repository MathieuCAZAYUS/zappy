use std::{
    collections::{HashMap, HashSet},
    path::{Component, Path, PathBuf},
};

use macroquad::prelude::{FilterMode, Texture2D};

const SPRITE_ROOT: &str = "assets/sprites";

pub struct SpriteLoader {
    textures: HashMap<String, Texture2D>,
    failed: HashSet<String>,
}

impl SpriteLoader {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            failed: HashSet::new(),
        }
    }

    pub fn texture(&mut self, requested_path: &str) -> Result<&Texture2D, String> {
        let key = requested_path.trim().to_string();

        if key.is_empty() {
            return Err("empty sprite path".to_string());
        }

        if self.failed.contains(&key) {
            return Err(format!("sprite '{key}' already failed to load"));
        }

        if !self.textures.contains_key(&key) {
            let resolved = match Self::resolve_sprite_path(&key) {
                Ok(path) => path,
                Err(err) => {
                    self.failed.insert(key.clone());
                    return Err(err);
                }
            };

            let bytes = match std::fs::read(&resolved) {
                Ok(bytes) => bytes,
                Err(err) => {
                    self.failed.insert(key.clone());
                    return Err(format!(
                        "failed to read sprite '{}': {err}",
                        resolved.display()
                    ));
                }
            };

            let texture = Texture2D::from_file_with_format(&bytes, None);
            texture.set_filter(FilterMode::Nearest);

            self.textures.insert(key.clone(), texture);
        }

        Ok(self
            .textures
            .get(&key)
            .expect("texture should exist after loading"))
    }

    fn resolve_sprite_path(requested_path: &str) -> Result<PathBuf, String> {
        let path = Path::new(requested_path);

        if path.is_absolute() {
            return Err(format!(
                "sprite path '{requested_path}' must be relative to {SPRITE_ROOT}/"
            ));
        }

        for component in path.components() {
            match component {
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(format!(
                        "sprite path '{requested_path}' cannot leave {SPRITE_ROOT}/"
                    ));
                }
                _ => {}
            }
        }

        if path.starts_with(SPRITE_ROOT) {
            Ok(path.to_path_buf())
        } else {
            Ok(Path::new(SPRITE_ROOT).join(path))
        }
    }
}
