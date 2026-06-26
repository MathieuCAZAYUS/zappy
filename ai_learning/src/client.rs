use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use crate::agent::NeuralAgent;
use crate::protocol::{action_to_command, parse_response, ServerResponse};
use crate::state::GameState;

const CMD_BUFFER_LIMIT: usize = 10;

pub struct ZappyClient {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    team: String,
    pending: usize,
    state: Option<GameState>,
    response_queue: Vec<ServerResponse>,
}

impl ZappyClient {
    pub fn connect(host: &str, port: u16, team: &str) -> anyhow::Result<Self> {
        let addr = format!("{host}:{port}");
        let stream = TcpStream::connect(&addr)
            .map_err(|e| anyhow::anyhow!("Cannot connect to {addr}: {e}"))?;
        let writer = stream.try_clone()?;
        let mut reader = BufReader::new(stream);

        let mut line = String::new();

        reader.read_line(&mut line)?;
        if line.trim() != "WELCOME" {
            anyhow::bail!("Expected WELCOME, got {:?}", line.trim());
        }
        eprintln!("[AI] Connected. Sending team: {team}");

        {
            let mut w = writer.try_clone()?;
            w.write_all(format!("{team}\n").as_bytes())?;
            w.flush()?;
        }

        line.clear();
        reader.read_line(&mut line)?;
        let client_num: u32 = line.trim().parse()
            .map_err(|_| anyhow::anyhow!("Expected CLIENT-NUM, got {:?}", line.trim()))?;
        eprintln!("[AI] Free slots: {client_num}");

        line.clear();
        reader.read_line(&mut line)?;
        let dims: Vec<u32> = line.trim().split(' ')
            .filter_map(|t| t.parse().ok())
            .collect();
        if dims.len() < 2 {
            anyhow::bail!("Expected X Y, got {:?}", line.trim());
        }
        let (map_w, map_h) = (dims[0], dims[1]);
        eprintln!("[AI] Map size: {map_w}x{map_h}");

        Ok(Self {
            reader,
            writer,
            team: team.to_string(),
            pending: 0,
            state: Some(GameState::new(map_w, map_h)),
            response_queue: Vec::new(),
        })
    }

    fn send(&mut self, cmd: &str) -> anyhow::Result<()> {
        if self.pending >= CMD_BUFFER_LIMIT {
            self.recv_one()?;
        }
        let msg = format!("{cmd}\n");
        self.writer.write_all(msg.as_bytes())?;
        self.writer.flush()?;
        self.pending += 1;
        eprintln!("[AI →] {cmd}");
        Ok(())
    }

    fn recv_one(&mut self) -> anyhow::Result<ServerResponse> {
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line)?;
            if n == 0 {
                anyhow::bail!("Server closed connection");
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            eprintln!("[AI ←] {trimmed}");

            if let Some(resp) = parse_response(trimmed) {
                if self.pending > 0 {
                    self.pending -= 1;
                }
                return Ok(resp);
            }
            eprintln!("[AI] Unknown server line: {trimmed:?}");
        }
    }

    fn drain_pending(&mut self) -> anyhow::Result<()> {
        while self.pending > 0 {
            let resp = self.recv_one()?;
            self.handle_unsolicited(resp);
        }
        Ok(())
    }

    fn handle_unsolicited(&mut self, resp: ServerResponse) {
        match resp {
            ServerResponse::Message { direction, .. } => {
                if let Some(s) = &mut self.state {
                    s.last_message = Some(direction);
                }
            }
            ServerResponse::Eject { direction } => {
                eprintln!("[AI] Ejected from direction {direction}");
            }
            ServerResponse::ElevationUnderway => {
                eprintln!("[AI] Elevation underway...");
            }
            ServerResponse::CurrentLevel(l) => {
                if let Some(s) = &mut self.state {
                    s.level = l;
                    eprintln!("[AI] Now level {l}");
                }
            }
            _ => {}
        }
    }

    fn refresh_state(&mut self) -> anyhow::Result<()> {
        self.send("Look")?;
        self.send("Inventory")?;

        let mut got_look = false;
        let mut got_inv = false;

        while !got_look || !got_inv {
            let resp = self.recv_one()?;
            let state = self.state.as_mut().unwrap();
            match resp {
                ServerResponse::Look(tiles) => {
                    state.look_tiles = tiles;
                    got_look = true;
                }
                ServerResponse::Inventory { food, linemate, deraumere, sibur, mendiane, phiras, thystame } => {
                    state.inventory = [food, linemate, deraumere, sibur, mendiane, phiras, thystame];
                    state.survival_ticks = food * 126;
                    got_inv = true;
                }
                other => self.handle_unsolicited(other),
            }
        }
        Ok(())
    }

    pub fn run(&mut self, agent: &NeuralAgent) -> anyhow::Result<()> {
        eprintln!("[AI] Starting main loop");

        loop {
            self.refresh_state()?;

            let action = {
                let state = self.state.as_mut().unwrap();
                let sv = state.to_state_vector();
                let mask = state.valid_mask();
                agent.act(&sv, &mask)
            };
            let cmd = action_to_command(action);
            eprintln!("[AI] Action {action} → {cmd}");

            self.send(cmd)?;
            let resp = self.recv_one()?;

            match resp {
                ServerResponse::Dead => {
                    eprintln!("[AI] Player died. Exiting.");
                    return Ok(());
                }
                ServerResponse::Ok => {}
                ServerResponse::Ko => {
                    eprintln!("[AI] Command {cmd} returned ko — ignored");
                }
                ServerResponse::ElevationUnderway => {
                    // Incantation started; wait for the result line.
                    eprintln!("[AI] Incantation in progress...");
                    loop {
                        let r = self.recv_one()?;
                        match r {
                            ServerResponse::CurrentLevel(l) => {
                                self.state.as_mut().unwrap().level = l;
                                eprintln!("[AI] Elevated to level {l}");
                                break;
                            }
                            ServerResponse::Ko => {
                                eprintln!("[AI] Incantation failed");
                                break;
                            }
                            ServerResponse::Dead => {
                                eprintln!("[AI] Died during incantation");
                                return Ok(());
                            }
                            other => self.handle_unsolicited(other),
                        }
                    }
                }
                ServerResponse::ConnectNbr(_) => {}
                other => self.handle_unsolicited(other),
            }
        }
    }
}
