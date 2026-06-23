wit_bindgen::generate!({
    path: "../../wit",
    world: "ui-world",
});

use std::sync::Mutex;
use crate::local::zappy::host_api::{host_subscribe, emit_event};

#[derive(Clone)]
struct Tile {
    food:      u32,
    linemate:  u32,
    deraumere: u32,
    sibur:     u32,
    mendiane:  u32,
    phiras:    u32,
    thystame:  u32,
}

impl Tile {
    fn empty() -> Self {
        Self { food: 0, linemate: 0, deraumere: 0, sibur: 0, mendiane: 0, phiras: 0, thystame: 0 }
    }
}

#[derive(Clone)]
struct Player {
    id:        u32,
    x:         u32,
    y:         u32,
    direction: u32,
    level:     u32,
    team:      String,
    inv_food:      u32,
    inv_linemate:  u32,
    inv_deraumere: u32,
    inv_sibur:     u32,
    inv_mendiane:  u32,
    inv_phiras:    u32,
    inv_thystame:  u32,
}

struct GameState {
    width:   u32,
    height:  u32,
    freq:    u32,
    tiles:   Vec<Tile>,
    players: Vec<Player>,
    teams:   Vec<String>,
}

impl GameState {
    fn new() -> Self {
        Self {
            width: 0, height: 0, freq: 0,
            tiles: Vec::new(),
            players: Vec::new(),
            teams: Vec::new(),
        }
    }

    fn tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if self.width == 0 { return None; }
        let idx = (y * self.width + x) as usize;
        self.tiles.get_mut(idx)
    }
}

static STATE: Mutex<GameState> = Mutex::new(GameState {
    width: 0, height: 0, freq: 0,
    tiles: Vec::new(),
    players: Vec::new(),
    teams: Vec::new(),
});
static INITIALIZED: Mutex<bool> = Mutex::new(false);

struct Module;

