use crate::protocol::TileView;
 
pub const MAX_VISION_TILES: usize = 64;
pub const N_RESOURCES: usize = 7;
pub const N_ACTIONS: usize = 22;
 
pub const TICKS_PER_FOOD: u32 = 126;
pub const MAX_SURVIVAL_TICKS: u32 = TICKS_PER_FOOD * 10;
pub const MAX_LEVEL: u32 = 8;
pub const N_SOUND_DIRECTIONS: usize = 9;
pub const SOUND_DIM: usize = 1 + N_SOUND_DIRECTIONS;
pub const INV_NORM_CAP: u32 = 20;
pub const VIS_NORM_CAP: u32 = 10;
 
pub const STATE_DIM: usize =
    2 + 4 + 1 + 1 + N_RESOURCES + SOUND_DIM + MAX_VISION_TILES * N_RESOURCES;
 
pub struct GameState {
    pub x: f32,
    pub y: f32,
    pub direction: u8,
    pub level: u32,
    pub survival_ticks: u32,
    pub inventory: [u32; 7],
    pub last_message: Option<u8>,
    pub look_tiles: Vec<TileView>,
    pub map_w: u32,
    pub map_h: u32,
}
 
impl GameState {
    pub fn new(map_w: u32, map_h: u32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            direction: 0,
            level: 1,
            survival_ticks: MAX_SURVIVAL_TICKS,
            inventory: [0; 7],
            last_message: None,
            look_tiles: Vec::new(),
            map_w,
            map_h,
        }
    }
 
    pub fn to_state_vector(&mut self) -> Vec<f32> {
        let mut v = Vec::with_capacity(STATE_DIM);
 
        v.push(self.x);
        v.push(self.y);
 
        let mut dir = [0.0f32; 4];
        dir[self.direction as usize] = 1.0;
        v.extend_from_slice(&dir);
 
        v.push(self.level as f32 / MAX_LEVEL as f32);
 
        v.push((self.survival_ticks.min(MAX_SURVIVAL_TICKS)) as f32 / MAX_SURVIVAL_TICKS as f32);
 
        for &qty in &self.inventory {
            v.push((qty.min(INV_NORM_CAP)) as f32 / INV_NORM_CAP as f32);
        }
 
        let mut sound = [0.0f32; SOUND_DIM];
        if let Some(dir) = self.last_message.take() {
            sound[0] = 1.0;
            if (dir as usize) < N_SOUND_DIRECTIONS {
                sound[1 + dir as usize] = 1.0;
            }
        }
        v.extend_from_slice(&sound);
 
        let n_visible = ((self.level + 1) * (self.level + 1)) as usize;
        let mut idx = 0;
        for tile_i in 0..MAX_VISION_TILES {
            if tile_i < self.look_tiles.len().min(n_visible) {
                let t = &self.look_tiles[tile_i];
                let res = t.resources();
                for &r in &res {
                    v.push((r.min(VIS_NORM_CAP)) as f32 / VIS_NORM_CAP as f32);
                }
            } else {
                for _ in 0..N_RESOURCES {
                    v.push(0.0);
                }
            }
            idx += 1;
        }
        let _ = idx;
 
        debug_assert_eq!(v.len(), STATE_DIM, "State vector length mismatch");
        v
    }
 
    pub fn valid_mask(&self) -> [bool; N_ACTIONS] {
        let mut mask = [true; N_ACTIONS];
 
        let tile = self.look_tiles.first();
 
        mask[0] = true;
 
        mask[1] = tile.map(|t| t.players > 1).unwrap_or(false);
 
        if let Some(t) = tile {
            mask[5]  = t.food      > 0;
            mask[6]  = t.linemate  > 0;
            mask[7]  = t.deraumere > 0;
            mask[8]  = t.sibur     > 0;
            mask[9]  = t.mendiane  > 0;
            mask[10] = t.phiras    > 0;
            mask[11] = t.thystame  > 0;
        } else {
            for i in 5..=11 { mask[i] = false; }
        }
 
        mask[12] = self.inventory[0] > 0;
        mask[13] = self.inventory[1] > 0;
        mask[14] = self.inventory[2] > 0;
        mask[15] = self.inventory[3] > 0;
        mask[16] = self.inventory[4] > 0;
        mask[17] = self.inventory[5] > 0;
        mask[18] = self.inventory[6] > 0;
 
        mask[20] = self.inventory[0] > 0;
 
        mask[21] = true;
 
        mask
    }
 
    pub fn set_position(&mut self, x: u32, y: u32, orientation: u8) {
        self.x = x as f32 / self.map_w as f32;
        self.y = y as f32 / self.map_h as f32;
        self.direction = orientation.saturating_sub(1).min(3);
    }
}

