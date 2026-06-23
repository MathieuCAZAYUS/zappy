mod graphic;
mod input_manager;
mod manager;
mod model_loader;
mod module;
mod watcher;

use colored::*;
use macroquad::prelude::{Color as MQColor, *};
use manager::ModuleManager;
use module::ResponseCommand;
use wasmtime::{Config, Engine};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::{model_loader::TextureRegistry, module::RenderCommand};

const MAX_CMDS_CAPACITY: usize = 128;
const LOAD_EVENT_NAME: &str = "obj_viewer:load_full_scene";

#[macro_export]
macro_rules! log_all {
    ($manager:expr, $($arg:tt)*) => {{
        let msg = format!($($arg)*);
        println!("{msg}");
        if let Ok(mut s) = $manager.shared.lock() {
            s.logs_to_broadcast.push($crate::module::parse_ansi_colors(&msg));
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

fn render_scene_pipeline(
    manager: &mut ModuleManager,
    all_cmds: &[Vec<RenderCommand>],
    reg: &TextureRegistry,
) {
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
                graphic::render_3d_command(manager, cmd, reg);
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
                    "{} {} {}{} {:?}",
                    "[CRASH]".red().bold(),
                    "Shutting down".bright_black(),
                    m.name.italic().bright_blue().bold(),
                    ":".bright_black(),
                    e.to_string().bright_black(),
                );

                if let Ok(mut s) = m.store.data().shared.lock() {
                    s.cleanup_module(&m.name);
                }

                false
            }
        }
    });
}

fn load_models(manager: &mut ModuleManager) -> bool {
    let models = if let Ok(s) = manager.shared.lock() {
        s.models.clone()
    } else {
        Vec::new()
    };

    for json_payload in models {
        log_all!(
            manager,
            "{} {}",
            "[INFO]".bright_magenta().bold(),
            "Sending load_full_scene...".bright_black(),
        );

        if let Ok(mut s) = manager.shared.lock() {
            s.event_queue
                .push((LOAD_EVENT_NAME.to_string(), json_payload));
        }
    }

    true
}

fn start_network_thread(
    host: String,
    port: u16,
    shared: Arc<Mutex<manager::SharedEngineState>>,
) {
    std::thread::spawn(move || {
        let addr = format!("{}:{}", host, port);

        let stream = match TcpStream::connect(&addr) {
            Ok(s) => {
                println!("[NET] Connecté au serveur {}", addr);
                s
            }
            Err(e) => {
                println!("[NET] Impossible de se connecter à {} : {}", addr, e);
                return;
            }
        };
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = stream;

        let mut welcome = String::new();
        if reader.read_line(&mut welcome).is_err() || !welcome.contains("WELCOME") {
            println!("[NET] Handshake échoué, réponse reçue : {:?}", welcome);
            return;
        }
        println!("[NET] Handshake OK : {:?}", welcome.trim());

        if let Err(e) = writer.write_all(b"GRAPHIC\n") {
            println!("[NET] Erreur envoi GRAPHIC : {}", e);
            return;
        }
        if let Err(e) = writer.write_all(b"mct\n") {
            println!("[NET] Erreur envoi mct : {}", e);
            return;
        }
        println!("[NET] mct envoyé → attente du contenu de la map...");
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    println!("[NET] Connexion fermée par le serveur.");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }
                    println!("[NET] Reçu : {}", trimmed);
                    if let Ok(mut s) = shared.lock() {
                        s.event_queue.push(("server:line".to_string(), trimmed));
                    }
                }
                Err(e) => {
                    println!("[NET] Erreur lecture : {}", e);
                    break;
                }
            }
        }
    });
}

#[macroquad::main("Zappy")]
async fn main() -> Result<(), anyhow::Error> {
    colored::control::set_override(true);

    let args: Vec<String> = std::env::args().collect();
    let mut port: u16 = 4242;
    let mut host = String::from("localhost");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                if let Some(v) = args.get(i + 1) {
                    port = v.parse().expect("Port invalide");
                    i += 2;
                }
            }
            "-h" => {
                if let Some(v) = args.get(i + 1) {
                    host = v.clone();
                    i += 2;
                }
            }
            _ => { i += 1; }
        }
    }

    let mut config = Config::new();
    config.wasm_component_model(true);

    let mut manager = ModuleManager::new(Engine::new(&config)?);
    manager.scan_and_load_all();
    let (reload_rx, _watcher) = watcher::setup()?;

    start_network_thread(host, port, manager.shared.clone());

    let mut tex_reg = TextureRegistry::new();
    let models = model_loader::discover_models(&mut tex_reg);
    log_all!(
        manager,
        "{} {} {}",
        "[INFO]".bright_magenta().bold(),
        "Number of models found:".bright_black(),
        format!("{}", models.len()).bright_black().underline()
    );

    for (name, payload) in models {
        log_all!(
            manager,
            "{} {} {}",
            "[INFO]".bright_magenta().bold(),
            "Model found:".bright_black(),
            name.bright_blue().bold().italic()
        );
        match serde_json::to_string(&payload) {
            Ok(json) => {
                if let Ok(mut s) = manager.shared.lock() {
                    s.models.push(json);
                }
            }
            Err(e) => {
                log_all!(
                    manager,
                    "{} {} {:?}",
                    "[ERROR]".red().bold(),
                    "Parsing FullModelPayload:".bright_black(),
                    e.to_string().bright_black(),
                );
            }
        }
    }
    let mut input_manager = input_manager::InputManager::new();

    let mut all_cmds = Vec::with_capacity(MAX_CMDS_CAPACITY);

    log_all!(
        manager,
        "{} {}",
        "[SYSTEM]".bright_blue().bold(),
        "Core started successfully!".bright_black()
    );

    let mut loaded_models = false;

    loop {
        let input_state = input_manager.process(&mut manager.context);
        handle_module_reloads(&mut manager, &reload_rx);
        pending_commands(&mut manager);

        manager.dispatch_events();
        manager.handle_inputs(input_state);
        manager.broadcast_logs();

        all_cmds.clear();
        update_modules_pipeline(&mut manager, &mut all_cmds);
        render_scene_pipeline(&mut manager, &all_cmds, &tex_reg);

        if !loaded_models {
            loaded_models = load_models(&mut manager);
        }
        next_frame().await;
    }
}
