use crate::constants::{
    MINIMUM_RESOURCE_QUANTITY, RESOURCE_DENSITY_DERAUMERE, RESOURCE_DENSITY_FOOD,
    RESOURCE_DENSITY_LINEMATE, RESOURCE_DENSITY_MENDIANE, RESOURCE_DENSITY_PHIRAS,
    RESOURCE_DENSITY_SIBUR, RESOURCE_DENSITY_THYSTAME,
};
use crate::map::{GameMap, Resource};
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone, Copy)]
pub struct ResourceRule {
    pub resource: Resource,
    pub density: f64,
}

const RESOURCE_RULES: [ResourceRule; 7] = [
    ResourceRule {
        resource: Resource::Food,
        density: RESOURCE_DENSITY_FOOD,
    },
    ResourceRule {
        resource: Resource::Linemate,
        density: RESOURCE_DENSITY_LINEMATE,
    },
    ResourceRule {
        resource: Resource::Deraumere,
        density: RESOURCE_DENSITY_DERAUMERE,
    },
    ResourceRule {
        resource: Resource::Sibur,
        density: RESOURCE_DENSITY_SIBUR,
    },
    ResourceRule {
        resource: Resource::Mendiane,
        density: RESOURCE_DENSITY_MENDIANE,
    },
    ResourceRule {
        resource: Resource::Phiras,
        density: RESOURCE_DENSITY_PHIRAS,
    },
    ResourceRule {
        resource: Resource::Thystame,
        density: RESOURCE_DENSITY_THYSTAME,
    },
];

pub fn spawn_initial_resources(game_map: &mut GameMap) {
    for rule in RESOURCE_RULES {
        let quantity = calculate_resource_quantity(game_map.tile_count(), rule.density);

        distribute_resource(game_map, rule.resource, quantity);
    }
}

pub fn calculate_resource_quantity(tile_count: usize, density: f64) -> usize {
    let calculated_quantity = (tile_count as f64 * density).round() as usize;

    calculated_quantity.max(MINIMUM_RESOURCE_QUANTITY)
}

fn distribute_resource(game_map: &mut GameMap, resource: Resource, quantity: usize) {
    let mut tile_indexes: Vec<usize> = (0..game_map.tile_count()).collect();
    let mut random_generator = thread_rng();

    tile_indexes.shuffle(&mut random_generator);

    for tile_index in tile_indexes.into_iter().take(quantity) {
        if let Some(tile) = game_map.get_tile_by_index_mut(tile_index) {
            tile.add_resource(resource);
        }
    }
}

pub fn count_map_resource(game_map: &GameMap, resource: Resource) -> usize {
    let mut total = 0;

    for tile_index in 0..game_map.tile_count() {
        if let Some(tile) = game_map.get_tile_by_index(tile_index) {
            total += tile.resource_count(resource);
        }
    }

    total
}

pub fn print_resource_totals(game_map: &GameMap) {
    println!(
        "Resources: food={}, linemate={}, deraumere={}, sibur={}, mendiane={}, phiras={}, thystame={}",
        count_map_resource(game_map, Resource::Food),
        count_map_resource(game_map, Resource::Linemate),
        count_map_resource(game_map, Resource::Deraumere),
        count_map_resource(game_map, Resource::Sibur),
        count_map_resource(game_map, Resource::Mendiane),
        count_map_resource(game_map, Resource::Phiras),
        count_map_resource(game_map, Resource::Thystame),
    );
}

#[cfg(test)]
mod tests {
    use super::{calculate_resource_quantity, count_map_resource, spawn_initial_resources};
    use crate::constants::{
        MINIMUM_RESOURCE_QUANTITY, RESOURCE_DENSITY_FOOD, RESOURCE_DENSITY_THYSTAME,
    };
    use crate::map::{GameMap, Resource};

    const TEST_MAP_WIDTH: usize = 10;
    const TEST_MAP_HEIGHT: usize = 10;
    const EXPECTED_FOOD_QUANTITY: usize = 50;
    const EXPECTED_THYSTAME_QUANTITY: usize = 5;
    const SMALL_MAP_TILE_COUNT: usize = 1;

    #[test]
    fn calculates_food_quantity() {
        let tile_count = TEST_MAP_WIDTH * TEST_MAP_HEIGHT;

        assert_eq!(
            calculate_resource_quantity(tile_count, RESOURCE_DENSITY_FOOD),
            EXPECTED_FOOD_QUANTITY
        );
    }

    #[test]
    fn calculates_thystame_quantity() {
        let tile_count = TEST_MAP_WIDTH * TEST_MAP_HEIGHT;

        assert_eq!(
            calculate_resource_quantity(tile_count, RESOURCE_DENSITY_THYSTAME),
            EXPECTED_THYSTAME_QUANTITY
        );
    }

    #[test]
    fn guarantees_minimum_resource_quantity() {
        assert_eq!(
            calculate_resource_quantity(SMALL_MAP_TILE_COUNT, RESOURCE_DENSITY_THYSTAME),
            MINIMUM_RESOURCE_QUANTITY
        );
    }

    #[test]
    fn spawns_expected_resources_on_map() {
        let mut game_map = GameMap::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT);

        spawn_initial_resources(&mut game_map);

        assert_eq!(
            count_map_resource(&game_map, Resource::Food),
            EXPECTED_FOOD_QUANTITY
        );

        assert_eq!(
            count_map_resource(&game_map, Resource::Thystame),
            EXPECTED_THYSTAME_QUANTITY
        );
    }
}
