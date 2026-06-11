use crate::module::{EngineContext, InputAction, InputState, cube_plugin::__with_name2::Delta};
use colored::Colorize;
use macroquad::prelude::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, recommended_watcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::mpsc::{Receiver, channel},
    thread,
    time::Duration,
};

const APP_DIR_NAME: &str = "zappy";
const BINDINGS_FILENAME: &str = "bindings.json";
const FILE_SYNC_DELAY_MS: u64 = 50;
const SCROLL_DEADZONE: f32 = 0.0;
const DEFAULT_MOUSE_DELTA: f32 = 0.0;

const DEFAULT_BINDINGS_JSON: &str = r#"{
    "pressed": {
        "ToggleConsole": ["F1"],
        "Confirm": ["Enter"],
        "Delete": ["Backspace"],
        "NavigateUp": ["Up"],
        "NavigateDown": ["Down"],
        "PrimaryAction": ["MouseLeft"]
    },
    "down": {
        "MoveForward": ["W", "Up", "Z"],
        "MoveBackward": ["S", "Down"],
        "MoveLeft": ["A", "Left", "Q"],
        "MoveRight": ["D", "Right"]
    }
}"#;

#[derive(Serialize, Deserialize)]
struct KeyConfig {
    pressed: HashMap<String, Vec<String>>,
    down: HashMap<String, Vec<String>>,
}

#[derive(Clone, Copy)]
enum InputTrigger {
    Key(KeyCode),
    Mouse(MouseButton),
}

pub struct InputManager {
    pressed_bindings: Vec<(InputTrigger, InputAction)>,
    down_bindings: Vec<(InputTrigger, InputAction)>,
    config_path: PathBuf,
    watcher_rx: Receiver<notify::Result<notify::Event>>,
    _watcher: RecommendedWatcher,
}

impl InputManager {
    pub fn new() -> Self {
        let config_path = Self::resolve_config_path();
        Self::ensure_config_exists(&config_path);

        let (tx, rx) = channel();
        let mut watcher = recommended_watcher(tx).expect("Failed to initialize input watcher");

        let watch_dir = config_path.parent().expect("Invalid config directory");
        watcher
            .watch(watch_dir, RecursiveMode::NonRecursive)
            .unwrap();

        let mut manager = Self {
            pressed_bindings: Vec::new(),
            down_bindings: Vec::new(),
            config_path,
            watcher_rx: rx,
            _watcher: watcher,
        };

        manager.reload_config();
        manager
    }

    pub fn process(&mut self, context: &mut EngineContext) -> InputState {
        self.check_for_updates();
        let mut actions = Vec::new();

        for (trigger, action) in &self.pressed_bindings {
            if Self::is_trigger_pressed(trigger) {
                actions.push(*action);
            }
        }

        for (trigger, action) in &self.down_bindings {
            if Self::is_trigger_down(trigger) {
                actions.push(*action);
            }
        }

        Self::process_analog_inputs(&mut actions);
        let raw_chars = Self::collect_raw_chars();
        Self::handle_context_switching(&actions, context);
        let mouse_delta = Self::handle_cursor_and_delta(context);

        InputState {
            context: *context,
            actions,
            raw_chars,
            mouse_delta,
        }
    }

    fn check_for_updates(&mut self) {
        let mut updated = false;
        while let Ok(Ok(event)) = self.watcher_rx.try_recv() {
            if event.kind.is_modify() && event.paths.contains(&self.config_path) {
                updated = true;
            }
        }
        if updated {
            thread::sleep(Duration::from_millis(FILE_SYNC_DELAY_MS));
            self.reload_config();
        }
    }

