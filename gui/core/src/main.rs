mod graphic;
mod input_manager;
mod manager;
mod module;
mod watcher;

use colored::*;
use macroquad::prelude::{Color as MQColor, *};
use manager::ModuleManager;
use module::ResponseCommand;
use wasmtime::{Config, Engine};

use crate::module::RenderCommand;

const MAX_CMDS_CAPACITY: usize = 128;

macro_rules! log_all {
    ($manager:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        println!("{msg}");
        if let Ok(mut s) = $manager.shared.lock() {
            s.logs_to_broadcast.push(crate::module::parse_ansi_colors(&msg));
        }
    }};
}

fn handle_module_reloads(
    manager: &mut ModuleManager,
    reload_rx: &std::sync::mpsc::Receiver<String>,
) {
    let reloads = if let Ok(mut s) = manager.shared.lock() {
        std::mem::take(&mut s.reload_queue)
    } else {
        Vec::new()
    };
    for req in reloads {
        match req {
            None => {
                log_all!(
                    manager,
                    "{} {}",
                    "[SYSTEM]".bright_blue().bold(),
                    "Reloading all modules...".bright_black()
                );
                manager.scan_and_load_all();
            }
            Some(name) => {
                log_all!(
                    manager,
                    "{} {}{}{}",
                    "[SYSTEM]".bright_blue().bold(),
                    "Reloading module '".bright_black(),
                    name.italic().bright_blue().bold(),
                    "'".bright_black()
                );
                manager.reload_module(&name);
            }
        }
    }
    if let Ok(changed_module) = reload_rx.try_recv() {
        std::thread::sleep(std::time::Duration::from_millis(50));
        log_all!(
            manager,
            "{} {} {}",
            "[WATCHER]".bright_yellow().bold(),
            "File edit:".bright_black(),
            changed_module.italic().bright_black()
        );
        manager.reload_module(&changed_module);
    }
}

fn execute_single_command(manager: &mut ModuleManager, cmd: String, args: Vec<String>) {
    let mut handled = false;
    for module in &mut manager.pipeline {
        match module.call_run_command(&cmd, &args) {
            Ok(ResponseCommand::Ok) => {
                handled = true;
                break;
            }
            Ok(ResponseCommand::BadArgument) => {
                log_all!(
                    manager,
                    "{} {}{}{}{}{}",
                    "[ERROR]".red().bold(),
                    "Bad argument(s): '".bright_black(),
                    args.join(" ").magenta(),
                    "'. See command's argument(s) with '".bright_black(),
                    "help".green(),
                    "'.".bright_black()
                );
                handled = true;
                break;
            }
            Ok(ResponseCommand::Unknown) => {}
            Err(e) => {
                log_all!(
                    manager,
                    "{} {} {}{}{e}",
                    "[ERROR]".red().bold(),
                    "running command:".bright_black(),
                    cmd.green(),
                    ": ".bright_black()
                );
            }
        }
    }
    if !handled {
        log_all!(
            manager,
            "{} {} {}{}{}{}",
            "[ERROR]".red().bold(),
            "Unknown command:".bright_black(),
            cmd.green(),
            ". See available commands with '".bright_black(),
            "help".green(),
            "'.".bright_black()
        );
    }
}

fn pending_commands(manager: &mut ModuleManager) {
    let commands = if let Ok(mut s) = manager.shared.lock() {
        std::mem::take(&mut s.command_queue)
    } else {
        Vec::new()
    };
    for (cmd, args) in commands {
        execute_single_command(manager, cmd, args);
    }
}

fn render_scene_pipeline(all_cmds: &[Vec<RenderCommand>]) {
    clear_background(MQColor::new(0.1, 0.1, 0.12, 1.0));

    let mut camera_set = None;
    for cmds in all_cmds {
        if let Some(cam) = graphic::extract_camera(cmds) {
            camera_set = Some(cam);
        }
    }
    if let Some(cam) = camera_set {
        set_camera(&cam);
        for cmds in all_cmds {
            for cmd in cmds {
                graphic::render_3d_command(cmd);
            }
        }
        set_default_camera();
    }
    for cmds in all_cmds {
        for cmd in cmds {
            graphic::render_2d_command(cmd);
        }
    }
}

fn update_modules_pipeline(manager: &mut ModuleManager, all_cmds: &mut Vec<Vec<RenderCommand>>) {
    let dt = get_frame_time();
    let time = get_time() as f32;
    manager.pipeline.retain_mut(|m| {
        match m.call_update_module(time, dt, screen_width(), screen_height()) {
            Ok(cmds) => {
                all_cmds.push(cmds);
                true
            }
            Err(e) => {
                log_all!(
                    manager,
                    "{} {} {}{} {e}",
                    "[CRASH]".red().bold(),
                    "Shutting down".bright_black(),
                    m.name.italic().bright_blue().bold(),
                    ":".bright_black()
                );

                if let Ok(mut s) = m.store.data().shared.lock() {
                    s.cleanup_module(&m.name);
                }

                false
            }
        }
    });
}

#[macroquad::main("Zappy")]
async fn main() -> Result<(), anyhow::Error> {
    colored::control::set_override(true);

    let mut config = Config::new();
    config.wasm_component_model(true);

    let mut manager = ModuleManager::new(Engine::new(&config)?);
    manager.scan_and_load_all();
    let (reload_rx, _watcher) = watcher::setup()?;

    let mut input_manager = input_manager::InputManager::new();

    let mut all_cmds = Vec::with_capacity(MAX_CMDS_CAPACITY);

    log_all!(
        manager,
        "{} {}",
        "[SYSTEM]".bright_blue().bold(),
        "Core started successfully!".bright_black()
    );

    loop {
        let input_state = input_manager.process(&mut manager.context);
        handle_module_reloads(&mut manager, &reload_rx);
        pending_commands(&mut manager);

        manager.handle_inputs(input_state);
        manager.dispatch_events();
        manager.broadcast_logs();

        all_cmds.clear();
        update_modules_pipeline(&mut manager, &mut all_cmds);
        render_scene_pipeline(&all_cmds);
        next_frame().await;
    }
}
