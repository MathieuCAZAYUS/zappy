use mio::Token;

pub const SERVER_TOKEN_ID: usize = 0;
pub const SERVER_TOKEN: Token = Token(SERVER_TOKEN_ID);

pub const FIRST_CLIENT_TOKEN_ID: usize = 1;
pub const TOKEN_INCREMENT: usize = 1;

pub const FIRST_PLAYER_ID: usize = 1;
pub const PLAYER_ID_INCREMENT: usize = 1;

pub const FIRST_EGG_ID: usize = 1;
pub const EGG_ID_INCREMENT: usize = 1;

pub const EVENTS_CAPACITY: usize = 128;
pub const READ_BUFFER_SIZE: usize = 512;

pub const ERROR_EXIT: i32 = 84;

pub const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0";
pub const WELCOME_MESSAGE: &[u8] = b"WELCOME\n";
pub const KO_RESPONSE: &[u8] = b"ko\n";

pub const HELP_FLAG: &str = "--help";

pub const PORT_FLAG: &str = "-p";
pub const WIDTH_FLAG: &str = "-x";
pub const HEIGHT_FLAG: &str = "-y";
pub const TEAM_NAMES_FLAG: &str = "-n";
pub const CLIENTS_NB_FLAG: &str = "-c";
pub const FREQUENCY_FLAG: &str = "-f";

pub const USAGE: &str =
    "USAGE: ./zappy_server -p port -x width -y height -n name1 name2 ... -c clientsNb -f freq";

pub const MIN_PORT: u16 = 1;
pub const MIN_WIDTH: usize = 1;
pub const MIN_HEIGHT: usize = 1;
pub const MIN_CLIENTS_NB: usize = 1;
pub const MIN_FREQUENCY: usize = 1;

pub const LINE_DELIMITER: char = '\n';
pub const CARRIAGE_RETURN: char = '\r';
pub const EMPTY_LINE: &str = "";

pub const GRAPHIC_TEAM_NAME: &str = "GRAPHIC";
pub const RESPONSE_SEPARATOR: &str = " ";
pub const RESPONSE_END: &str = "\n";

pub const INITIAL_PLAYER_LEVEL: usize = 1;

pub const ORIENTATION_COUNT: usize = 4;
pub const ORIENTATION_NORTH_INDEX: usize = 0;
pub const ORIENTATION_EAST_INDEX: usize = 1;
pub const ORIENTATION_SOUTH_INDEX: usize = 2;
pub const ORIENTATION_WEST_INDEX: usize = 3;

pub const EMPTY_RESOURCE_COUNT: usize = 0;
pub const RESOURCE_INCREMENT: usize = 1;

pub const RESOURCE_DENSITY_FOOD: f64 = 0.5;
pub const RESOURCE_DENSITY_LINEMATE: f64 = 0.3;
pub const RESOURCE_DENSITY_DERAUMERE: f64 = 0.15;
pub const RESOURCE_DENSITY_SIBUR: f64 = 0.1;
pub const RESOURCE_DENSITY_MENDIANE: f64 = 0.1;
pub const RESOURCE_DENSITY_PHIRAS: f64 = 0.08;
pub const RESOURCE_DENSITY_THYSTAME: f64 = 0.05;

pub const MINIMUM_RESOURCE_QUANTITY: usize = 1;

pub const MAX_PENDING_COMMANDS: usize = 10;