    fn reload_config(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.config_path)
            && let Ok(config) = serde_json::from_str::<KeyConfig>(&content)
        {
            self.pressed_bindings = Self::compile_bindings(&config.pressed);
            self.down_bindings = Self::compile_bindings(&config.down);
            println!(
                "{} {}",
                "[INPUT]".magenta().bold(),
                "Bindings successfully reloaded from config!".bright_black()
            );
        }
    }

    fn compile_bindings(map: &HashMap<String, Vec<String>>) -> Vec<(InputTrigger, InputAction)> {
        let mut compiled = Vec::new();
        for (action_str, keys) in map {
            if let Some(action) = Self::parse_action(action_str) {
                for key_str in keys {
                    if let Some(trigger) = Self::parse_trigger(key_str) {
                        compiled.push((trigger, action));
                    }
                }
            }
        }
        compiled
    }

    fn resolve_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(APP_DIR_NAME);
        fs::create_dir_all(&path).ok();
        path.push(BINDINGS_FILENAME);
        path
    }

    fn ensure_config_exists(path: &PathBuf) {
        if !path.exists() {
            fs::write(path, DEFAULT_BINDINGS_JSON).expect("Failed to write default config");
        }
    }

    fn is_trigger_pressed(t: &InputTrigger) -> bool {
        match t {
            InputTrigger::Key(k) => is_key_pressed(*k),
            InputTrigger::Mouse(m) => is_mouse_button_pressed(*m),
        }
    }

    fn is_trigger_down(t: &InputTrigger) -> bool {
        match t {
            InputTrigger::Key(k) => is_key_down(*k),
            InputTrigger::Mouse(m) => is_mouse_button_down(*m),
        }
    }

    fn process_analog_inputs(actions: &mut Vec<InputAction>) {
        let (_, scroll_y) = mouse_wheel();
        if scroll_y > SCROLL_DEADZONE {
            actions.push(InputAction::ScrollUp);
        } else if scroll_y < -SCROLL_DEADZONE {
            actions.push(InputAction::ScrollDown);
        }
    }

    fn collect_raw_chars() -> String {
        let mut raw = String::new();
        while let Some(c) = get_char_pressed() {
            if !c.is_control() {
                raw.push(c);
            }
        }
        raw
    }

    fn handle_context_switching(actions: &[InputAction], context: &mut EngineContext) {
        if actions.contains(&InputAction::ToggleConsole) {
            *context = match context {
                EngineContext::Gameplay => EngineContext::UiConsole,
                EngineContext::UiConsole => EngineContext::Gameplay,
            };
        }
    }

    fn handle_cursor_and_delta(context: &EngineContext) -> Delta {
        let is_gameplay = *context == EngineContext::Gameplay;
        macroquad::input::set_cursor_grab(is_gameplay);
        macroquad::input::show_mouse(!is_gameplay);

        let d = if is_gameplay {
            mouse_delta_position()
        } else {
            Vec2::new(DEFAULT_MOUSE_DELTA, DEFAULT_MOUSE_DELTA)
        };
        Delta { x: d.x, y: d.y }
    }

    fn parse_action(s: &str) -> Option<InputAction> {
        match s {
            "ToggleConsole" => Some(InputAction::ToggleConsole),
            "Confirm" => Some(InputAction::Confirm),
            "Delete" => Some(InputAction::Delete),
            "NavigateUp" => Some(InputAction::NavigateUp),
            "NavigateDown" => Some(InputAction::NavigateDown),
            "MoveForward" => Some(InputAction::MoveForward),
            "MoveBackward" => Some(InputAction::MoveBackward),
            "MoveLeft" => Some(InputAction::MoveLeft),
            "MoveRight" => Some(InputAction::MoveRight),
            "PrimaryAction" => Some(InputAction::PrimaryAction),
            _ => None,
        }
    }

    fn parse_trigger(s: &str) -> Option<InputTrigger> {
        match s.to_uppercase().as_str() {
            "MOUSELEFT" => Some(InputTrigger::Mouse(MouseButton::Left)),
            "MOUSERIGHT" => Some(InputTrigger::Mouse(MouseButton::Right)),
            "F1" => Some(InputTrigger::Key(KeyCode::F1)),
            "ENTER" => Some(InputTrigger::Key(KeyCode::Enter)),
            "BACKSPACE" => Some(InputTrigger::Key(KeyCode::Backspace)),
            "UP" => Some(InputTrigger::Key(KeyCode::Up)),
            "DOWN" => Some(InputTrigger::Key(KeyCode::Down)),
            "LEFT" => Some(InputTrigger::Key(KeyCode::Left)),
            "RIGHT" => Some(InputTrigger::Key(KeyCode::Right)),
            "W" => Some(InputTrigger::Key(KeyCode::W)),
            "A" => Some(InputTrigger::Key(KeyCode::A)),
            "S" => Some(InputTrigger::Key(KeyCode::S)),
            "D" => Some(InputTrigger::Key(KeyCode::D)),
            "Q" => Some(InputTrigger::Key(KeyCode::Q)),
            "Z" => Some(InputTrigger::Key(KeyCode::Z)),
            _ => None,
        }
    }
}
