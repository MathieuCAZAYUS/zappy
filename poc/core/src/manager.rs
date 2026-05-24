use crate::module::{EngineContext, InputState, ModuleInstance, TextSegment};
use colored::*;
use std::{
    collections::HashMap,
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
    pub event_queue: Vec<(String, String)>,
    pub event_subscriptions: HashMap<String, Vec<String>>,
    pub kv_store: HashMap<String, Vec<u8>>,
}

impl SharedEngineState {
    pub fn cleanup_module(&mut self, name: &str) {
        self.cached_commands
            .retain(|(p_name, _, _, _)| p_name != name);
        self.loaded_modules.retain(|m| m != name);
        for subs in self.event_subscriptions.values_mut() {
            subs.retain(|m| m != name);
        }
    }
}

pub struct ModuleManager {
    engine: Engine,
    pub pipeline: Vec<ModuleInstance>,
    pub shared: Arc<Mutex<SharedEngineState>>,
    pub context: EngineContext,
}

impl ModuleManager {
    pub fn new(engine: Engine) -> Self {
        let shared = Arc::new(Mutex::new(SharedEngineState {
            cached_commands: Vec::new(),
            loaded_modules: Vec::new(),
            reload_queue: Vec::new(),
            logs_to_broadcast: Vec::new(),
            command_queue: Vec::new(),
            event_queue: Vec::new(),
            event_subscriptions: HashMap::new(),
            kv_store: HashMap::new(),
        }));
        Self {
            engine,
            pipeline: Vec::new(),
            shared,
            context: EngineContext::Gameplay,
        }
    }

    pub fn scan_and_load_all(&mut self) {
        self.pipeline.clear();
        if let Ok(mut s) = self.shared.lock() {
            s.cached_commands.clear();
            s.loaded_modules.clear();
            s.event_subscriptions.clear();
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
                        path.to_string_lossy().italic().bright_blue().bold(),
                        ":".bright_black()
                    );
                }
            }
        }
    }

    pub fn reload_module(&mut self, name: &str) {
        let mut previous_state: Option<Vec<u8>> = None;
        if let Some(old_module) = self.pipeline.iter_mut().find(|p| p.name == name) {
            previous_state = old_module.call_serialize();
        }

        self.pipeline.retain(|p| p.name != name);
        let path = format!("modules/{name}.wasm");

        if let Ok(mut s) = self.shared.lock() {
            s.cleanup_module(name);
        }

        if let Ok(mut s) = self.shared.lock() {
            s.cached_commands.retain(|(p_name, _, _, _)| p_name != name);
            s.loaded_modules.retain(|m| m != name);
        }

        if let Ok(mut module) =
            ModuleInstance::load(&self.engine, Path::new(&path), self.shared.clone())
        {
            if let Some(state_bytes) = previous_state
                && module.call_deserialize(&state_bytes).is_none()
            {
                eprintln!(
                    "{} {} {}",
                    "[WARN]".yellow().bold(),
                    "Failed to deserialize state for hot-reloaded module:".bright_black(),
                    name.italic().bright_blue().bold(),
                );
            }
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

    pub fn handle_inputs(&mut self, state: InputState) {
        self.pipeline
            .retain_mut(|module| match module.call_handle_input(&state) {
                Ok(_) => true,
                Err(e) => {
                    eprintln!(
                        "{} {} {} {} {e}",
                        "[CRASH]".red().bold(),
                        "Module".bright_black(),
                        module.name.italic().bright_blue().bold(),
                        "panicked (Input):".bright_black()
                    );
                    false
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

    pub fn dispatch_events(&mut self) {
        let mut events = Vec::new();
        let mut subs_map = HashMap::new();

        if let Ok(mut s) = self.shared.lock() {
            events = std::mem::take(&mut s.event_queue);
            subs_map = s.event_subscriptions.clone();
        }

        for (event_name, payload) in events {
            let subscribers = subs_map.get(&event_name).cloned().unwrap_or_default();

            self.pipeline.retain_mut(|module| {
                if subscribers.contains(&module.name) {
                    match module.call_handle_event(&event_name, &payload) {
                        Ok(_) => true,
                        Err(e) => {
                            eprintln!(
                                "{} {} {} {} {e}",
                                "[CRASH]".red().bold(),
                                "Module".bright_black(),
                                module.name.italic().bright_blue().bold(),
                                "failed to handle event:".bright_black(),
                            );
                            if let Ok(mut s) = module.store.data().shared.lock() {
                                s.cleanup_module(&module.name);
                            }
                            false
                        }
                    }
                } else {
                    true
                }
            });
        }
    }
}
