use crate::constants::{EMPTY_RESOURCE_COUNT, RESOURCE_INCREMENT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Food,
    Linemate,
    Deraumere,
    Sibur,
    Mendiane,
    Phiras,
    Thystame,
}

#[derive(Debug, Clone, Default)]
pub struct Tile {
    pub food: usize,
    pub linemate: usize,
    pub deraumere: usize,
    pub sibur: usize,
    pub mendiane: usize,
    pub phiras: usize,
    pub thystame: usize,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            food: EMPTY_RESOURCE_COUNT,
            linemate: EMPTY_RESOURCE_COUNT,
            deraumere: EMPTY_RESOURCE_COUNT,
            sibur: EMPTY_RESOURCE_COUNT,
            mendiane: EMPTY_RESOURCE_COUNT,
            phiras: EMPTY_RESOURCE_COUNT,
            thystame: EMPTY_RESOURCE_COUNT,
        }
    }

    pub fn resource_count(&self, resource: Resource) -> usize {
        match resource {
            Resource::Food => self.food,
            Resource::Linemate => self.linemate,
            Resource::Deraumere => self.deraumere,
            Resource::Sibur => self.sibur,
            Resource::Mendiane => self.mendiane,
            Resource::Phiras => self.phiras,
            Resource::Thystame => self.thystame,
        }
    }

    pub fn add_resource(&mut self, resource: Resource) {
        match resource {
            Resource::Food => self.food += RESOURCE_INCREMENT,
            Resource::Linemate => self.linemate += RESOURCE_INCREMENT,
            Resource::Deraumere => self.deraumere += RESOURCE_INCREMENT,
            Resource::Sibur => self.sibur += RESOURCE_INCREMENT,
            Resource::Mendiane => self.mendiane += RESOURCE_INCREMENT,
            Resource::Phiras => self.phiras += RESOURCE_INCREMENT,
            Resource::Thystame => self.thystame += RESOURCE_INCREMENT,
        }
    }

    pub fn remove_resource(&mut self, resource: Resource) -> bool {
        let resource_count = match resource {
            Resource::Food => &mut self.food,
            Resource::Linemate => &mut self.linemate,
            Resource::Deraumere => &mut self.deraumere,
            Resource::Sibur => &mut self.sibur,
            Resource::Mendiane => &mut self.mendiane,
            Resource::Phiras => &mut self.phiras,
            Resource::Thystame => &mut self.thystame,
        };

        if *resource_count == EMPTY_RESOURCE_COUNT {
            return false;
        }

        *resource_count -= RESOURCE_INCREMENT;
        true
    }
}

#[derive(Debug)]
pub struct GameMap {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
}

impl GameMap {
    pub fn new(width: usize, height: usize) -> Self {
        let tile_count = width * height;

        Self {
            width,
            height,
            tiles: vec![Tile::new(); tile_count],
        }
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        let index = self.index(x, y)?;

        self.tiles.get(index)
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        let index = self.index(x, y)?;

        self.tiles.get_mut(index)
    }

    pub fn get_tile_by_index(&self, index: usize) -> Option<&Tile> {
        self.tiles.get(index)
    }

    pub fn get_tile_by_index_mut(&mut self, index: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(index)
    }

    pub fn get_wrapped_tile(&self, x: isize, y: isize) -> &Tile {
        let wrapped_x = self.wrap_x(x);
        let wrapped_y = self.wrap_y(y);
        let index = wrapped_y * self.width + wrapped_x;

        &self.tiles[index]
    }

    pub fn get_wrapped_tile_mut(&mut self, x: isize, y: isize) -> &mut Tile {
        let wrapped_x = self.wrap_x(x);
        let wrapped_y = self.wrap_y(y);
        let index = wrapped_y * self.width + wrapped_x;

        &mut self.tiles[index]
    }

    pub fn wrap_x(&self, x: isize) -> usize {
        x.rem_euclid(self.width as isize) as usize
    }

    pub fn wrap_y(&self, y: isize) -> usize {
        y.rem_euclid(self.height as isize) as usize
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }

        Some(y * self.width + x)
    }
}

#[cfg(test)]
mod tests {
    use super::{GameMap, Resource};
    use crate::constants::{EMPTY_RESOURCE_COUNT, RESOURCE_INCREMENT};

    const TEST_MAP_WIDTH: usize = 10;
    const TEST_MAP_HEIGHT: usize = 5;
    const FIRST_TILE_POSITION: usize = 0;
    const OUTSIDE_LEFT_POSITION: isize = -1;

    #[test]
    fn creates_expected_number_of_tiles() {
        let map = GameMap::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT);

        assert_eq!(map.tile_count(), TEST_MAP_WIDTH * TEST_MAP_HEIGHT);
    }

    #[test]
    fn new_tile_contains_no_resources() {
        let map = GameMap::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT);

        let tile = map
            .get_tile(FIRST_TILE_POSITION, FIRST_TILE_POSITION)
            .expect("first tile should exist");

        assert_eq!(tile.resource_count(Resource::Food), EMPTY_RESOURCE_COUNT);
    }

    #[test]
    fn adds_and_removes_resource() {
        let mut map = GameMap::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT);

        let tile = map
            .get_tile_mut(FIRST_TILE_POSITION, FIRST_TILE_POSITION)
            .expect("first tile should exist");

        tile.add_resource(Resource::Food);

        assert_eq!(tile.resource_count(Resource::Food), RESOURCE_INCREMENT);

        assert!(tile.remove_resource(Resource::Food));

        assert_eq!(tile.resource_count(Resource::Food), EMPTY_RESOURCE_COUNT);
    }

    #[test]
    fn cannot_remove_missing_resource() {
        let mut map = GameMap::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT);

        let tile = map
            .get_tile_mut(FIRST_TILE_POSITION, FIRST_TILE_POSITION)
            .expect("first tile should exist");

        assert!(!tile.remove_resource(Resource::Food));
    }

    #[test]
    fn returns_none_for_invalid_coordinates() {
        let map = GameMap::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT);

        assert!(map.get_tile(TEST_MAP_WIDTH, FIRST_TILE_POSITION).is_none());

        assert!(map.get_tile(FIRST_TILE_POSITION, TEST_MAP_HEIGHT).is_none());
    }

    #[test]
    fn wraps_negative_x_position() {
        let map = GameMap::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT);

        assert_eq!(
            map.wrap_x(OUTSIDE_LEFT_POSITION),
            TEST_MAP_WIDTH - RESOURCE_INCREMENT
        );
    }

    #[test]
    fn wraps_position_after_right_edge() {
        let map = GameMap::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT);

        assert_eq!(map.wrap_x(TEST_MAP_WIDTH as isize), FIRST_TILE_POSITION);
    }
}
