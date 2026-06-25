use crate::game::GameState;
use crate::world::map::Resource;

impl GameState {
    pub fn look_response(&self, player_id: usize) -> Option<String> {
        let player = self.players.get(&player_id)?;
        let (forward_x, forward_y) = player.orientation.forward_delta();
        let (right_x, right_y) = player.orientation.right_delta();
        let mut response = String::from("[");
        let mut first_tile = true;

        for distance in 0..=player.level {
            for lateral in -(distance as isize)..=(distance as isize) {
                if !first_tile {
                    response.push(',');
                }
                first_tile = false;

                let x = player.x as isize + forward_x * distance as isize + right_x * lateral;
                let y = player.y as isize + forward_y * distance as isize + right_y * lateral;
                let (x, y) = self.map.wrapped_position(x, y);
                let content = self.tile_vision_content(x, y);

                if !content.is_empty() {
                    response.push(' ');
                    response.push_str(&content);
                }
            }
        }

        response.push_str(" ]\n");
        Some(response)
    }

    fn tile_vision_content(&self, x: usize, y: usize) -> String {
        let mut objects = Vec::new();

        let player_count = self
            .players
            .values()
            .filter(|player| player.x == x && player.y == y)
            .count();

        objects.extend(std::iter::repeat_n("player", player_count));

        if let Some(tile) = self.map.get_tile(x, y) {
            for resource in Resource::ALL {
                objects.extend(std::iter::repeat_n(
                    resource.protocol_name(),
                    tile.resource_count(resource),
                ));
            }
        }

        objects.join(" ")
    }
}
