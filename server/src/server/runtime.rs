use super::client::ClientId;
use super::network::{self, PollEvent, PollTarget};
use super::{protocol, scheduler};
use crate::config::{parse_args, Config, ParseOutcome};
use crate::constants::{DEFAULT_BIND_ADDRESS, USAGE};
use crate::game::GameState;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::time::Instant;

pub fn run() -> Result<(), String> {
    let Some(config) = load_config()? else {
        return Ok(());
    };

    let listener = create_listener(&config)?;
    let game = GameState::new(&config);

    run_event_loop(listener, game)
}

fn load_config() -> Result<Option<Config>, String> {
    match parse_args()? {
        ParseOutcome::Help => {
            println!("{USAGE}");
            Ok(None)
        }
        ParseOutcome::Config(config) => Ok(Some(config)),
    }
}

fn create_listener(config: &Config) -> Result<TcpListener, String> {
    let ip_address = DEFAULT_BIND_ADDRESS
        .parse::<IpAddr>()
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    let address = SocketAddr::new(ip_address, config.port);
    let listener = TcpListener::bind(address)
        .map_err(|error| format!("failed to bind server socket: {error}"))?;

    listener
        .set_nonblocking(true)
        .map_err(|error| format!("failed to configure server socket: {error}"))?;

    Ok(listener)
}

fn run_event_loop(listener: TcpListener, mut game: GameState) -> Result<(), String> {
    loop {
        prepare_iteration(&mut game);

        let events = wait_for_events(&listener, &game)?;
        let disconnected = process_events(&listener, &mut game, events)?;

        finish_iteration(&mut game, disconnected);
    }
}

fn prepare_iteration(game: &mut GameState) {
    scheduler::process_timers(game, Instant::now());

    let mut disconnected = network::flush_all_clients(game);

    collect_closed_clients(game, &mut disconnected);
    remove_clients(game, disconnected);
}

fn wait_for_events(listener: &TcpListener, game: &GameState) -> Result<Vec<PollEvent>, String> {
    let timeout = scheduler::next_timeout(game, Instant::now());

    match network::poll_events(listener, game, timeout) {
        Ok(events) => Ok(events),
        Err(error) if error.kind() == ErrorKind::Interrupted => Ok(Vec::new()),
        Err(error) => Err(format!("poll failed: {error}")),
    }
}

fn process_events(
    listener: &TcpListener,
    game: &mut GameState,
    events: Vec<PollEvent>,
) -> Result<Vec<ClientId>, String> {
    let mut disconnected = Vec::new();

    for event in events {
        match event.target {
            PollTarget::Listener => {
                process_listener_event(listener, game, event)?;
            }
            PollTarget::Client(client_id) => {
                process_client_event(game, client_id, event, &mut disconnected);
            }
        }
    }

    Ok(disconnected)
}

fn process_listener_event(
    listener: &TcpListener,
    game: &mut GameState,
    event: PollEvent,
) -> Result<(), String> {
    if !event.readable {
        return Ok(());
    }

    network::accept_connections(listener, game)
        .map_err(|error| format!("failed to accept client: {error}"))
}

fn process_client_event(
    game: &mut GameState,
    client_id: ClientId,
    event: PollEvent,
    disconnected: &mut Vec<ClientId>,
) {
    if event.readable {
        process_client_input(game, client_id, disconnected);
    }

    if event.writable && network::flush_client(game, client_id).is_err() {
        disconnected.push(client_id);
    }

    if event.closed {
        disconnected.push(client_id);
    }
}

fn process_client_input(
    game: &mut GameState,
    client_id: ClientId,
    disconnected: &mut Vec<ClientId>,
) {
    match network::read_client(game, client_id) {
        Ok(peer_closed) => {
            protocol::handle_complete_client_lines(game, client_id);

            if peer_closed {
                disconnected.push(client_id);
            }
        }
        Err(_) => disconnected.push(client_id),
    }
}

fn finish_iteration(game: &mut GameState, mut disconnected: Vec<ClientId>) {
    scheduler::process_timers(game, Instant::now());
    disconnected.extend(network::flush_all_clients(game));

    collect_closed_clients(game, &mut disconnected);
    remove_clients(game, disconnected);
}

fn collect_closed_clients(game: &GameState, disconnected: &mut Vec<ClientId>) {
    disconnected.extend(game.clients.iter().filter_map(|(client_id, client)| {
        (client.close_after_flush && !client.has_pending_output()).then_some(*client_id)
    }));
}

fn remove_clients(game: &mut GameState, client_ids: Vec<ClientId>) {
    let unique_client_ids = client_ids.into_iter().collect::<HashSet<_>>();

    for client_id in unique_client_ids {
        network::remove_client(game, client_id);
    }
}
