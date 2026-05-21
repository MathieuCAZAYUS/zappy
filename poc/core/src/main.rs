mod manager;
mod plugin;
mod watcher;

use colored::*;
use macroquad::prelude::{Color as MQColor, *};
use manager::PluginManager;
use plugin::KeyEvent;
use wasmtime::{Config, Engine};

use crate::plugin::RenderCommand;

macro_rules! log_all {
    ($manager:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        println!("{msg}");
        if let Ok(mut s) = $manager.shared.lock() {
            s.logs_to_broadcast.push(crate::plugin::parse_ansi_colors(&msg));
        }
    }};
}

#[macroquad::main("Zappy PoC")]
async fn main() -> Result<(), anyhow::Error> {
    colored::control::set_override(true);

    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let mut manager = PluginManager::new(engine);
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
                        "Reloading all plugins...".bright_black()
                    );
                    manager.scan_and_load_all();
                }
                Some(name) => {
                    log_all!(
                        manager,
                        "{} {}{}{}",
                        "[SYSTEM]".bright_blue().bold(),
                        "Reloading plugin '".bright_black(),
                        name.italic().bright_black(),
                        "'".bright_black(),
                    );
                    manager.reload_plugin(&name);
                }
            }
        }

        if let Ok(changed_plugin) = reload_rx.try_recv() {
            std::thread::sleep(std::time::Duration::from_millis(50));
            log_all!(
                manager,
                "{} {} {}",
                "[WATCHER]".bright_yellow().bold(),
                "File edit:".bright_black(),
                changed_plugin.italic().bright_black()
            );
            manager.reload_plugin(&changed_plugin);
        }

        if is_key_pressed(KeyCode::F1) {
            manager.handle_inputs(KeyEvent::Pressed("F1".into()));
        }
        if is_key_pressed(KeyCode::Enter) {
            manager.handle_inputs(KeyEvent::Pressed("Enter".into()));
        }
        if is_key_pressed(KeyCode::Backspace) {
            manager.handle_inputs(KeyEvent::Pressed("Backspace".into()));
        }

        while let Some(c) = get_char_pressed()
            && !c.is_control()
        {
            manager.handle_inputs(KeyEvent::CharInput(c.to_string()));
        }

        manager.broadcast_logs();

        manager.pipeline.retain_mut(|plugin| {
            match plugin.bindings.call_update_plugin(
                &mut plugin.store,
                get_time() as f32,
                screen_width(),
                screen_height(),
            ) {
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
                        plugin.name.italic().bright_black(),
                        ":".bright_black()
                    );
                    false
                }
            }
        });

        next_frame().await;
    }
}
