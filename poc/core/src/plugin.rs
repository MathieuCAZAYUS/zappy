use std::sync::{Arc, Mutex};

use colored::*;
use wasmtime::component::*;
use wasmtime::{Engine, Store};

use crate::manager::SharedEngineState;

wasmtime::component::bindgen!({
    path: "../wit/zappy.wit",
    world: "plugin-world",
});

pub struct StateData;

impl wasmtime::component::HasData for StateData {
    type Data<'a> = &'a mut HostState;
}

pub struct HostState {
    pub shared: Arc<Mutex<SharedEngineState>>,
}

impl PluginWorldImports for HostState {
    fn host_log(&mut self, msg: String) {
        if let Ok(mut s) = self.shared.lock() {
            s.logs_to_broadcast.push(format!(
                "{} {}",
                "[PLUGIN]".bright_magenta().bold(),
                msg.bright_black()
            ));
        }
    }

    fn host_system_command(&mut self, cmd: String, args: Vec<String>) -> String {
        let mut s = match self.shared.lock() {
            Ok(state) => state,
            Err(_) => return "Intern Core Error".to_string(),
        };

        match cmd.as_str() {
            "reload" => {
                if args.is_empty() {
                    s.reload_queue.push(None);
                    "Core: Reloading all plugins...".to_string()
                } else {
                    s.reload_queue.push(Some(args[0].clone()));
                    format!("Core: Reloading '{}'...", args[0])
                }
            }
            "help" => {
                let mut out = "=== AVAILABLE COMMANDS ===\n".to_string();
                out.push_str(">>> help               - Show this help menu\n");
                out.push_str(">>> reload [plugin]    - Reload one or all plugins\n");
                for (name, help) in &s.cached_commands {
                    out.push_str(&format!("  {:<18} - {}\n", name, help));
                }
                out
            }
            _ => format!("Unknown command: {cmd}. See available commands with 'help'."),
        }
    }
}

pub struct PluginInstance {
    pub name: String,
    pub store: Store<HostState>,
    pub bindings: PluginWorld,
}

impl PluginInstance {
    pub fn load(
        engine: &Engine,
        path: &std::path::Path,
        shared: Arc<Mutex<SharedEngineState>>,
    ) -> Result<Self, anyhow::Error> {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let component_bytes = std::fs::read(path)?;

        let component = Component::new(engine, &component_bytes)?;
        let mut store = Store::new(engine, HostState { shared });
        let mut linker = Linker::new(engine);

        PluginWorld::add_to_linker::<HostState, StateData>(
            &mut linker,
            |state: &mut HostState| state,
        )?;

        linker.define_unknown_imports_as_traps(&component)?;

        let bindings = PluginWorld::instantiate(&mut store, &component, &linker)?;

        Ok(PluginInstance {
            name,
            store,
            bindings,
        })
    }
}
