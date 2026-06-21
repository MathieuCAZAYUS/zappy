use crate::client::{Client, ClientState};
use crate::command::CommandQueue;
use crate::config::Config;
use crate::constants::{DEFAULT_BIND_ADDRESS, READ_BUFFER_SIZE, TOKEN_INCREMENT, WELCOME_MESSAGE};
use crate::egg::Egg;
use crate::player::Player;
use crate::protocol::handle_complete_client_lines;
use crate::team::Team;
use mio::net::TcpListener;
use mio::{Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;

pub fn create_listener(port: u16) -> io::Result<TcpListener> {
    let address = create_socket_address(port)?;
    TcpListener::bind(address)
}

fn create_socket_address(port: u16) -> io::Result<SocketAddr> {
    format!("{}:{}", DEFAULT_BIND_ADDRESS, port)
        .parse::<SocketAddr>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}

pub fn accept_new_clients(
    listener: &mut TcpListener,
    poll: &mut Poll,
    clients: &mut HashMap<Token, Client>,
    next_token_id: &mut usize,
) {
    loop {
        match listener.accept() {
            Ok((mut socket, address)) => {
                println!("New client connected: {}", address);

                let token = Token(*next_token_id);
                *next_token_id += TOKEN_INCREMENT;

                if let Err(error) = poll
                    .registry()
                    .register(&mut socket, token, Interest::READABLE)
                {
                    eprintln!("Failed to register client: {}", error);
                    continue;
                }

                if let Err(error) = socket.write_all(WELCOME_MESSAGE) {
                    eprintln!("Failed to send welcome message: {}", error);
                    continue;
                }

                clients.insert(
                    token,
                    Client {
                        socket,
                        input_buffer: String::new(),
                        state: ClientState::WaitingTeamName,
                        team_name: None,
                        player_id: None,
                        command_queue: CommandQueue::new(),
                    },
                );
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                break;
            }
            Err(error) => {
                eprintln!("Accept error: {}", error);
                break;
            }
        }
    }
}

pub fn read_from_client(
    token: Token,
    clients: &mut HashMap<Token, Client>,
    config: &Config,
    teams: &mut [Team],
    players: &mut Vec<Player>,
    next_player_id: &mut usize,
    eggs: &mut Vec<Egg>,
) {
    let mut should_disconnect = false;

    if let Some(client) = clients.get_mut(&token) {
        let mut buffer = [0; READ_BUFFER_SIZE];

        match client.socket.read(&mut buffer) {
            Ok(size) if size == 0 => {
                println!("Client disconnected");
                should_disconnect = true;
            }
            Ok(size) => {
                append_to_client_buffer(client, &buffer, size);

                handle_complete_client_lines(client, config, teams, players, next_player_id, eggs);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
            Err(error) => {
                eprintln!("Read error: {}", error);
                should_disconnect = true;
            }
        }
    }

    if should_disconnect {
        clients.remove(&token);
    }
}

fn append_to_client_buffer(client: &mut Client, buffer: &[u8], size: usize) {
    let received_text = String::from_utf8_lossy(&buffer[..size]);

    client.input_buffer.push_str(&received_text);
}
