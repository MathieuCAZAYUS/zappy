use std::sync::{Arc, Mutex};

use colored::*;
use wasmtime::component::*;
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::manager::SharedEngineState;

pub mod ui_plugin {
    wasmtime::component::bindgen!({
        path: "../wit",
        world: "ui-world",
    });
}

pub mod cube_plugin {
    wasmtime::component::bindgen!({
        path: "../wit",
        world: "cube-world",
        with: {
            "local:zappy/graphic": super::ui_plugin::local::zappy::graphic,
            "local:zappy/input": super::ui_plugin::local::zappy::input,
            "local:zappy/command": super::ui_plugin::local::zappy::command,
            "local:zappy/system": super::ui_plugin::local::zappy::system,
        }
    });
}

pub use ui_plugin::local::zappy::{
    command::{CommandDesc, ResponseCommand},
    graphic::{Color, RenderCommand},
    input::KeyEvent,
    system::TextSegment,
};

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

pub struct HostState {
    pub shared: Arc<Mutex<SharedEngineState>>,
    pub wasi_ctx: WasiCtx,
    pub table: ResourceTable,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.table,
        }
    }
}

impl wasmtime::component::HasData for HostState {
    type Data<'a> = &'a mut Self;
}

impl ui_plugin::local::zappy::host_api::Host for HostState {
    fn host_log(&mut self, msg: String) {
        self.shared_host_log(msg);
    }
    fn host_system_command(&mut self, cmd: String, args: Vec<String>) -> Vec<TextSegment> {
        self.shared_host_system_command(cmd, args)
    }
}

impl cube_plugin::local::zappy::host_api::Host for HostState {
    fn host_log(&mut self, msg: String) {
        self.shared_host_log(msg);
    }
    fn host_system_command(&mut self, cmd: String, args: Vec<String>) -> Vec<TextSegment> {
        self.shared_host_system_command(cmd, args)
    }
}

impl ui_plugin::local::zappy::graphic::Host for HostState {}
impl ui_plugin::local::zappy::input::Host for HostState {}
impl ui_plugin::local::zappy::system::Host for HostState {}
impl ui_plugin::local::zappy::command::Host for HostState {}

impl HostState {
    fn shared_host_log(&mut self, msg: String) {
        if let Ok(mut s) = self.shared.lock() {
            let formatted = format!(
                "{} {}",
                "[module]".bright_magenta().bold(),
                msg.bright_black()
            );
            println!("{formatted}");
            s.logs_to_broadcast.push(parse_ansi_colors(&formatted));
        }
    }

    fn shared_host_system_command(&mut self, cmd: String, args: Vec<String>) -> Vec<TextSegment> {
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
                            ": Reloading all modules...".bright_black()
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
                        "=====".bright_black(),
                        "AVAILABLE COMMANDS".yellow(),
                        "=====".bright_black()
                    )
                    .as_str(),
                );
                out.append(&mut parse_ansi_colors(
                    format!(
                        "{} {:<15} {:<10} {} {}\n",
                        ">".green(),
                        "help".green(),
                        "".magenta(),
                        "-".bright_black(),
                        "Show this help menu".blue()
                    )
                    .as_str(),
                ));
                out.append(&mut parse_ansi_colors(
                    format!(
                        "{} {:<15} {:<10} {} {}\n",
                        ">".green(),
                        "reload".green(),
                        "[MODULE]".magenta(),
                        "-".bright_black(),
                        "Reload one or all modules".blue()
                    )
                    .as_str(),
                ));
                out.append(&mut parse_ansi_colors(
                    format!(
                        "{} {:<15} {:<10} {} {}\n",
                        ">".green(),
                        "modules".green(),
                        "".magenta(),
                        "-".bright_black(),
                        "List currently loaded modules".blue()
                    )
                    .as_str(),
                ));
                for (_, name, options, help) in &s.cached_commands {
                    out.append(&mut parse_ansi_colors(
                        format!(
                            "{} {:<15} {:<10} {} {}\n",
                            ">".green(),
                            name.green(),
                            options.magenta(),
                            "-".bright_black(),
                            help.blue()
                        )
                        .as_str(),
                    ));
                }
                out
            }
            "modules" => {
                let mut out = parse_ansi_colors(
                    format!(
                        "{} {} {}\n",
                        "=====".bright_black(),
                        "LOADED MODULES".cyan(),
                        "=====".bright_black()
                    )
                    .as_str(),
                );
                for (i, module) in s.loaded_modules.iter().enumerate() {
                    out.append(&mut parse_ansi_colors(
                        format!("{} {}\n", format!("{}.", i).black(), module.bright_blue())
                            .as_str(),
                    ));
                }
                out
            }
            _ => {
                s.command_queue.push((cmd.clone(), args));
                Vec::new()
            }
        }
    }
}

