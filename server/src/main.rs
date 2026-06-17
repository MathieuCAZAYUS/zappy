use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::env;
use std::io::{self, Read, Write};
use std::net::SocketAddr;

const SERVER_TOKEN_ID: usize = 0;
const SERVER_TOKEN: Token = Token(SERVER_TOKEN_ID);

const FIRST_CLIENT_TOKEN_ID: usize = 1;
const TOKEN_INCREMENT: usize = 1;

const EVENTS_CAPACITY: usize = 128;
const READ_BUFFER_SIZE: usize = 512;

const ERROR_EXIT: i32 = 84;

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0";
const WELCOME_MESSAGE: &[u8] = b"WELCOME\n";

const HELP_FLAG: &str = "--help";

const PORT_FLAG: &str = "-p";
const WIDTH_FLAG: &str = "-x";
const HEIGHT_FLAG: &str = "-y";
const TEAM_NAMES_FLAG: &str = "-n";
const CLIENTS_NB_FLAG: &str = "-c";
const FREQUENCY_FLAG: &str = "-f";

const USAGE: &str =
    "USAGE: ./zappy_server -p port -x width -y height -n name1 name2 ... -c clientsNb -f freq";

const MIN_PORT: u16 = 1;
const MIN_WIDTH: usize = 1;
const MIN_HEIGHT: usize = 1;
const MIN_CLIENTS_NB: usize = 1;
const MIN_FREQUENCY: usize = 1;

const LINE_DELIMITER: char = '\n';
const CARRIAGE_RETURN: char = '\r';
const EMPTY_LINE: &str = "";

#[derive(Debug)]
struct Config {
    port: u16,
    width: usize,
    height: usize,
    teams: Vec<String>,
    clients_nb: usize,
    freq: usize,
}

struct Client {
    socket: TcpStream,
    input_buffer: String,
}

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

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(EVENTS_CAPACITY);

    let mut listener = create_listener(config.port)?;

    poll.registry()
        .register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;

    let mut clients: HashMap<Token, Client> = HashMap::new();
    let mut next_token_id = FIRST_CLIENT_TOKEN_ID;

    println!("Server listening on port {}", config.port);

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER_TOKEN => {
                    accept_new_clients(&mut listener, &mut poll, &mut clients, &mut next_token_id);
                }
                client_token => {
                    read_from_client(client_token, &mut clients);
                }
            }
        }
    }
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|arg| arg == HELP_FLAG) {
        println!("{}", USAGE);
        std::process::exit(0);
    }

    let mut port = None;
    let mut width = None;
    let mut height = None;
    let mut teams = Vec::new();
    let mut clients_nb = None;
    let mut freq = None;

    let mut args_iter = args.iter().skip(1).peekable();

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            PORT_FLAG => {
                port = Some(parse_next_value::<u16>(&mut args_iter, PORT_FLAG)?);
            }
            WIDTH_FLAG => {
                width = Some(parse_next_value::<usize>(&mut args_iter, WIDTH_FLAG)?);
            }
            HEIGHT_FLAG => {
                height = Some(parse_next_value::<usize>(&mut args_iter, HEIGHT_FLAG)?);
            }
            CLIENTS_NB_FLAG => {
                clients_nb = Some(parse_next_value::<usize>(&mut args_iter, CLIENTS_NB_FLAG)?);
            }
            FREQUENCY_FLAG => {
                freq = Some(parse_next_value::<usize>(&mut args_iter, FREQUENCY_FLAG)?);
            }
            TEAM_NAMES_FLAG => {
                parse_team_names(&mut args_iter, &mut teams);
            }
            _ => {
                return Err(format!("Unknown argument: {}", arg));
            }
        }
    }

    let config = Config {
        port: port.ok_or("Missing -p port")?,
        width: width.ok_or("Missing -x width")?,
        height: height.ok_or("Missing -y height")?,
        teams,
        clients_nb: clients_nb.ok_or("Missing -c clientsNb")?,
        freq: freq.ok_or("Missing -f freq")?,
    };

    validate_config(&config)?;
    Ok(config)
}

fn parse_next_value<'a, T>(
    args_iter: &mut std::iter::Peekable<std::iter::Skip<std::slice::Iter<'a, String>>>,
    flag: &str,
) -> Result<T, String>
where
    T: std::str::FromStr,
{
    let value = args_iter
        .next()
        .ok_or_else(|| format!("Missing value for {}", flag))?;

    value
        .parse::<T>()
        .map_err(|_| format!("Invalid value for {}", flag))
}

fn parse_team_names<'a>(
    args_iter: &mut std::iter::Peekable<std::iter::Skip<std::slice::Iter<'a, String>>>,
    teams: &mut Vec<String>,
) {
    while let Some(next_arg) = args_iter.peek() {
        if next_arg.starts_with('-') {
            break;
        }

        if let Some(team_name) = args_iter.next() {
            teams.push(team_name.clone());
        }
    }
}

fn validate_config(config: &Config) -> Result<(), String> {
    if config.port < MIN_PORT {
        return Err("Port must be greater than 0".to_string());
    }

    if config.width < MIN_WIDTH {
        return Err("Width must be greater than 0".to_string());
    }

    if config.height < MIN_HEIGHT {
        return Err("Height must be greater than 0".to_string());
    }

    if config.teams.is_empty() {
        return Err("Missing teams after -n".to_string());
    }

    if config.clients_nb < MIN_CLIENTS_NB {
        return Err("clientsNb must be greater than 0".to_string());
    }

    if config.freq < MIN_FREQUENCY {
        return Err("freq must be greater than 0".to_string());
    }

    Ok(())
}

fn create_listener(port: u16) -> io::Result<TcpListener> {
    let address = create_socket_address(port)?;
    TcpListener::bind(address)
}

fn create_socket_address(port: u16) -> io::Result<SocketAddr> {
    format!("{}:{}", DEFAULT_BIND_ADDRESS, port)
        .parse::<SocketAddr>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}

fn accept_new_clients(
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

fn read_from_client(token: Token, clients: &mut HashMap<Token, Client>) {
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
                handle_complete_client_lines(client);
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

fn handle_complete_client_lines(client: &mut Client) {
    while let Some(line_end_index) = client.input_buffer.find(LINE_DELIMITER) {
        let line = extract_client_line(client, line_end_index);

        if line != EMPTY_LINE {
            println!("Received line: {}", line);
        }
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
