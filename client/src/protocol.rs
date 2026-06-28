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
        let (direction_text, text) = rest.split_once(',').unwrap_or((rest, ""));
        let direction: u8 = direction_text.trim().parse().ok()?;
        let text = text.trim_start().to_string();
        return Some(ServerResponse::Message { direction, text });
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

// unit tests

fn parse(s: &str) -> ServerResponse {
    parse_response(s).unwrap_or_else(|| panic!("parse_response returned None for: {s:?}"))
}
 
fn assert_none(s: &str) {
    assert!(
        parse_response(s).is_none(),
        "Expected None for {s:?} but got Some"
    );
}
 
#[test]
fn test_ok() {
    assert!(matches!(parse("ok"), ServerResponse::Ok));
}
 
#[test]
fn test_ok_with_trailing_whitespace() {
    assert!(matches!(parse("ok\n"), ServerResponse::Ok));
    assert!(matches!(parse("ok  "), ServerResponse::Ok));
}
 
#[test]
fn test_ko() {
    assert!(matches!(parse("ko"), ServerResponse::Ko));
}
 
#[test]
fn test_dead() {
    assert!(matches!(parse("dead"), ServerResponse::Dead));
}
 
#[test]
fn test_elevation_underway() {
    assert!(matches!(parse("Elevation underway"), ServerResponse::ElevationUnderway));
}
 
#[test]
fn test_current_level_min() {
    assert!(matches!(parse("Current level: 1"), ServerResponse::CurrentLevel(1)));
}
 
#[test]
fn test_current_level_max() {
    assert!(matches!(parse("Current level: 8"), ServerResponse::CurrentLevel(8)));
}
 
#[test]
fn test_current_level_mid() {
    assert!(matches!(parse("Current level: 4"), ServerResponse::CurrentLevel(4)));
}
 
#[test]
fn test_message_direction_zero() {
    let r = parse("message 0, hello");
    assert!(matches!(r, ServerResponse::Message { direction: 0, .. }));
    if let ServerResponse::Message { text, .. } = r {
        assert_eq!(text, "hello");
    }
}
 
#[test]
fn test_message_direction_eight() {
    let r = parse("message 8, zappy");
    assert!(matches!(r, ServerResponse::Message { direction: 8, .. }));
    if let ServerResponse::Message { text, .. } = r {
        assert_eq!(text, "zappy");
    }
}
 
#[test]
fn test_message_empty_text() {
    let r = parse("message 3, ");
    assert!(matches!(r, ServerResponse::Message { direction: 3, .. }));
    if let ServerResponse::Message { text, .. } = r {
        assert_eq!(text, "");
    }
}
 
#[test]
fn test_message_text_with_comma() {
    let r = parse("message 2, hello, world");
    if let ServerResponse::Message { direction, text } = r {
        assert_eq!(direction, 2);
        assert_eq!(text, "hello, world");
    } else {
        panic!("Expected Message");
    }
}
 
#[test]
fn test_eject_all_directions() {
    for k in 1u8..=8 {
        let line = format!("eject: {k}");
        assert!(
            matches!(parse(&line), ServerResponse::Eject { direction } if direction == k),
            "Failed for direction {k}"
        );
    }
}
 
#[test]
fn test_connect_nbr_zero() {
    assert!(matches!(parse("0"), ServerResponse::ConnectNbr(0)));
}
 
#[test]
fn test_connect_nbr_nonzero() {
    assert!(matches!(parse("6"), ServerResponse::ConnectNbr(6)));
}

#[test]
fn test_inventory_all_zero() {
    let r = parse("[food 0, linemate 0, deraumere 0, sibur 0, mendiane 0, phiras 0, thystame 0]");
    if let ServerResponse::Inventory { food, linemate, deraumere, sibur, mendiane, phiras, thystame } = r {
        assert_eq!(food, 0);
        assert_eq!(linemate, 0);
        assert_eq!(deraumere, 0);
        assert_eq!(sibur, 0);
        assert_eq!(mendiane, 0);
        assert_eq!(phiras, 0);
        assert_eq!(thystame, 0);
    } else {
        panic!("Expected Inventory");
    }
}
 
#[test]
fn test_inventory_typical() {
    let r = parse("[food 9, linemate 0, deraumere 0, sibur 0, mendiane 0, phiras 0, thystame 0]");
    if let ServerResponse::Inventory { food, linemate, .. } = r {
        assert_eq!(food, 9);
        assert_eq!(linemate, 0);
    } else {
        panic!("Expected Inventory");
    }
}
 
#[test]
fn test_inventory_all_nonzero() {
    let r = parse("[food 3, linemate 1, deraumere 2, sibur 1, mendiane 4, phiras 2, thystame 1]");
    if let ServerResponse::Inventory { food, linemate, deraumere, sibur, mendiane, phiras, thystame } = r {
        assert_eq!(food, 3);
        assert_eq!(linemate, 1);
        assert_eq!(deraumere, 2);
        assert_eq!(sibur, 1);
        assert_eq!(mendiane, 4);
        assert_eq!(phiras, 2);
        assert_eq!(thystame, 1);
    } else {
        panic!("Expected Inventory");
    }
}
 
