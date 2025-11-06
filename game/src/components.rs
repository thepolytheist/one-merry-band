//! ECS Components for One Merry Band
//!
//! This module defines all the components used in the Entity Component System (ECS)
//! architecture of the game.

use specs::{Component, VecStorage, NullStorage};

/// Represents a position on the game map
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Calculate Manhattan distance to another position
    pub fn distance_to(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

/// Core stats for all entities
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Stats {
    pub health: i32,
    pub max_health: i32,
    pub speed: i32,
    pub damage: i32,
}

impl Stats {
    pub fn new(health: i32, speed: i32, damage: i32) -> Self {
        Self {
            health,
            max_health: health,
            speed,
            damage,
        }
    }

    /// Check if the entity is alive
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    /// Apply damage to this entity
    pub fn take_damage(&mut self, damage: i32) {
        self.health = (self.health - damage).max(0);
    }

    /// Heal this entity
    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }
}

/// Tracks action points for turn-based gameplay
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct ActionPoints {
    pub current: i32,
    pub max: i32,
}

impl ActionPoints {
    pub fn new(max: i32) -> Self {
        Self {
            current: max,
            max,
        }
    }

    /// Refresh action points at the start of a turn
    pub fn refresh(&mut self) {
        self.current = self.max;
    }

    /// Spend action points, returns true if successful
    pub fn spend(&mut self, amount: i32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// Check if enough action points are available
    pub fn can_spend(&self, amount: i32) -> bool {
        self.current >= amount
    }
}

/// Types of adventurers available to the player
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdventurerType {
    Fighter,
    Archer,
    Wizard,
    Scout,
}

impl AdventurerType {
    /// Get the display character for this adventurer type
    pub fn display_char(&self) -> char {
        match self {
            AdventurerType::Fighter => '@',
            AdventurerType::Archer => '%',
            AdventurerType::Wizard => '&',
            AdventurerType::Scout => '$',
        }
    }

    /// Get the point cost for this adventurer type
    pub fn cost(&self) -> i32 {
        match self {
            AdventurerType::Fighter => 3,
            AdventurerType::Archer => 3,
            AdventurerType::Wizard => 5,
            AdventurerType::Scout => 1,
        }
    }

    /// Get the base stats for this adventurer type
    /// Stats are: (health, speed, damage)
    pub fn base_stats(&self) -> (i32, i32, i32) {
        match self {
            AdventurerType::Fighter => (12, 10, 10),
            AdventurerType::Archer => (8, 12, 8),
            AdventurerType::Wizard => (10, 10, 15),
            AdventurerType::Scout => (8, 8, 5),
        }
    }
}

/// Component marking an entity as an adventurer (player-controlled)
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Adventurer {
    pub adventurer_type: AdventurerType,
}

impl Adventurer {
    pub fn new(adventurer_type: AdventurerType) -> Self {
        Self { adventurer_type }
    }
}

/// Types of enemies in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    GiantRat,
    Goblin,
    Orc,
    Ogre,
}

impl EnemyType {
    /// Get the display character for this enemy type
    pub fn display_char(&self) -> char {
        match self {
            EnemyType::GiantRat => 'r',
            EnemyType::Goblin => 'g',
            EnemyType::Orc => 'o',
            EnemyType::Ogre => 'O',
        }
    }

    /// Get the base stats for this enemy type
    /// Stats are: (health, speed, damage)
    pub fn base_stats(&self) -> (i32, i32, i32) {
        match self {
            EnemyType::GiantRat => (5, 12, 4),
            EnemyType::Goblin => (8, 10, 6),
            EnemyType::Orc => (15, 8, 10),
            EnemyType::Ogre => (25, 6, 15),
        }
    }

    /// Get the name of this enemy type
    pub fn name(&self) -> &'static str {
        match self {
            EnemyType::GiantRat => "Giant Rat",
            EnemyType::Goblin => "Goblin",
            EnemyType::Orc => "Orc",
            EnemyType::Ogre => "Ogre",
        }
    }
}

/// Component marking an entity as an enemy
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Enemy {
    pub enemy_type: EnemyType,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType) -> Self {
        Self { enemy_type }
    }
}

/// Marker component for entities controlled by the player
#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct PlayerControlled;

/// Represents a renderable entity with a display character
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Renderable {
    pub glyph: char,
}

impl Renderable {
    pub fn new(glyph: char) -> Self {
        Self { glyph }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0, 0);
        let p2 = Position::new(3, 4);
        assert_eq!(p1.distance_to(&p2), 7);
    }

    #[test]
    fn test_stats_damage() {
        let mut stats = Stats::new(20, 10, 5);
        assert!(stats.is_alive());

        stats.take_damage(15);
        assert_eq!(stats.health, 5);
        assert!(stats.is_alive());

        stats.take_damage(10);
        assert_eq!(stats.health, 0);
        assert!(!stats.is_alive());
    }

    #[test]
    fn test_stats_healing() {
        let mut stats = Stats::new(20, 10, 5);
        stats.take_damage(15);
        stats.heal(10);
        assert_eq!(stats.health, 15);

        stats.heal(20);
        assert_eq!(stats.health, 20); // Can't exceed max health
    }

    #[test]
    fn test_action_points() {
        let mut ap = ActionPoints::new(2);
        assert!(ap.can_spend(2));
        assert!(ap.spend(1));
        assert_eq!(ap.current, 1);
        assert!(!ap.can_spend(2));

        ap.refresh();
        assert_eq!(ap.current, 2);
    }

    #[test]
    fn test_adventurer_costs() {
        assert_eq!(AdventurerType::Fighter.cost(), 3);
        assert_eq!(AdventurerType::Archer.cost(), 3);
        assert_eq!(AdventurerType::Wizard.cost(), 5);
        assert_eq!(AdventurerType::Scout.cost(), 1);
    }
}
