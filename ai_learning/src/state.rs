use crate::protocol::TileView;

pub const MAX_VISION_TILES: usize = 64;
pub const N_RESOURCES: usize = 7;
pub const N_ACTIONS: usize = 22;

pub const STATE_DIM: usize = 2 + 4 + 1 + 1 + 7 + 10 + MAX_VISION_TILES * N_RESOURCES;

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
            survival_ticks: 1260,
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

        v.push(self.level as f32 / 8.0);

        v.push((self.survival_ticks.min(1260)) as f32 / 1260.0);

        for &qty in &self.inventory {
            v.push((qty.min(20)) as f32 / 20.0);
        }

        let mut sound = [0.0f32; 10];
        if let Some(dir) = self.last_message.take() {
            sound[0] = 1.0;
            if (dir as usize) < 9 {
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
                    v.push((r.min(10)) as f32 / 10.0);
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
