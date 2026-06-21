use crate::constants::{
    INITIAL_PLAYER_LEVEL, ORIENTATION_COUNT, ORIENTATION_EAST_INDEX, ORIENTATION_NORTH_INDEX,
    ORIENTATION_SOUTH_INDEX, ORIENTATION_WEST_INDEX,
};
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    pub fn random() -> Self {
        let mut random_generator = rand::thread_rng();

        let orientation_index = random_generator.gen_range(0..ORIENTATION_COUNT);

        match orientation_index {
            ORIENTATION_NORTH_INDEX => Self::North,
            ORIENTATION_EAST_INDEX => Self::East,
            ORIENTATION_SOUTH_INDEX => Self::South,
            ORIENTATION_WEST_INDEX => Self::West,
            _ => unreachable!("generated orientation index must be valid"),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    pub id: usize,
    pub team_name: String,
    pub x: usize,
    pub y: usize,
    pub orientation: Orientation,
    pub level: usize,
}

impl Player {
    pub fn new(id: usize, team_name: String, x: usize, y: usize, orientation: Orientation) -> Self {
        Self {
            id,
            team_name,
            x,
            y,
            orientation,
            level: INITIAL_PLAYER_LEVEL,
        }
    }
}
