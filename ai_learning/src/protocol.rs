#[derive(Debug, Clone)]
pub enum ServerResponse {
    Ok,
    Ko,
    Dead,
    Message { direction: u8, text: String },
    Eject { direction: u8 },
    Inventory {
        food: u32,
        linemate: u32,
        deraumere: u32,
        sibur: u32,
        mendiane: u32,
        phiras: u32,
        thystame: u32,
    },
    Look(Vec<TileView>),
    ConnectNbr(u32),
    ElevationUnderway,
    CurrentLevel(u32),
}

#[derive(Debug, Clone, Default)]
pub struct TileView {
    pub players: u32,
    pub food: u32,
    pub linemate: u32,
    pub deraumere: u32,
    pub sibur: u32,
    pub mendiane: u32,
    pub phiras: u32,
    pub thystame: u32,
}

impl TileView {
    pub fn resources(&self) -> [u32; 7] {
        [
            self.food,
            self.linemate,
            self.deraumere,
            self.sibur,
            self.mendiane,
            self.phiras,
            self.thystame,
        ]
    }
}

pub fn parse_response(line: &str) -> Option<ServerResponse> {
    let line = line.trim();

    if line == "ok" {
        return Some(ServerResponse::Ok);
    }
    if line == "ko" {
        return Some(ServerResponse::Ko);
    }
    if line == "dead" {
        return Some(ServerResponse::Dead);
    }
    if line == "Elevation underway" {
        return Some(ServerResponse::ElevationUnderway);
    }
    if let Some(rest) = line.strip_prefix("Current level: ") {
        if let Ok(n) = rest.trim().parse::<u32>() {
            return Some(ServerResponse::CurrentLevel(n));
        }
    }
    if let Some(rest) = line.strip_prefix("message ") {
        // "message K, text"
        let mut parts = rest.splitn(2, ", ");
        let k: u8 = parts.next()?.parse().ok()?;
        let text = parts.next().unwrap_or("").to_string();
        return Some(ServerResponse::Message { direction: k, text });
    }
    if let Some(rest) = line.strip_prefix("eject: ") {
        let k: u8 = rest.trim().parse().ok()?;
        return Some(ServerResponse::Eject { direction: k });
    }
    if let Ok(n) = line.parse::<u32>() {
        return Some(ServerResponse::ConnectNbr(n));
    }
    if line.starts_with('[') && line.ends_with(']') {
        let is_inventory = line.split(',').any(|part| {
            let t = part.trim();
            let mut words = t.splitn(2, ' ');
            let key = words.next().unwrap_or("");
            let rest = words.next().unwrap_or("").trim();
            matches!(key, "food"|"linemate"|"deraumere"|"sibur"|"mendiane"|"phiras"|"thystame")
                && rest.parse::<u32>().is_ok()
        });
        if is_inventory {
            return parse_inventory(line);
        } else {
            return Some(ServerResponse::Look(parse_look(line)));
        }
    }

    None
}

fn parse_inventory(line: &str) -> Option<ServerResponse> {
    let inner = line.trim_start_matches('[').trim_end_matches(']');
    let mut inv = ServerResponse::Inventory {
        food: 0, linemate: 0, deraumere: 0,
        sibur: 0, mendiane: 0, phiras: 0, thystame: 0,
    };
    for part in inner.split(',') {
        let part = part.trim();
        let mut kv = part.splitn(2, ' ');
        let key = kv.next()?.trim();
        let val: u32 = kv.next()?.trim().parse().ok()?;
        match key {
            "food"      => { if let ServerResponse::Inventory { food,      .. } = &mut inv { *food      = val; } }
            "linemate"  => { if let ServerResponse::Inventory { linemate,  .. } = &mut inv { *linemate  = val; } }
            "deraumere" => { if let ServerResponse::Inventory { deraumere, .. } = &mut inv { *deraumere = val; } }
            "sibur"     => { if let ServerResponse::Inventory { sibur,     .. } = &mut inv { *sibur     = val; } }
            "mendiane"  => { if let ServerResponse::Inventory { mendiane,  .. } = &mut inv { *mendiane  = val; } }
            "phiras"    => { if let ServerResponse::Inventory { phiras,    .. } = &mut inv { *phiras    = val; } }
            "thystame"  => { if let ServerResponse::Inventory { thystame,  .. } = &mut inv { *thystame  = val; } }
            _ => {}
        }
    }
    Some(inv)
}

fn parse_look(line: &str) -> Vec<TileView> {
    let inner = line.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(|tile_str| {
            let mut tile = TileView::default();
            for token in tile_str.split_whitespace() {
                match token {
                    "player"    => tile.players    += 1,
                    "food"      => tile.food       += 1,
                    "linemate"  => tile.linemate   += 1,
                    "deraumere" => tile.deraumere  += 1,
                    "sibur"     => tile.sibur      += 1,
                    "mendiane"  => tile.mendiane   += 1,
                    "phiras"    => tile.phiras     += 1,
                    "thystame"  => tile.thystame   += 1,
                    _ => {}
                }
            }
            tile
        })
        .collect()
}

pub fn action_to_command(action: usize) -> &'static str {
    match action {
        0  => "Incantation",
        1  => "Eject",
        2  => "Forward",
        3  => "Left",
        4  => "Right",
        5  => "Take food",
        6  => "Take linemate",
        7  => "Take deraumere",
        8  => "Take sibur",
        9  => "Take mendiane",
        10 => "Take phiras",
        11 => "Take thystame",
        12 => "Set food",
        13 => "Set linemate",
        14 => "Set deraumere",
        15 => "Set sibur",
        16 => "Set mendiane",
        17 => "Set phiras",
        18 => "Set thystame",
        19 => "Broadcast zappy",
        20 => "Take food",
        21 => "Fork",
        _  => "Forward",
    }
}