fn parse_line(line: &str, state: &mut GameState) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() { return; }

    match parts[0] {

        "msz" if parts.len() == 3 => {
            let w = parts[1].parse().unwrap_or(0);
            let h = parts[2].parse().unwrap_or(0);
            state.width  = w;
            state.height = h;
            state.tiles  = vec![Tile::empty(); (w * h) as usize];
            emit_event("zappy:map_size", &format!("{} {}", w, h));
        }

        "sgt" if parts.len() == 2 => {
            state.freq = parts[1].parse().unwrap_or(0);
        }

        "bct" if parts.len() == 10 => {
            let x: u32 = parts[1].parse().unwrap_or(0);
            let y: u32 = parts[2].parse().unwrap_or(0);
            if let Some(tile) = state.tile_mut(x, y) {
                tile.food      = parts[3].parse().unwrap_or(0);
                tile.linemate  = parts[4].parse().unwrap_or(0);
                tile.deraumere = parts[5].parse().unwrap_or(0);
                tile.sibur     = parts[6].parse().unwrap_or(0);
                tile.mendiane  = parts[7].parse().unwrap_or(0);
                tile.phiras    = parts[8].parse().unwrap_or(0);
                tile.thystame  = parts[9].parse().unwrap_or(0);
            }
            emit_event("zappy:tile_update", &format!("{} {}", x, y));
        }

        "tna" if parts.len() == 2 => {
            state.teams.push(parts[1].to_string());
            emit_event("zappy:team", parts[1]);
        }

        "pnw" if parts.len() == 7 => {
            let id_str = parts[1].trim_start_matches('#');
            let id: u32 = id_str.parse().unwrap_or(0);
            let x: u32  = parts[2].parse().unwrap_or(0);
            let y: u32  = parts[3].parse().unwrap_or(0);
            let dir: u32 = parts[4].parse().unwrap_or(1);
            let lvl: u32 = parts[5].parse().unwrap_or(1);
            let team     = parts[6].to_string();

            state.players.retain(|p| p.id != id);
            state.players.push(Player { id, x, y, direction: dir, level: lvl, team, 
                inv_food: 0, inv_linemate: 0, inv_deraumere: 0, inv_sibur: 0, 
                inv_mendiane: 0, inv_phiras: 0, inv_thystame: 0,});
            emit_event("zappy:player_new", &format!("{} {} {}", id, x, y));
        }

        "ppo" if parts.len() == 5 => {
            let id_str = parts[1].trim_start_matches('#');
            let id: u32 = id_str.parse().unwrap_or(0);
            let x: u32  = parts[2].parse().unwrap_or(0);
            let y: u32  = parts[3].parse().unwrap_or(0);
            let dir: u32 = parts[4].parse().unwrap_or(1);
            if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
                p.x = x; p.y = y; p.direction = dir;
            }
            emit_event("zappy:player_move", &format!("{} {} {}", id, x, y));
        }

        "plv" if parts.len() == 3 => {
            let id_str = parts[1].trim_start_matches('#');
            let id: u32 = id_str.parse().unwrap_or(0);
            let lvl: u32 = parts[2].parse().unwrap_or(1);
            if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
                p.level = lvl;
            }
            emit_event("zappy:player_level", &format!("{} {}", id, lvl));
        }

        "pdi" if parts.len() == 2 => {
            let id_str = parts[1].trim_start_matches('#');
            let id: u32 = id_str.parse().unwrap_or(0);
            state.players.retain(|p| p.id != id);
            emit_event("zappy:player_death", &id.to_string());
        }

        "seg" if parts.len() == 2 => {
            emit_event("zappy:game_over", parts[1]);
        }
        "pin" if parts.len() == 11 => {
            let id: u32 = parts[1].trim_start_matches('#').parse().unwrap_or(0);
            if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
                p.inv_food      = parts[3].parse().unwrap_or(0);
                p.inv_linemate  = parts[4].parse().unwrap_or(0);
                p.inv_deraumere = parts[5].parse().unwrap_or(0);
                p.inv_sibur     = parts[6].parse().unwrap_or(0);
                p.inv_mendiane  = parts[7].parse().unwrap_or(0);
                p.inv_phiras    = parts[8].parse().unwrap_or(0);
                p.inv_thystame  = parts[9].parse().unwrap_or(0);
            }
            emit_event("zappy:player_inventory", parts[1].trim_start_matches('#'));
        }
        "pex" if parts.len() == 2 => {
            let id: u32 = parts[1].trim_start_matches('#').parse().unwrap_or(0);
            emit_event("zappy:player_expulsion", &id.to_string());
        }
        "pbc" if parts.len() >= 3 => {
            let id: u32 = parts[1].trim_start_matches('#').parse().unwrap_or(0);
            let msg = parts[2..].join(" ");
            emit_event("zappy:player_broadcast", &format!("{} {}", id, msg));
        }
        "pic" if parts.len() >= 4 => {
            let x: u32   = parts[1].parse().unwrap_or(0);
            let y: u32   = parts[2].parse().unwrap_or(0);
            let lvl: u32 = parts[3].parse().unwrap_or(0);
            emit_event("zappy:incantation_start", &format!("{} {} {}", x, y, lvl));
        }
        "pie" if parts.len() == 4 => {
            let x: u32 = parts[1].parse().unwrap_or(0);
            let y: u32 = parts[2].parse().unwrap_or(0);
            emit_event("zappy:incantation_end", &format!("{} {} {}", x, y, parts[3]));
        }
        "pfk" if parts.len() == 2 => {
            let id: u32 = parts[1].trim_start_matches('#').parse().unwrap_or(0);
            emit_event("zappy:player_fork", &id.to_string());
        }
        "pdr" if parts.len() == 3 => {
            let id: u32  = parts[1].trim_start_matches('#').parse().unwrap_or(0);
            let res: u32 = parts[2].parse().unwrap_or(0);
            emit_event("zappy:player_drop", &format!("{} {}", id, res));
        }
        "pgt" if parts.len() == 3 => {
            let id: u32  = parts[1].trim_start_matches('#').parse().unwrap_or(0);
            let res: u32 = parts[2].parse().unwrap_or(0);
            emit_event("zappy:player_take", &format!("{} {}", id, res));
        }
        "enw" if parts.len() == 5 => {
            let egg_id: u32    = parts[1].trim_start_matches('#').parse().unwrap_or(0);
            let player_id: u32 = parts[2].trim_start_matches('#').parse().unwrap_or(0);
            let x: u32 = parts[3].parse().unwrap_or(0);
            let y: u32 = parts[4].parse().unwrap_or(0);
            emit_event("zappy:egg_laid", &format!("{} {} {} {}", egg_id, player_id, x, y));
        }
        "ebo" if parts.len() == 2 => {
            let egg_id: u32 = parts[1].trim_start_matches('#').parse().unwrap_or(0);
            emit_event("zappy:egg_hatched", &egg_id.to_string());
        }
        "edi" if parts.len() == 2 => {
            let egg_id: u32 = parts[1].trim_start_matches('#').parse().unwrap_or(0);
            emit_event("zappy:egg_dead", &egg_id.to_string());
        }
        "sst" if parts.len() == 2 => {
            state.freq = parts[1].parse().unwrap_or(state.freq);
            emit_event("zappy:freq_update", &state.freq.to_string());
        }
        "smg" if parts.len() >= 2 => {
            emit_event("zappy:server_message", &parts[1..].join(" "));
        }
        "suc" => { emit_event("zappy:unknown_command", ""); }
        "sbp" => { emit_event("zappy:bad_parameter", ""); }

        _ => {}
    }
}

impl Guest for Module {
    fn serialize() -> Vec<u8> { Vec::new() }
    fn deserialize(_: Vec<u8>) {}
    fn handle_input(_: InputState) {}
    fn get_commands() -> Vec<CommandDesc> { Vec::new() }
    fn run_command(_: String, _: Vec<String>) -> ResponseCommand { ResponseCommand::Unknown }
    fn accept_log(_: Vec<TextSegment>) {}

    fn update_module(_time: f32, _dt: f32, _w: f32, _h: f32) -> Vec<RenderCommand> {
        let mut init = INITIALIZED.lock().unwrap();
        if !*init {
            host_subscribe("server:line");
            *init = true;
        }
        Vec::new()
    }

    fn handle_event(event_name: String, payload: String) {
        if event_name == "server:line" {
            let mut state = STATE.lock().unwrap();
            parse_line(&payload, &mut state);
        }
    }
}

export!(Module);