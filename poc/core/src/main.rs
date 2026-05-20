use colored::*;
use macroquad::prelude::{Color as MQColor, *};
use notify::{RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};

wasmtime::component::bindgen!({
    path: "../wit/zappy.wit",
    world: "plugin-world",
});

struct Plugin {
    store: Store<()>,
    bindings: PluginWorld,
}

impl Plugin {
    fn load(engine: &Engine, plugin_name: &str) -> Result<Self, anyhow::Error> {
        let path = format!("plugins/{plugin_name}.wasm");

        let component_bytes = std::fs::read(path)?;
        let component = Component::new(engine, &component_bytes)?;

        let mut store = Store::new(engine, ());
        let linker = Linker::new(engine);
        let bindings = PluginWorld::instantiate(&mut store, &component, &linker)?;

        Ok(Plugin { store, bindings })
    }
}

fn setup_watcher(plugin_name: &str) -> (Receiver<()>, notify::RecommendedWatcher) {
    let (tx, rx) = channel();
    let path = format!("plugins/{}.wasm", plugin_name);

    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res
                && notify::EventKind::is_modify(&event.kind)
            {
                tx.send(()).ok();
            }
        })
        .unwrap();

    watcher
        .watch(Path::new(&path), RecursiveMode::NonRecursive)
        .ok();
    (rx, watcher)
}

#[macroquad::main("Zappy PoC")]
async fn main() {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config).unwrap();

    let plugin_name = "plugin_cube";

    let (reload_rx, _watcher) = setup_watcher(plugin_name);

    let mut plugin = Plugin::load(&engine, plugin_name).unwrap_or_else(|_| {
        panic!(
            "{} {} {}",
            "[ERROR]".red().bold(),
            "loading plugin".bright_black(),
            plugin_name.bright_black().underline(),
        )
    });

    println!(
        "{} {}",
        "[READY]".bright_green().bold(),
        "Core".bright_black()
    );

    loop {
        clear_background(MQColor::new(0.1, 0.1, 0.12, 1.0));

        if reload_rx.try_recv().is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(50));
            match Plugin::load(&engine, plugin_name) {
                Ok(new_plugin) => {
                    plugin = new_plugin;
                    println!(
                        "{} {} {}",
                        "[HOT RELOAD]".bright_purple().bold(),
                        plugin_name.bright_black().underline(),
                        "reloaded!".bright_black()
                    );
                }
                Err(e) => println!(
                    "{} {} {}{} {}",
                    "[HOT RELOAD]".red().bold(),
                    "Waiting for".bright_black(),
                    plugin_name.bright_black().underline(),
                    "'s compilation:".bright_black(),
                    e.to_string().bright_black(),
                ),
            }
        }

        let input_state = CubeState {
            time: get_time() as f32,
        };

        if let Ok(cmd) = plugin
            .bindings
            .call_update_cube(&mut plugin.store, input_state)
        {
            let cx = screen_width() / 2.0;
            let cy = screen_height() / 2.0;

            draw_rectangle_ex(
                cx + cmd.x - 25.0,
                cy + cmd.y - 25.0,
                50.0,
                50.0,
                DrawRectangleParams {
                    rotation: cmd.rotation,
                    color: SKYBLUE,
                    ..Default::default()
                },
            );
        }

        next_frame().await;
    }
}
