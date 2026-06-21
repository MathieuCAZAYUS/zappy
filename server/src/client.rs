use crate::command::CommandQueue;
use mio::net::TcpStream;

#[derive(Debug, PartialEq)]
pub enum ClientState {
    WaitingTeamName,
    Ai,
    Gui,
}

pub struct Client {
    pub socket: TcpStream,
    pub input_buffer: String,
    pub state: ClientState,
    pub team_name: Option<String>,
    pub player_id: Option<usize>,
    pub command_queue: CommandQueue,
}
