use crate::client::{Client, ClientState};
use crate::command::QueuedCommand;
use crate::config::Config;
use crate::constants::MAX_PENDING_COMMANDS;
use crate::constants::{
    CARRIAGE_RETURN, EMPTY_LINE, GRAPHIC_TEAM_NAME, KO_RESPONSE, LINE_DELIMITER,
    PLAYER_ID_INCREMENT, RESPONSE_END, RESPONSE_SEPARATOR,
};
use crate::egg::{count_team_eggs, take_random_team_egg, Egg};
use crate::player::{Orientation, Player};
use crate::team::Team;
use std::io::Write;

pub fn handle_complete_client_lines(
    client: &mut Client,
    config: &Config,
    teams: &mut [Team],
    players: &mut Vec<Player>,
    next_player_id: &mut usize,
    eggs: &mut Vec<Egg>,
) {
    while let Some(line_end_index) = client.input_buffer.find(LINE_DELIMITER) {
        let line = extract_client_line(client, line_end_index);

        if line == EMPTY_LINE {
            continue;
        }

        handle_client_line(client, &line, config, teams, players, next_player_id, eggs);
    }
}

fn extract_client_line(client: &mut Client, line_end_index: usize) -> String {
    let mut line = client.input_buffer[..line_end_index].to_string();

    client.input_buffer.drain(..=line_end_index);

    if line.ends_with(CARRIAGE_RETURN) {
        line.pop();
    }

    line
}

fn handle_client_line(
    client: &mut Client,
    line: &str,
    config: &Config,
    teams: &mut [Team],
    players: &mut Vec<Player>,
    next_player_id: &mut usize,
    eggs: &mut Vec<Egg>,
) {
    match client.state {
        ClientState::WaitingTeamName => {
            handle_handshake_line(client, line, config, teams, players, next_player_id, eggs);
        }
        ClientState::Ai => {
            queue_ai_command(client, line);
        }
        ClientState::Gui => {
            println!("GUI command: {}", line);
        }
    }
}

fn handle_handshake_line(
    client: &mut Client,
    line: &str,
    config: &Config,
    teams: &mut [Team],
    players: &mut Vec<Player>,
    next_player_id: &mut usize,
    eggs: &mut Vec<Egg>,
) {
    if line == GRAPHIC_TEAM_NAME {
        authenticate_gui_client(client);
        return;
    }

    if let Some(team) = find_team_mut(teams, line) {
        authenticate_ai_client(client, team, config, players, next_player_id, eggs);
        return;
    }

    reject_connection(client, line);
}

fn authenticate_gui_client(client: &mut Client) {
    client.state = ClientState::Gui;
    client.team_name = None;
    client.player_id = None;

    println!("GUI client authenticated");
}

fn find_team_mut<'a>(teams: &'a mut [Team], team_name: &str) -> Option<&'a mut Team> {
    teams.iter_mut().find(|team| team.name == team_name)
}

fn authenticate_ai_client(
    client: &mut Client,
    team: &mut Team,
    config: &Config,
    players: &mut Vec<Player>,
    next_player_id: &mut usize,
    eggs: &mut Vec<Egg>,
) {
    if !team.reserve_slot() {
        reject_connection(client, &team.name);
        return;
    }

    let Some(egg) = take_random_team_egg(eggs, &team.name) else {
        team.release_slot();
        reject_connection(client, &team.name);
        return;
    };

    let player_id = *next_player_id;
    *next_player_id += PLAYER_ID_INCREMENT;

    let player = Player::new(
        player_id,
        team.name.clone(),
        egg.x,
        egg.y,
        Orientation::random(),
    );

    println!("Egg {} hatched for team {}", egg.id, team.name);

    println!(
        "Player {} spawned at ({}, {}) facing {:?}",
        player.id, player.x, player.y, player.orientation
    );

    players.push(player);

    client.state = ClientState::Ai;
    client.team_name = Some(team.name.clone());
    client.player_id = Some(player_id);

    let available_eggs = count_team_eggs(eggs, &team.name);

    let response = format!(
        "{}{}{}{}{}{}",
        available_eggs, RESPONSE_END, config.width, RESPONSE_SEPARATOR, config.height, RESPONSE_END
    );

    if let Err(error) = client.socket.write_all(response.as_bytes()) {
        eprintln!("Failed to send AI handshake response: {}", error);
    }

    println!(
        "AI client authenticated for team {} with player {}",
        team.name, player_id
    );
}

fn reject_connection(client: &mut Client, team_name: &str) {
    eprintln!("Unknown team or no available egg: {}", team_name);

    if let Err(error) = client.socket.write_all(KO_RESPONSE) {
        eprintln!("Failed to send rejection response: {}", error);
    }
}

fn queue_ai_command(client: &mut Client, line: &str) {
    let command = QueuedCommand::new(line.to_string());

    if !client.command_queue.push(command, MAX_PENDING_COMMANDS) {
        println!("Command queue full for player {:?}", client.player_id);
        return;
    }

    println!(
        "Queued command for player {:?}: {} ({}/{})",
        client.player_id,
        line,
        client.command_queue.len(),
        MAX_PENDING_COMMANDS
    );
}
