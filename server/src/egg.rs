use crate::constants::EGG_ID_INCREMENT;
use crate::team::Team;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug)]
pub struct Egg {
    pub id: usize,
    pub team_name: String,
    pub x: usize,
    pub y: usize,
}

impl Egg {
    pub fn new(id: usize, team_name: String, x: usize, y: usize) -> Self {
        Self {
            id,
            team_name,
            x,
            y,
        }
    }
}

pub fn create_initial_eggs(
    teams: &[Team],
    clients_per_team: usize,
    map_width: usize,
    map_height: usize,
    next_egg_id: &mut usize,
) -> Vec<Egg> {
    let mut eggs = Vec::new();
    let mut random_generator = rand::thread_rng();

    for team in teams {
        for _ in 0..clients_per_team {
            let x = random_generator.gen_range(0..map_width);
            let y = random_generator.gen_range(0..map_height);

            eggs.push(Egg::new(*next_egg_id, team.name.clone(), x, y));

            *next_egg_id += EGG_ID_INCREMENT;
        }
    }

    eggs
}

pub fn take_random_team_egg(eggs: &mut Vec<Egg>, team_name: &str) -> Option<Egg> {
    let matching_indexes: Vec<usize> = eggs
        .iter()
        .enumerate()
        .filter_map(|(index, egg)| {
            if egg.team_name == team_name {
                Some(index)
            } else {
                None
            }
        })
        .collect();

    let mut random_generator = rand::thread_rng();

    let selected_index = matching_indexes.choose(&mut random_generator).copied()?;

    Some(eggs.remove(selected_index))
}

pub fn count_team_eggs(eggs: &[Egg], team_name: &str) -> usize {
    eggs.iter().filter(|egg| egg.team_name == team_name).count()
}
