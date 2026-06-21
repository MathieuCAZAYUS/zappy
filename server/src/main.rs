mod client;
mod command;
mod config;
mod constants;
mod egg;
mod map;
mod network;
mod player;
mod protocol;
mod resources;
mod team;

use crate::client::Client;
use crate::config::{parse_args, Config};
use crate::constants::{
    ERROR_EXIT, EVENTS_CAPACITY, FIRST_CLIENT_TOKEN_ID, FIRST_EGG_ID, FIRST_PLAYER_ID,
    SERVER_TOKEN, USAGE,
};
use crate::egg::{create_initial_eggs, Egg};
use crate::map::GameMap;
use crate::network::{accept_new_clients, create_listener, read_from_client};
use crate::player::Player;
use crate::resources::{print_resource_totals, spawn_initial_resources};
use crate::team::Team;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io;

fn main() -> io::Result<()> {
    let config = match parse_args() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Error: {}", error);
            eprintln!("{}", USAGE);
            std::process::exit(ERROR_EXIT);
        }
    };

    println!("Starting server with config: {:?}", config);

    let mut game_map = GameMap::new(config.width, config.height);

    spawn_initial_resources(&mut game_map);

    println!(
        "Created map: {}x{} with {} tiles",
        game_map.width,
        game_map.height,
        game_map.tile_count()
    );

    print_resource_totals(&game_map);

    let mut teams = create_teams(&config);

    let mut next_egg_id = FIRST_EGG_ID;

    let mut eggs: Vec<Egg> = create_initial_eggs(
        &teams,
        config.clients_nb,
        config.width,
        config.height,
        &mut next_egg_id,
    );

    println!("Created {} initial eggs", eggs.len());

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(EVENTS_CAPACITY);
    let mut listener = create_listener(config.port)?;

    poll.registry()
        .register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;

    let mut clients: HashMap<Token, Client> = HashMap::new();
    let mut next_token_id = FIRST_CLIENT_TOKEN_ID;

    let mut players: Vec<Player> = Vec::new();
    let mut next_player_id = FIRST_PLAYER_ID;

    println!("Server listening on port {}", config.port);

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER_TOKEN => {
                    accept_new_clients(&mut listener, &mut poll, &mut clients, &mut next_token_id);
                }
                client_token => {
                    read_from_client(
                        client_token,
                        &mut clients,
                        &config,
                        &mut teams,
                        &mut players,
                        &mut next_player_id,
                        &mut eggs,
                    );
                }
            }
        }
    }
}

fn create_teams(config: &Config) -> Vec<Team> {
    config
        .teams
        .iter()
        .map(|team_name| Team::new(team_name.clone(), config.clients_nb))
        .collect()
}
