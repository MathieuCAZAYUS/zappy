mod manager;
mod module;
mod sprite_loader;
mod watcher;

use colored::*;
use macroquad::prelude::{Color as MQColor, *};
use manager::ModuleManager;
use module::{EngineContext, InputAction, InputState, ResponseCommand};
use sprite_loader::SpriteLoader;
use wasmtime::{Config, Engine};

use crate::module::RenderCommand;

macro_rules! log_all {
    ($manager:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        println!("{msg}");
        if let Ok(mut s) = $manager.shared.lock() {
            s.logs_to_broadcast.push(crate::module::parse_ansi_colors(&msg));
        }
    }};
}

#[macroquad::main("Zappy PoC")]
async fn main() -> Result<(), anyhow::Error> {
    colored::control::set_override(true);

    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let mut manager = ModuleManager::new(engine);
    let mut sprite_loader = SpriteLoader::new();
    manager.scan_and_load_all();
    let (reload_rx, _watcher) = watcher::setup()?;

    log_all!(
        manager,
        "{} {}",
        "[SYSTEM]".bright_blue().bold(),
        "Core started successfully!".bright_black()
    );

    loop {
        clear_background(MQColor::new(0.1, 0.1, 0.12, 1.0));

        let pending_commands = if let Ok(mut s) = manager.shared.lock() {
            std::mem::take(&mut s.command_queue)
        } else {
            Vec::new()
        };

        for (cmd, args) in pending_commands {
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
                            ": ".bright_black(),
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
                        "'".bright_black(),
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
            actions.push(InputAction::MoveUp);
        }
        if is_key_pressed(KeyCode::Down) {
            actions.push(InputAction::NavigateDown);
            actions.push(InputAction::MoveDown);
        }
        if is_key_pressed(KeyCode::Left) {
            actions.push(InputAction::MoveLeft);
        }
        if is_key_pressed(KeyCode::Right) {
            actions.push(InputAction::MoveRight);
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

        let input_state = InputState {
            context: manager.context,
            actions,
            raw_chars,
        };

        manager.handle_inputs(input_state);

        manager.dispatch_events();

        manager.broadcast_logs();

        let dt = get_frame_time();
        manager.pipeline.retain_mut(|module| {
            match module.call_update_module(get_time() as f32, dt, screen_width(), screen_height())
            {
                Ok(cmds) => {
                    for cmd in cmds {
                        match cmd {
                            RenderCommand::Rect(r) => {
                                draw_rectangle_ex(
                                    r.x,
                                    r.y,
                                    r.w,
                                    r.h,
                                    DrawRectangleParams {
                                        rotation: r.rotation,
                                        color: MQColor::from_rgba(
                                            r.color.r, r.color.g, r.color.b, r.color.a,
                                        ),
                                        ..Default::default()
                                    },
                                );
                            }
                            RenderCommand::Text(t) => {
                                draw_text(
                                    &t.text,
                                    t.x,
                                    t.y,
                                    t.size,
                                    MQColor::from_rgba(t.color.r, t.color.g, t.color.b, t.color.a),
                                );
                            }
                            RenderCommand::Sprite(s) => match sprite_loader.texture(&s.path) {
                                Ok(texture) => {
                                    draw_texture_ex(
                                        texture,
                                        s.x,
                                        s.y,
                                        MQColor::from_rgba(
                                            s.color.r, s.color.g, s.color.b, s.color.a,
                                        ),
                                        DrawTextureParams {
                                            dest_size: Some(vec2(s.w, s.h)),
                                            source: s
                                                .source
                                                .map(|src| Rect::new(src.x, src.y, src.w, src.h)),
                                            rotation: s.rotation,
                                            flip_x: s.flip_x,
                                            flip_y: s.flip_y,
                                            ..Default::default()
                                        },
                                    );
                                }
                                Err(e) => {
                                    log_all!(
                                        manager,
                                        "{} {}",
                                        "[SPRITE]".yellow().bold(),
                                        e.bright_black()
                                    );
                                }
                            },
                        }
                    }
                    true
                }
                Err(e) => {
                    log_all!(
                        manager,
                        "{} {} {}{} {e}",
                        "[CRASH]".red().bold(),
                        "Shutting down".bright_black(),
                        module.name.italic().bright_blue().bold(),
                        ":".bright_black()
                    );

                    if let Ok(mut s) = module.store.data().shared.lock() {
                        s.cleanup_module(&module.name);
                    }

                    false
                }
            }
        });

        next_frame().await;
    }
}