// unit tests

fn empty_state() -> GameState {
    GameState::new(10, 10)
}
 
fn tile_with(food: u32, linemate: u32) -> TileView {
    let mut t = TileView::default();
    t.food = food;
    t.linemate = linemate;
    t
}
 
#[test]
fn test_state_dim_constant_is_473() {
    assert_eq!(STATE_DIM, 473);
}

#[test]
fn test_state_vector_length_matches_dim() {
    let mut s = empty_state();
    let v = s.to_state_vector();
    assert_eq!(v.len(), STATE_DIM);
}

#[test]
fn test_state_vector_length_with_look_tiles() {
    let mut s = empty_state();
    s.look_tiles = vec![TileView::default(); 4];
    let v = s.to_state_vector();
    assert_eq!(v.len(), STATE_DIM);
}

#[test]
fn test_position_origin() {
    let mut s = empty_state();
    s.x = 0.0;
    s.y = 0.0;
    let v = s.to_state_vector();
    assert_eq!(v[0], 0.0);
    assert_eq!(v[1], 0.0);
}

#[test]
fn test_position_normalised() {
    let mut s = GameState::new(20, 10);
    s.set_position(10, 5, 1);
    let v = s.to_state_vector();
    assert!((v[0] - 0.5).abs() < 1e-6, "x should be 0.5, got {}", v[0]);
    assert!((v[1] - 0.5).abs() < 1e-6, "y should be 0.5, got {}", v[1]);
}

#[test]
fn test_set_position_orientation_mapping() {
    let mut s = empty_state();
    s.set_position(0, 0, 1);
    assert_eq!(s.direction, 0);
    s.set_position(0, 0, 2);
    assert_eq!(s.direction, 1);
    s.set_position(0, 0, 3);
    assert_eq!(s.direction, 2);
    s.set_position(0, 0, 4);
    assert_eq!(s.direction, 3);
}

#[test]
fn test_direction_north_one_hot() {
    let mut s = empty_state();
    s.direction = 0;
    let v = s.to_state_vector();
    assert_eq!(v[2], 1.0);
    assert_eq!(v[3], 0.0);
    assert_eq!(v[4], 0.0);
    assert_eq!(v[5], 0.0);
}

#[test]
fn test_direction_west_one_hot() {
    let mut s = empty_state();
    s.direction = 3;
    let v = s.to_state_vector();
    assert_eq!(v[2], 0.0);
    assert_eq!(v[3], 0.0);
    assert_eq!(v[4], 0.0);
    assert_eq!(v[5], 1.0);
}

#[test]
fn test_level_1_encoding() {
    let mut s = empty_state();
    s.level = 1;
    let v = s.to_state_vector();
    assert!((v[6] - 1.0 / MAX_LEVEL as f32).abs() < 1e-6);
}

#[test]
fn test_level_8_encoding() {
    let mut s = empty_state();
    s.level = 8;
    let v = s.to_state_vector();
    assert!((v[6] - 1.0).abs() < 1e-6);
}
 
