use super::broadcast::broadcast_direction;
use crate::constants::{KO_RESPONSE, OK_RESPONSE};
use crate::entities::player::{Orientation, Player};
use crate::game::GameState;
use crate::gui;
use crate::server::client::ClientId;

struct EjectionContext {
    origin: (usize, usize),
    movement: (isize, isize),
    player_ids: Vec<usize>,
    egg_ids: Vec<usize>,
}

impl EjectionContext {
    fn has_targets(&self) -> bool {
        !self.player_ids.is_empty() || !self.egg_ids.is_empty()
    }
}

fn move_forward(player: &mut Player, width: usize, height: usize) {
    player.move_forward(width, height);
}

fn turn_right(player: &mut Player) {
    player.turn_right();
}

fn turn_left(player: &mut Player) {
    player.turn_left();
}

impl GameState {
    pub fn execute_forward(&mut self, client_id: ClientId, player_id: usize) {
        match self.players.get_mut(&player_id) {
            Some(player) => {
                move_forward(player, self.map.width, self.map.height);
            }
            None => {
                self.queue_to_client(client_id, KO_RESPONSE);
                return;
            }
        }

        self.queue_to_client(client_id, OK_RESPONSE);
        self.broadcast_player_position(player_id);
    }

    pub fn execute_right(&mut self, client_id: ClientId, player_id: usize) {
        match self.players.get_mut(&player_id) {
            Some(player) => turn_right(player),
            None => {
                self.queue_to_client(client_id, KO_RESPONSE);
                return;
            }
        }

        self.queue_to_client(client_id, OK_RESPONSE);
        self.broadcast_player_position(player_id);
    }

    pub fn execute_left(&mut self, client_id: ClientId, player_id: usize) {
        match self.players.get_mut(&player_id) {
            Some(player) => turn_left(player),
            None => {
                self.queue_to_client(client_id, KO_RESPONSE);
                return;
            }
        }

        self.queue_to_client(client_id, OK_RESPONSE);
        self.broadcast_player_position(player_id);
    }

    pub fn execute_eject(&mut self, client_id: ClientId, ejector_id: usize) {
        match self.prepare_ejection(ejector_id) {
            Some(context) if context.has_targets() => {
                self.complete_ejection(client_id, ejector_id, context);
            }
            _ => {
                self.queue_to_client(client_id, KO_RESPONSE);
            }
        }
    }

    fn prepare_ejection(&self, ejector_id: usize) -> Option<EjectionContext> {
        let ejector = self.players.get(&ejector_id)?;
        let origin = (ejector.x, ejector.y);
        let movement = ejector.orientation.forward_delta();

        Some(EjectionContext {
            origin,
            movement,
            player_ids: self.ejection_player_ids(ejector_id, origin),
            egg_ids: self.ejection_egg_ids(origin),
        })
    }

    fn ejection_player_ids(&self, ejector_id: usize, origin: (usize, usize)) -> Vec<usize> {
        self.players
            .values()
            .filter(|player| player.id != ejector_id && (player.x, player.y) == origin)
            .map(|player| player.id)
            .collect()
    }

    fn ejection_egg_ids(&self, origin: (usize, usize)) -> Vec<usize> {
        self.eggs
            .values()
            .filter(|egg| (egg.x, egg.y) == origin)
            .map(|egg| egg.id)
            .collect()
    }

    fn complete_ejection(
        &mut self,
        client_id: ClientId,
        ejector_id: usize,
        context: EjectionContext,
    ) {
        self.eject_players(&context);
        self.destroy_ejected_eggs(&context.egg_ids);

        self.broadcast_to_guis(&gui::pex(ejector_id));
        self.queue_to_client(client_id, OK_RESPONSE);
    }

    fn eject_players(&mut self, context: &EjectionContext) {
        for player_id in &context.player_ids {
            self.eject_player(*player_id, context);
        }
    }

    fn eject_player(&mut self, player_id: usize, context: &EjectionContext) {
        let Some((position, orientation)) = self.move_ejected_player(player_id, context.movement)
        else {
            return;
        };

        self.notify_ejected_player(player_id, position, orientation, context.origin);
        self.broadcast_player_position(player_id);
    }

    fn move_ejected_player(
        &mut self,
        player_id: usize,
        movement: (isize, isize),
    ) -> Option<((usize, usize), Orientation)> {
        let player = self.players.get_mut(&player_id)?;

        player.x = (player.x as isize + movement.0).rem_euclid(self.map.width as isize) as usize;
        player.y = (player.y as isize + movement.1).rem_euclid(self.map.height as isize) as usize;

        Some(((player.x, player.y), player.orientation))
    }

    fn notify_ejected_player(
        &mut self,
        player_id: usize,
        position: (usize, usize),
        orientation: Orientation,
        origin: (usize, usize),
    ) {
        let direction = broadcast_direction(
            position,
            orientation,
            origin,
            self.map.width,
            self.map.height,
        );

        let Some(client_id) = self.player_token(player_id) else {
            return;
        };

        self.queue_to_client(client_id, &format!("eject: {direction}\n"));
    }

    fn destroy_ejected_eggs(&mut self, egg_ids: &[usize]) {
        for egg_id in egg_ids {
            self.eggs.remove(egg_id);
            self.broadcast_to_guis(&gui::edi(*egg_id));
        }
    }

    fn broadcast_player_position(&mut self, player_id: usize) {
        let Some(message) = self.players.get(&player_id).map(gui::ppo) else {
            return;
        };

        self.broadcast_to_guis(&message);
    }
}