#[test]
fn test_inventory_with_spaces_after_comma() {
    let r = parse("[food 345, sibur 3, phiras 5,deraumere 0, linemate 0, mendiane 0, thystame 0]");
    if let ServerResponse::Inventory { food, sibur, phiras, deraumere, .. } = r {
        assert_eq!(food, 345);
        assert_eq!(sibur, 3);
        assert_eq!(phiras, 5);
        assert_eq!(deraumere, 0);
    } else {
        panic!("Expected Inventory");
    }
}
 
#[test]
fn test_look_level1_empty_tiles() {
    let r = parse("[player,,,]");
    if let ServerResponse::Look(tiles) = r {
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0].players, 1);
        assert_eq!(tiles[1].players, 0);
        assert_eq!(tiles[1].food, 0);
    } else {
        panic!("Expected Look");
    }
}
 
#[test]
fn test_look_with_resources_on_tile() {
    let r = parse("[player deraumere,, food mendiane, food food mendiane phiras]");
    if let ServerResponse::Look(tiles) = r {
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0].players, 1);
        assert_eq!(tiles[0].deraumere, 1);
        assert_eq!(tiles[2].food, 1);
        assert_eq!(tiles[2].mendiane, 1);
        assert_eq!(tiles[3].food, 2);
        assert_eq!(tiles[3].mendiane, 1);
        assert_eq!(tiles[3].phiras, 1);
    } else {
        panic!("Expected Look");
    }
}
 
#[test]
fn test_look_multiple_players_on_tile() {
    let r = parse("[player player deraumere,,]");
    if let ServerResponse::Look(tiles) = r {
        assert_eq!(tiles[0].players, 2);
        assert_eq!(tiles[0].deraumere, 1);
    } else {
        panic!("Expected Look");
    }
}
 
#[test]
fn test_look_not_confused_with_inventory_containing_food() {
    let r = parse("[player thystame, deraumere phiras, food, food]");
    assert!(
        matches!(r, ServerResponse::Look(_)),
        "Should be Look, not Inventory — tile content 'food' is not 'food N'"
    );
    if let ServerResponse::Look(tiles) = r {
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0].thystame, 1);
        assert_eq!(tiles[2].food, 1);
        assert_eq!(tiles[3].food, 1);
    }
}
 
#[test]
fn test_look_all_stone_types() {
    let r = parse("[linemate deraumere sibur mendiane phiras thystame food player]");
    if let ServerResponse::Look(tiles) = r {
        let t = &tiles[0];
        assert_eq!(t.linemate,  1);
        assert_eq!(t.deraumere, 1);
        assert_eq!(t.sibur,     1);
        assert_eq!(t.mendiane,  1);
        assert_eq!(t.phiras,    1);
        assert_eq!(t.thystame,  1);
        assert_eq!(t.food,      1);
        assert_eq!(t.players,   1);
    } else {
        panic!("Expected Look");
    }
}
 
#[test]
fn test_look_leading_space_in_brackets() {
    let r = parse("[ player deraumere,, food mendiane, food food mendiane phiras ]");
    assert!(matches!(r, ServerResponse::Look(_)), "Should parse even with leading/trailing space inside brackets");
}

#[test]
fn test_empty_line_returns_none() {
    assert_none("");
}
 
#[test]
fn test_whitespace_only_returns_none() {
    assert_none("   ");
}
 
#[test]
fn test_garbage_returns_none() {
    assert_none("xyzzy");
    assert_none("HELLO");
    assert_none("forward");
}

#[test]
fn test_all_22_actions_produce_non_empty_command() {
    for i in 0..22 {
        let cmd = action_to_command(i);
        assert!(!cmd.is_empty(), "Action {i} produced empty command");
    }
}
 
#[test]
fn test_action_commands_are_valid_server_commands() {
    for i in 0..22 {
        let cmd = action_to_command(i);
        let first = cmd.chars().next().unwrap();
        assert!(
            first.is_uppercase(),
            "Action {i} → {cmd:?} doesn't start with uppercase"
        );
    }
}
 
#[test]
fn test_action_incantation() {
    assert_eq!(action_to_command(0), "Incantation");
}
 
#[test]
fn test_action_forward() {
    assert_eq!(action_to_command(2), "Forward");
}
 
#[test]
fn test_action_take_food() {
    assert_eq!(action_to_command(5), "Take food");
}
 
#[test]
fn test_action_set_thystame() {
    assert_eq!(action_to_command(18), "Set thystame");
}
 
#[test]
fn test_action_fork() {
    assert_eq!(action_to_command(21), "Fork");
}
 
#[test]
fn test_action_out_of_range_falls_back_to_forward() {
    assert_eq!(action_to_command(99), "Forward");
    assert_eq!(action_to_command(22), "Forward");
}

#[test]
fn test_tile_resources_order() {
    let mut t = TileView::default();
    t.food      = 1;
    t.linemate  = 2;
    t.deraumere = 3;
    t.sibur     = 4;
    t.mendiane  = 5;
    t.phiras    = 6;
    t.thystame  = 7;
    let r = t.resources();
    assert_eq!(r, [1, 2, 3, 4, 5, 6, 7]);
}
