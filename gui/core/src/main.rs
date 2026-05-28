mod graphic;
mod manager;
mod module;
mod watcher;

use colored::*;
use macroquad::{
    input::set_cursor_grab,
    prelude::{Color as MQColor, *},
};
use manager::ModuleManager;
use module::{EngineContext, InputAction, InputState, ResponseCommand};
use wasmtime::{Config, Engine};

use crate::module::{RenderCommand, cube_plugin::__with_name2::Delta};

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
                    name.italic().bright_blue().black(),
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

fn process_input(manager: &mut ModuleManager) -> InputState {
    let mut actions = Vec::new();
    if is_key_pressed(KeyCode::F1) {
        actions.push(InputAction::ToggleConsole);
    }
    if is_key_pressed(KeyCode::Enter) {
        actions.push(InputAction::Confirm);
    }
    if is_key_pressed(KeyCode::Backspace) {
        actions.push(InputAction::Delete);
    }
    if is_key_pressed(KeyCode::Up) {
        actions.push(InputAction::NavigateUp);
    }
    if is_key_down(KeyCode::Up) {
        actions.push(InputAction::MoveUp);
        actions.push(InputAction::MoveForward);
    }
    if is_key_pressed(KeyCode::Down) {
        actions.push(InputAction::NavigateDown);
    }
    if is_key_down(KeyCode::Down) {
        actions.push(InputAction::MoveDown);
        actions.push(InputAction::MoveBackward);
    }
    if is_key_down(KeyCode::Left) {
        actions.push(InputAction::MoveLeft);
    }
    if is_key_down(KeyCode::Right) {
        actions.push(InputAction::MoveRight);
    }
    if is_mouse_button_pressed(MouseButton::Left) {
        actions.push(InputAction::PrimaryAction);
    }

    let (_, scroll_y) = mouse_wheel();
    if scroll_y > 0.0 {
        actions.push(InputAction::ScrollUp);
    } else if scroll_y < 0.0 {
        actions.push(InputAction::ScrollDown);
    }

    let mut raw_chars = String::new();
    while let Some(c) = get_char_pressed() {
        if !c.is_control() {
            raw_chars.push(c);
        }
    }

    if actions.contains(&InputAction::ToggleConsole) {
        manager.context = match manager.context {
            EngineContext::Gameplay => EngineContext::UiConsole,
            EngineContext::UiConsole => EngineContext::Gameplay,
        };
    }

    let is_gameplay = manager.context == EngineContext::Gameplay;
    set_cursor_grab(is_gameplay);
    show_mouse(!is_gameplay);

    let mouse_d = if manager.context == EngineContext::Gameplay {
        mouse_delta_position()
    } else {
        macroquad::math::Vec2::new(0.0, 0.0)
    };

    InputState {
        context: manager.context,
        actions,
        raw_chars,
        mouse_delta: Delta {
            x: mouse_d.x,
            y: mouse_d.y,
        },
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

    log_all!(
        manager,
        "{} {}",
        "[SYSTEM]".bright_blue().bold(),
        "Core started successfully!".bright_black()
    );

    loop {
        let input_state = process_input(&mut manager);
        handle_module_reloads(&mut manager, &reload_rx);
        pending_commands(&mut manager);

        manager.handle_inputs(input_state);
        manager.dispatch_events();
        manager.broadcast_logs();

        let mut all_cmds = Vec::new();
        update_modules_pipeline(&mut manager, &mut all_cmds);
        render_scene_pipeline(&all_cmds);
        next_frame().await;
    }
}