#[test]
fn test_survival_full() {
    let mut s = empty_state();
    s.survival_ticks = MAX_SURVIVAL_TICKS;
    let v = s.to_state_vector();
    assert!((v[7] - 1.0).abs() < 1e-6);
}

#[test]
fn test_survival_zero() {
    let mut s = empty_state();
    s.survival_ticks = 0;
    let v = s.to_state_vector();
    assert_eq!(v[7], 0.0);
}

#[test]
fn test_survival_capped_at_1260() {
    let mut s = empty_state();
    s.survival_ticks = MAX_SURVIVAL_TICKS * 2;
    let v = s.to_state_vector();
    assert!((v[7] - 1.0).abs() < 1e-6);
}

#[test]
fn test_inventory_all_zero() {
    let mut s = empty_state();
    s.inventory = [0; 7];
    let v = s.to_state_vector();
    for i in 8..15 {
        assert_eq!(v[i], 0.0, "inventory[{}] should be 0", i - 8);
    }
}

#[test]
fn test_inventory_normalised() {
    let mut s = empty_state();
    s.inventory = [INV_NORM_CAP; 7];
    let v = s.to_state_vector();
    for i in 8..15 {
        assert!((v[i] - 1.0).abs() < 1e-6, "inventory[{}] should be 1.0 (normalised at INV_NORM_CAP)", i - 8);
    }
}

#[test]
fn test_inventory_capped_at_20() {
    let mut s = empty_state();
    s.inventory = [INV_NORM_CAP * 5, 0, 0, 0, 0, 0, 0];
    let v = s.to_state_vector();
    assert!((v[8] - 1.0).abs() < 1e-6);
}

#[test]
fn test_no_message_sound_all_zero() {
    let mut s = empty_state();
    s.last_message = None;
    let v = s.to_state_vector();
    for i in 15..25 {
        assert_eq!(v[i], 0.0, "sound[{}] should be 0", i - 15);
    }
}
 
#[test]
fn test_message_direction_0_encodes_correctly() {
    let mut s = empty_state();
    s.last_message = Some(0);
    let v = s.to_state_vector();
    assert_eq!(v[15], 1.0, "has_message flag should be 1");
    assert_eq!(v[16], 1.0, "direction 0 bit should be 1");
    for i in 17..25 {
        assert_eq!(v[i], 0.0, "other direction bits should be 0");
    }
}

#[test]
fn test_message_direction_8_encodes_correctly() {
    let mut s = empty_state();
    s.last_message = Some(8);
    let v = s.to_state_vector();
    assert_eq!(v[15], 1.0);
    assert_eq!(v[15 + N_SOUND_DIRECTIONS], 1.0, "direction 8 bit should be last sound index");
}

#[test]
fn test_message_is_consumed_after_state_vector() {
    let mut s = empty_state();
    s.last_message = Some(3);
    s.to_state_vector();
    assert!(s.last_message.is_none(), "last_message should be consumed after to_state_vector()");
}

#[test]
fn test_vision_all_zeros_when_no_look_tiles() {
    let mut s = empty_state();
    s.look_tiles = vec![];
    let v = s.to_state_vector();
    for i in 25..STATE_DIM {
        assert_eq!(v[i], 0.0, "vision[{}] should be 0 with no tiles", i - 25);
    }
}

#[test]
fn test_vision_first_tile_food() {
    let mut s = empty_state();
    let mut tile = TileView::default();
    tile.food = VIS_NORM_CAP / 2;
    s.look_tiles = vec![tile];
    let v = s.to_state_vector();
    assert!((v[25] - 0.5).abs() < 1e-6, "food=5 should normalise to 0.5, got {}", v[25]);
}
 
