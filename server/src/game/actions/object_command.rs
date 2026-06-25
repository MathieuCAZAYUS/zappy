use crate::constants::{ID_INCREMENT, KO_RESPONSE, OK_RESPONSE};
use crate::entities::egg::Egg;
use crate::entities::player::Player;
use crate::game::GameState;
use crate::gui;
use crate::server::client::ClientId;
use crate::world::map::{GameMap, Resource};

pub fn take_object(player: &mut Player, map: &mut GameMap, resource: Resource) -> bool {
    let Some(tile) = map.get_tile_mut(player.x, player.y) else {
        return false;
    };
    if !tile.remove_resource(resource) {
        return false;
    }
    player.inventory.add(resource);
    true
}

pub fn set_object(player: &mut Player, map: &mut GameMap, resource: Resource) -> bool {
    if !player.inventory.remove(resource) {
        return false;
    }
    let Some(tile) = map.get_tile_mut(player.x, player.y) else {
        player.inventory.add(resource);
        return false;
    };
    tile.add_resource(resource);
    true
}

impl GameState {
    pub fn finish_fork(&mut self, token: ClientId, player_id: usize) {
        let Some(player) = self.players.get(&player_id) else {
            self.queue_to_client(token, KO_RESPONSE);
            return;
        };
        let egg = Egg::new(
            self.next_egg_id,
            Some(player_id),
            player.team_name.clone(),
            player.x,
            player.y,
        );
        self.next_egg_id += ID_INCREMENT;
        let message = gui::enw(&egg);
        self.eggs.insert(egg.id, egg);
        self.queue_to_client(token, OK_RESPONSE);
        self.broadcast_to_guis(&message);
    }

    pub fn execute_take(&mut self, token: ClientId, player_id: usize, resource: Resource) {
        let success = {
            let (players, map) = (&mut self.players, &mut self.map);
            players
                .get_mut(&player_id)
                .is_some_and(|player| take_object(player, map, resource))
        };
        if !success {
            self.queue_to_client(token, KO_RESPONSE);
            return;
        }
        self.queue_to_client(token, OK_RESPONSE);
        self.broadcast_to_guis(&gui::pgt(player_id, resource));
        let message = self.players.get(&player_id).map(gui::pin);
        if let Some(message) = message {
            self.broadcast_to_guis(&message);
        }
        if let Some(player) = self.players.get(&player_id) {
            let tile_message = gui::bct(&self.map, player.x, player.y);
            self.broadcast_to_guis(&tile_message);
        }
    }

    pub fn execute_set(&mut self, token: ClientId, player_id: usize, resource: Resource) {
        let success = {
            let (players, map) = (&mut self.players, &mut self.map);
            players
                .get_mut(&player_id)
                .is_some_and(|player| set_object(player, map, resource))
        };
        if !success {
            self.queue_to_client(token, KO_RESPONSE);
            return;
        }
        self.queue_to_client(token, OK_RESPONSE);
        self.broadcast_to_guis(&gui::pdr(player_id, resource));
        let message = self.players.get(&player_id).map(gui::pin);
        if let Some(message) = message {
            self.broadcast_to_guis(&message);
        }
        if let Some(player) = self.players.get(&player_id) {
            let tile_message = gui::bct(&self.map, player.x, player.y);
            self.broadcast_to_guis(&tile_message);
        }
    }
}
