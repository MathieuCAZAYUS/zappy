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

pub fn parse_ansi_colors(input: &str) -> Vec<TextSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut current_color = Color {
        r: 220,
        g: 220,
        b: 225,
        a: 255,
    };

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b'
            && let Some('[') = chars.peek()
        {
            chars.next();

            let mut code = String::new();
            while let Some(&nc) = chars.peek() {
                chars.next();
                if nc == 'm' {
                    break;
                }
                code.push(nc);
            }

            if !current_text.is_empty() {
                segments.push(TextSegment {
                    text: current_text.clone(),
                    color: current_color,
                });
                current_text.clear();
            }

            match code.as_str() {
                "0" => {
                    current_color = Color {
                        r: 220,
                        g: 220,
                        b: 225,
                        a: 255,
                    }
                }
                "30" | "90" | "1;30" | "1;90" => {
                    current_color = Color {
                        r: 100,
                        g: 100,
                        b: 100,
                        a: 255,
                    }
                }
                "31" | "91" | "1;31" | "1;91" => {
                    current_color = Color {
                        r: 255,
                        g: 80,
                        b: 80,
                        a: 255,
                    }
                }
                "32" | "92" | "1;32" | "1;92" => {
                    current_color = Color {
                        r: 100,
                        g: 255,
                        b: 100,
                        a: 255,
                    }
                }
                "33" | "93" | "1;33" | "1;93" => {
                    current_color = Color {
                        r: 255,
                        g: 255,
                        b: 100,
                        a: 255,
                    }
                }
                "34" | "94" | "1;34" | "1;94" => {
                    current_color = Color {
                        r: 100,
                        g: 150,
                        b: 255,
                        a: 255,
                    }
                }
                "35" | "95" | "1;35" | "1;95" => {
                    current_color = Color {
                        r: 255,
                        g: 100,
                        b: 255,
                        a: 255,
                    }
                }
                "36" | "96" | "1;36" | "1;96" => {
                    current_color = Color {
                        r: 100,
                        g: 255,
                        b: 255,
                        a: 255,
                    }
                }
                _ => {
                    current_color = Color {
                        r: 220,
                        g: 220,
                        b: 225,
                        a: 255,
                    }
                }
            }
        } else {
            current_text.push(c);
        }
    }

    if !current_text.is_empty() {
        segments.push(TextSegment {
            text: current_text,
            color: current_color,
        });
    }

    segments
}

impl PluginWorldImports for HostState {
    fn host_log(&mut self, msg: String) {
        if let Ok(mut s) = self.shared.lock() {
            let formatted = format!(
                "{} {}",
                "[PLUGIN]".bright_magenta().bold(),
                msg.bright_black()
            );
            println!("{formatted}");
            s.logs_to_broadcast.push(parse_ansi_colors(&formatted));
        }
    }

    fn host_system_command(&mut self, cmd: String, args: Vec<String>) -> Vec<TextSegment> {
        let mut s = match self.shared.lock() {
            Ok(state) => state,
            Err(_) => return parse_ansi_colors(format!("{}", "Intern Core Error".red()).as_str()),
        };

        match cmd.as_str() {
            "reload" => {
                if args.is_empty() {
                    s.reload_queue.push(None);
                    parse_ansi_colors(
                        format!(
                            "{}{}",
                            "[SYSTEM]".bright_blue(),
                            ": Reloading all plugins...".bright_black()
                        )
                        .as_str(),
                    )
                } else {
                    s.reload_queue.push(Some(args[0].clone()));
                    parse_ansi_colors(
                        format!(
                            "{}{}{}{}",
                            "[SYSTEM]".bright_blue(),
                            ": Reloading '".bright_black(),
                            args[0].cyan(),
                            "'...".bright_black()
                        )
                        .as_str(),
                    )
                }
            }
            "help" => {
                let mut out = parse_ansi_colors(
                    format!(
                        "{} {} {}\n",
                        "===".bright_black(),
                        "AVAILABLE COMMANDS".yellow(),
                        "===".bright_black()
                    )
                    .as_str(),
                );
                out.append(&mut parse_ansi_colors(
                    format!(
                        "{}               {} {}\n",
                        ">>> help".green(),
                        "-".bright_black(),
                        "Show this help menu".blue()
                    )
                    .as_str(),
                ));
                out.append(&mut parse_ansi_colors(
                    format!(
                        "{} {}    {} {}\n",
                        ">>> reload".green(),
                        "[plugin]".magenta(),
                        "-".bright_black(),
                        "Reload one or all plugins".blue()
                    )
                    .as_str(),
                ));
                for (name, help) in &s.cached_commands {
                    out.append(&mut parse_ansi_colors(
                        format!(
                            "{} {:<18} {} {}\n",
                            ">>>".green(),
                            name.green(),
                            "-".bright_black(),
                            help.blue()
                        )
                        .as_str(),
                    ));
                }
                out
            }
            _ => parse_ansi_colors(
                format!(
                    "{} {} {}{}{}{}",
                    "[ERROR]".red().bold(),
                    "Unknown command:".bright_black(),
                    cmd.green(),
                    ". See available commands with '".bright_black(),
                    "help".green(),
                    "'.".bright_black()
                )
                .as_str(),
            ),
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
