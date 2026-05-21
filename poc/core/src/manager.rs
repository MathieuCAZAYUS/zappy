use crate::plugin::{KeyEvent, PluginInstance, TextSegment};
use colored::*;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use wasmtime::Engine;

pub struct SharedEngineState {
    pub cached_commands: Vec<(String, String)>,
    pub reload_queue: Vec<Option<String>>,
    pub logs_to_broadcast: Vec<Vec<TextSegment>>,
}

pub struct PluginManager {
    engine: Engine,
    pub pipeline: Vec<PluginInstance>,
    pub shared: Arc<Mutex<SharedEngineState>>,
}

impl PluginManager {
    pub fn new(engine: Engine) -> Self {
        let shared = Arc::new(Mutex::new(SharedEngineState {
            cached_commands: Vec::new(),
            reload_queue: Vec::new(),
            logs_to_broadcast: Vec::new(),
        }));
        Self {
            engine,
            pipeline: Vec::new(),
            shared,
        }
    }

    pub fn scan_and_load_all(&mut self) {
        self.pipeline.clear();
        if let Ok(mut s) = self.shared.lock() {
            s.cached_commands.clear();
        }

        let dir = Path::new("plugins");
        if !dir.exists() {
            return;
        }

        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("wasm"))
            .collect();

        entries.sort();

        for path in entries {
            match PluginInstance::load(&self.engine, &path, self.shared.clone()) {
                Ok(mut plugin) => {
                    if let Ok(cmds) = plugin.bindings.call_get_commands(&mut plugin.store)
                        && let Ok(mut s) = self.shared.lock()
                    {
                        for c in cmds {
                            s.cached_commands.push((c.name, c.help));
                        }
                    }
                    self.pipeline.push(plugin);
                }
                Err(e) => {
                    eprintln!(
                        "{} {} {}{} {e}",
                        "[ERROR]".red().bold(),
                        "loading".bright_black(),
                        path.to_string_lossy().italic().bright_black(),
                        ":".bright_black()
                    );
                }
            }
        }
    }

    pub fn reload_plugin(&mut self, name: &str) {
        self.pipeline.retain(|p| p.name != name);
        let path = format!("plugins/{name}.wasm");

        if let Ok(mut plugin) =
            PluginInstance::load(&self.engine, Path::new(&path), self.shared.clone())
        {
            if let Ok(cmds) = plugin.bindings.call_get_commands(&mut plugin.store)
                && let Ok(mut s) = self.shared.lock()
            {
                s.cached_commands.retain(|(p_name, _)| p_name != name);
                for c in cmds {
                    s.cached_commands.push((c.name, c.help));
                }
            }
            self.pipeline.push(plugin);
            self.pipeline.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }

    pub fn handle_inputs(&mut self, event: KeyEvent) {
        let mut input_blocked = false;
        self.pipeline.retain_mut(|plugin| {
            if input_blocked {
                return true;
            }
            match plugin.bindings.call_handle_input(&mut plugin.store, &event) {
                Ok(consumed) => {
                    if consumed {
                        input_blocked = true;
                    }
                    true
                }
                Err(e) => {
                    eprintln!(
                        "{} {} {} {} {e}",
                        "[CRASH]".red().bold(),
                        "Plugin".bright_black(),
                        plugin.name.italic().bright_black(),
                        "panicked (Input):".bright_black()
                    );
                    false
                }
            }
        });
    }

    pub fn broadcast_logs(&mut self) {
        let logs = if let Ok(mut s) = self.shared.lock() {
            std::mem::take(&mut s.logs_to_broadcast)
        } else {
            Vec::new()
        };

        for log in logs {
            for plugin in &mut self.pipeline {
                plugin
                    .bindings
                    .call_accept_log(&mut plugin.store, &log)
                    .ok();
            }
        }
    }
}
