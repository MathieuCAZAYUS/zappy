use crate::module::{KeyEvent, ModuleInstance, TextSegment};
use colored::*;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use wasmtime::Engine;

pub struct SharedEngineState {
    pub cached_commands: Vec<(String, String, String, String)>,
    pub loaded_modules: Vec<String>,
    pub reload_queue: Vec<Option<String>>,
    pub logs_to_broadcast: Vec<Vec<TextSegment>>,
    pub command_queue: Vec<(String, Vec<String>)>,
}

pub struct ModuleManager {
    engine: Engine,
    pub pipeline: Vec<ModuleInstance>,
    pub shared: Arc<Mutex<SharedEngineState>>,
}

impl ModuleManager {
    pub fn new(engine: Engine) -> Self {
        let shared = Arc::new(Mutex::new(SharedEngineState {
            cached_commands: Vec::new(),
            loaded_modules: Vec::new(),
            reload_queue: Vec::new(),
            logs_to_broadcast: Vec::new(),
            command_queue: Vec::new(),
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
            s.loaded_modules.clear();
        }

        let dir = Path::new("modules");
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
            match ModuleInstance::load(&self.engine, &path, self.shared.clone()) {
                Ok(mut module) => {
                    if let Ok(mut s) = self.shared.lock() {
                        s.loaded_modules.push(module.name.clone());
                        if let Ok(cmds) = module.call_get_commands() {
                            for c in cmds {
                                s.cached_commands.push((
                                    module.name.clone(),
                                    c.name,
                                    c.options,
                                    c.help,
                                ));
                            }
                        }
                    }
                    self.pipeline.push(module);
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

    pub fn reload_module(&mut self, name: &str) {
        self.pipeline.retain(|p| p.name != name);
        let path = format!("modules/{name}.wasm");

        if let Ok(mut s) = self.shared.lock() {
            s.cached_commands.retain(|(p_name, _, _, _)| p_name != name);
            s.loaded_modules.retain(|m| m != name);
        }

        if let Ok(mut module) =
            ModuleInstance::load(&self.engine, Path::new(&path), self.shared.clone())
        {
            if let Ok(mut s) = self.shared.lock() {
                s.loaded_modules.push(module.name.clone());
                s.loaded_modules.sort();
                if let Ok(cmds) = module.call_get_commands() {
                    for c in cmds {
                        s.cached_commands
                            .push((module.name.clone(), c.name, c.options, c.help));
                    }
                }
            }
            self.pipeline.push(module);
            self.pipeline.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }

    pub fn handle_inputs(&mut self, event: KeyEvent) {
        let mut input_blocked = false;
        self.pipeline.retain_mut(|module| {
            if input_blocked {
                return true;
            }
            match module.call_handle_input(&event) {
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
                        "Module".bright_black(),
                        module.name.italic().bright_black(),
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
            for module in &mut self.pipeline {
                module.call_accept_log(&log).ok();
            }
        }
    }
}