pub enum ModuleBindings {
    Ui(ui_plugin::UiWorld),
    Cube(cube_plugin::CubeWorld),
}

pub struct ModuleInstance {
    pub name: String,
    pub store: Store<HostState>,
    pub bindings: ModuleBindings,
}

impl ModuleInstance {
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

        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();
        let mut store = Store::new(
            engine,
            HostState {
                shared: shared.clone(),
                wasi_ctx,
                table: ResourceTable::new(),
            },
        );
        let mut linker = Linker::new(engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        ui_plugin::UiWorld::add_to_linker::<HostState, HostState>(&mut linker, |s| s)?;
        linker.allow_shadowing(true);
        if let Ok(bindings) = ui_plugin::UiWorld::instantiate(&mut store, &component, &linker) {
            return Ok(ModuleInstance {
                name,
                store,
                bindings: ModuleBindings::Ui(bindings),
            });
        }

        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();
        let mut store = Store::new(
            engine,
            HostState {
                shared,
                wasi_ctx,
                table: ResourceTable::new(),
            },
        );
        let mut linker = Linker::new(engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        cube_plugin::CubeWorld::add_to_linker::<HostState, HostState>(&mut linker, |s| s)?;
        linker.allow_shadowing(true);
        match cube_plugin::CubeWorld::instantiate(&mut store, &component, &linker) {
            Ok(bindings) => Ok(ModuleInstance {
                name,
                store,
                bindings: ModuleBindings::Cube(bindings),
            }),
            Err(e) => Err(anyhow::anyhow!(
                "Component '{}' doesn't match UI nor Cube world. Last error: {}",
                name,
                e
            )),
        }
    }

    pub fn call_update_module(
        &mut self,
        time: f32,
        dt: f32,
        w: f32,
        h: f32,
    ) -> Result<Vec<RenderCommand>, wasmtime::Error> {
        match &self.bindings {
            ModuleBindings::Ui(ui) => ui.call_update_module(&mut self.store, time, dt, w, h),
            ModuleBindings::Cube(cube) => cube.call_update_module(&mut self.store, time, dt, w, h),
        }
    }

    pub fn call_handle_input(&mut self, event: &KeyEvent) -> Result<bool, wasmtime::Error> {
        match &self.bindings {
            ModuleBindings::Ui(ui) => ui.call_handle_input(&mut self.store, event),
            ModuleBindings::Cube(cube) => cube.call_handle_input(&mut self.store, event),
        }
    }

    pub fn call_run_command(
        &mut self,
        cmd: &str,
        args: &[String],
    ) -> Result<ResponseCommand, wasmtime::Error> {
        match &self.bindings {
            ModuleBindings::Ui(ui) => ui.call_run_command(&mut self.store, cmd, args),
            ModuleBindings::Cube(cube) => cube.call_run_command(&mut self.store, cmd, args),
        }
    }

    pub fn call_get_commands(&mut self) -> Result<Vec<CommandDesc>, wasmtime::Error> {
        match &self.bindings {
            ModuleBindings::Ui(ui) => ui.call_get_commands(&mut self.store),
            ModuleBindings::Cube(cube) => cube.call_get_commands(&mut self.store),
        }
    }

    pub fn call_accept_log(&mut self, logs: &[TextSegment]) -> Result<(), wasmtime::Error> {
        match &self.bindings {
            ModuleBindings::Ui(ui) => ui.call_accept_log(&mut self.store, logs),
            ModuleBindings::Cube(_) => Ok(()),
        }
    }
}
