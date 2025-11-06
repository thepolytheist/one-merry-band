//! Level generation and management
//!
//! This module handles the creation and management of game levels,
//! including random generation and tile management.

use rand::Rng;

/// Represents different types of tiles in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Floor,
    Wall,
}

impl Tile {
    /// Get the display character for this tile
    pub fn display_char(&self) -> char {
        match self {
            Tile::Floor => '.',
            Tile::Wall => '#',
        }
    }

    /// Check if this tile blocks movement
    pub fn blocks_movement(&self) -> bool {
        match self {
            Tile::Floor => false,
            Tile::Wall => true,
        }
    }
}

/// Represents a game level
pub struct Level {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Vec<Tile>>,
}

impl Level {
    /// Create a new level with the specified dimensions
    pub fn new(width: i32, height: i32) -> Self {
        let tiles = vec![vec![Tile::Wall; width as usize]; height as usize];
        Self {
            width,
            height,
            tiles,
        }
    }

    /// Generate a simple level with random rooms
    pub fn generate_simple(width: i32, height: i32) -> Self {
        let mut level = Self::new(width, height);
        let mut rng = rand::thread_rng();

        // Create a large open area in the center for the prototype
        let border = 2;
        let room_x1 = border;
        let room_y1 = border;
        let room_x2 = width - border;
        let room_y2 = height - border;

        // Fill the room area with floors
        for y in room_y1..room_y2 {
            for x in room_x1..room_x2 {
                level.tiles[y as usize][x as usize] = Tile::Floor;
            }
        }

        // Add some random walls for variety
        let num_obstacles = rng.gen_range(3..8);
        for _ in 0..num_obstacles {
            let x = rng.gen_range(room_x1 + 2..room_x2 - 2);
            let y = rng.gen_range(room_y1 + 2..room_y2 - 2);

            // Create small wall clusters
            for dx in 0..2 {
                for dy in 0..2 {
                    if rng.gen_bool(0.7) {
                        let nx = (x + dx).min(room_x2 - 1);
                        let ny = (y + dy).min(room_y2 - 1);
                        if level.is_in_bounds(nx, ny) {
                            level.tiles[ny as usize][nx as usize] = Tile::Wall;
                        }
                    }
                }
            }
        }

        level
    }

    /// Get the tile at the specified position
    pub fn get_tile(&self, x: i32, y: i32) -> Option<Tile> {
        if self.is_in_bounds(x, y) {
            Some(self.tiles[y as usize][x as usize])
        } else {
            None
        }
    }

    /// Check if a position is within the level bounds
    pub fn is_in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    /// Check if a position is walkable
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        if let Some(tile) = self.get_tile(x, y) {
            !tile.blocks_movement()
        } else {
            false
        }
    }

    /// Find a random walkable position
    pub fn find_random_walkable_position(&self) -> Option<(i32, i32)> {
        let mut rng = rand::thread_rng();

        // Try up to 100 times to find a walkable position
        for _ in 0..100 {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);

            if self.is_walkable(x, y) {
                return Some((x, y));
            }
        }

        None
    }

    /// Find multiple random walkable positions that don't overlap
    pub fn find_random_positions(&self, count: usize) -> Vec<(i32, i32)> {
        let mut positions = Vec::new();
        let mut attempts = 0;
        let max_attempts = 1000;

        while positions.len() < count && attempts < max_attempts {
            if let Some((x, y)) = self.find_random_walkable_position() {
                // Check if this position is already used
                if !positions.iter().any(|(px, py)| *px == x && *py == y) {
                    positions.push((x, y));
                }
            }
            attempts += 1;
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_creation() {
        let level = Level::new(20, 15);
        assert_eq!(level.width, 20);
        assert_eq!(level.height, 15);
        assert_eq!(level.tiles.len(), 15);
        assert_eq!(level.tiles[0].len(), 20);
    }

    #[test]
    fn test_bounds_checking() {
        let level = Level::new(20, 15);
        assert!(level.is_in_bounds(0, 0));
        assert!(level.is_in_bounds(19, 14));
        assert!(!level.is_in_bounds(-1, 0));
        assert!(!level.is_in_bounds(20, 15));
    }

    #[test]
    fn test_tile_blocking() {
        assert!(Tile::Wall.blocks_movement());
        assert!(!Tile::Floor.blocks_movement());
    }

    #[test]
    fn test_generate_simple() {
        let level = Level::generate_simple(30, 20);

        // Should have some floor tiles
        let mut has_floor = false;
        for row in &level.tiles {
            for tile in row {
                if *tile == Tile::Floor {
                    has_floor = true;
                    break;
                }
            }
        }
        assert!(has_floor);
    }

    #[test]
    fn test_find_random_positions() {
        let level = Level::generate_simple(30, 20);
        let positions = level.find_random_positions(5);

        // Should find positions (might be less than 5 in edge cases)
        assert!(!positions.is_empty());

        // All positions should be walkable
        for (x, y) in positions {
            assert!(level.is_walkable(x, y));
        }
    }
}
