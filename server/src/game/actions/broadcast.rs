use crate::constants::{KO_RESPONSE, OK_RESPONSE};
use crate::entities::player::Orientation;
use crate::game::GameState;
use crate::gui;
use crate::server::client::ClientId;
use std::f64::consts::FRAC_PI_4;

impl GameState {
    pub fn execute_broadcast(&mut self, token: ClientId, sender_id: usize, message: &str) {
        let Some(sender) = self.players.get(&sender_id) else {
            self.queue_to_client(token, KO_RESPONSE);
            return;
        };
        let sender_position = (sender.x, sender.y);
        let recipients = self
            .players
            .values()
            .filter(|player| player.id != sender_id)
            .map(|player| (player.id, player.x, player.y, player.orientation))
            .collect::<Vec<_>>();

        for (recipient_id, x, y, orientation) in recipients {
            let direction = broadcast_direction(
                (x, y),
                orientation,
                sender_position,
                self.map.width,
                self.map.height,
            );
            if let Some(recipient_token) = self.player_token(recipient_id) {
                self.queue_to_client(
                    recipient_token,
                    &format!("message {direction}, {message}\n"),
                );
            }
        }
        self.queue_to_client(token, OK_RESPONSE);
        self.broadcast_to_guis(&gui::pbc(sender_id, message));
    }
}

pub fn broadcast_direction(
    receiver: (usize, usize),
    receiver_orientation: Orientation,
    sender: (usize, usize),
    width: usize,
    height: usize,
) -> usize {
    let dx = shortest_delta(receiver.0, sender.0, width);
    let dy = shortest_delta(receiver.1, sender.1, height);
    if dx == 0 && dy == 0 {
        return 0;
    }
    let mut angle = (-(dx as f64)).atan2(-(dy as f64));
    if angle < 0.0 {
        angle += std::f64::consts::TAU;
    }
    let global_index = ((angle / FRAC_PI_4).round() as usize) % 8;
    let relative_index = (global_index + 8 - receiver_orientation.global_sound_index()) % 8;
    relative_index + 1
}

fn shortest_delta(from: usize, to: usize, size: usize) -> isize {
    let mut delta = to as isize - from as isize;
    let half = size as isize / 2;
    if delta > half {
        delta -= size as isize;
    } else if delta < -half {
        delta += size as isize;
    }
    delta
}

#[cfg(test)]
mod tests {
    use super::broadcast_direction;
    use crate::entities::player::Orientation;

    #[test]
    fn broadcast_uses_receiver_orientation() {
        assert_eq!(
            broadcast_direction((5, 5), Orientation::North, (5, 4), 10, 10),
            1
        );
        assert_eq!(
            broadcast_direction((5, 5), Orientation::East, (5, 4), 10, 10),
            3
        );
        assert_eq!(
            broadcast_direction((5, 5), Orientation::North, (5, 5), 10, 10),
            0
        );
    }
}