#[test]
fn test_vision_resource_order_in_tile() {
    let mut s = empty_state();
    let mut tile = TileView::default();
    tile.food      = 10;
    tile.linemate  = 5;
    tile.deraumere = 0;
    s.look_tiles = vec![tile];
    let v = s.to_state_vector();
    assert!((v[25] - 1.0).abs() < 1e-6, "food");
    assert!((v[26] - 0.5).abs() < 1e-6, "linemate");
    assert!((v[27] - 0.0).abs() < 1e-6, "deraumere");
}

#[test]
fn test_vision_resource_capped_at_10() {
    let mut s = empty_state();
    let mut tile = TileView::default();
    tile.food = VIS_NORM_CAP * 100;
    s.look_tiles = vec![tile];
    let v = s.to_state_vector();
    assert!((v[25] - 1.0).abs() < 1e-6, "food should clamp to 1.0 at VIS_NORM_CAP");
}

#[test]
fn test_mask_length_is_22() {
    let s = empty_state();
    assert_eq!(s.valid_mask().len(), N_ACTIONS);
    assert_eq!(N_ACTIONS, 22);
}

#[test]
fn test_mask_no_tiles_blocks_take_and_eject() {
    let mut s = empty_state();
    s.look_tiles = vec![];
    let mask = s.valid_mask();
    for i in 5..=11 {
        assert!(!mask[i], "action {i} (Take) should be blocked with no look data");
    }
    assert!(!mask[1], "Eject should be blocked when no other player visible");
}

#[test]
fn test_mask_take_food_allowed_when_food_on_tile() {
    let mut s = empty_state();
    s.look_tiles = vec![tile_with(3, 0)];
    let mask = s.valid_mask();
    assert!(mask[5], "Take food should be allowed");
    assert!(!mask[6], "Take linemate should be blocked (none on tile)");
}

#[test]
fn test_mask_take_linemate_allowed_when_linemate_on_tile() {
    let mut s = empty_state();
    s.look_tiles = vec![tile_with(0, 2)];
    let mask = s.valid_mask();
    assert!(mask[6], "Take linemate should be allowed");
    assert!(!mask[5], "Take food should be blocked");
}

#[test]
fn test_mask_drop_blocked_when_inventory_empty() {
    let s = empty_state();
    let mask = s.valid_mask();
    for i in 12..=18 {
        assert!(!mask[i], "Set action {i} should be blocked with empty inventory");
    }
}

#[test]
fn test_mask_drop_food_allowed_when_have_food() {
    let mut s = empty_state();
    s.inventory[0] = 5;
    let mask = s.valid_mask();
    assert!(mask[12], "Set food should be allowed");
    assert!(!mask[13], "Set linemate should be blocked");
}

#[test]
fn test_mask_eject_requires_other_player_on_tile() {
    let mut s = empty_state();
    let mut tile = TileView::default();
    tile.players = 1;
    s.look_tiles = vec![tile];
    let mask = s.valid_mask();
    assert!(!mask[1], "Eject blocked when alone on tile");
 
    s.look_tiles[0].players = 2;
    let mask = s.valid_mask();
    assert!(mask[1], "Eject allowed when another player present");
}

#[test]
fn test_mask_eat_requires_food_in_inventory() {
    let mut s = empty_state();
    s.inventory[0] = 0;
    assert!(!s.valid_mask()[20], "Eat blocked with no food");
 
    s.inventory[0] = 1;
    assert!(s.valid_mask()[20], "Eat allowed with food");
}

#[test]
fn test_mask_incantation_always_allowed() {
    let s = empty_state();
    assert!(s.valid_mask()[0], "Incantation should always be in mask");
}

#[test]
fn test_mask_fork_always_allowed() {
    let s = empty_state();
    assert!(s.valid_mask()[21], "Fork should always be in mask");
}

#[test]
fn test_mask_movement_always_allowed() {
    let s = empty_state();
    let mask = s.valid_mask();
    assert!(mask[2], "Forward always allowed");
    assert!(mask[3], "Left always allowed");
    assert!(mask[4], "Right always allowed");
}
