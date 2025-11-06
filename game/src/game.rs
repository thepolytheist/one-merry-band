//! Main game logic and state management
//!
//! This module contains the core game loop, state management, and coordination
//! between different game systems.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use specs::{Builder, Entity, Join, World, WorldExt};
use std::time::Duration;

use crate::components::*;
use crate::level::Level;
use crate::systems::*;

/// Represents the current state of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    PartySelection,
    Playing,
    GameOver,
    Victory,
}

/// Main game struct that manages the entire game
pub struct Game {
    pub world: World,
    pub state: GameState,
    pub current_actor: Option<Entity>,
}

impl Game {
    /// Create a new game instance
    pub fn new() -> Self {
        let mut world = World::new();

        // Register all components
        world.register::<Position>();
        world.register::<Stats>();
        world.register::<ActionPoints>();
        world.register::<Adventurer>();
        world.register::<Enemy>();
        world.register::<PlayerControlled>();
        world.register::<Renderable>();

        // Insert resources
        world.insert(ActionQueue::new());

        Self {
            world,
            state: GameState::PartySelection,
            current_actor: None,
        }
    }

    /// Start a new game with the selected party
    pub fn start_game(&mut self, selected_adventurers: Vec<AdventurerType>) {
        // Generate level
        let level = Level::generate_simple(60, 40);

        // Find spawn positions
        let spawn_positions = level.find_random_positions(selected_adventurers.len() + 5);
        self.world.insert(level);

        // Create adventurers
        for (i, adventurer_type) in selected_adventurers.iter().enumerate() {
            if let Some(&(x, y)) = spawn_positions.get(i) {
                let (health, speed, damage) = adventurer_type.base_stats();

                self.world
                    .create_entity()
                    .with(Position::new(x, y))
                    .with(Stats::new(health, speed, damage))
                    .with(ActionPoints::new(2))
                    .with(Adventurer::new(*adventurer_type))
                    .with(PlayerControlled)
                    .with(Renderable::new(adventurer_type.display_char()))
                    .build();
            }
        }

        // Create enemies
        let enemy_types = [
            EnemyType::GiantRat,
            EnemyType::Goblin,
            EnemyType::Orc,
        ];

        for i in selected_adventurers.len()..spawn_positions.len() {
            if let Some(&(x, y)) = spawn_positions.get(i) {
                let enemy_type = enemy_types[(i - selected_adventurers.len()) % enemy_types.len()];
                let (health, speed, damage) = enemy_type.base_stats();

                self.world
                    .create_entity()
                    .with(Position::new(x, y))
                    .with(Stats::new(health, speed, damage))
                    .with(ActionPoints::new(2))
                    .with(Enemy::new(enemy_type))
                    .with(Renderable::new(enemy_type.display_char()))
                    .build();
            }
        }

        self.state = GameState::Playing;
        self.update_current_actor();
    }

    /// Update the current actor based on turn order
    pub fn update_current_actor(&mut self) {
        let current_actor = {
            let entities = self.world.entities();
            let stats = self.world.read_storage::<Stats>();
            let action_points = self.world.read_storage::<ActionPoints>();

            get_next_actor(&entities, &stats, &action_points)
        };

        self.current_actor = current_actor;

        // If no actor has action points, start a new round
        if self.current_actor.is_none() {
            self.start_new_round();

            let current_actor = {
                let entities = self.world.entities();
                let stats = self.world.read_storage::<Stats>();
                let action_points = self.world.read_storage::<ActionPoints>();

                get_next_actor(&entities, &stats, &action_points)
            };

            self.current_actor = current_actor;
        }
    }

    /// Start a new round, refreshing all action points
    fn start_new_round(&mut self) {
        let mut action_points = self.world.write_storage::<ActionPoints>();

        for ap in (&mut action_points).join() {
            ap.refresh();
        }
    }

    /// Handle player input during gameplay
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<bool> {
        match self.state {
            GameState::Playing => self.handle_playing_input(key),
            _ => Ok(false),
        }
    }

    /// Handle input during the playing state
    fn handle_playing_input(&mut self, key: KeyEvent) -> Result<bool> {
        // Check if it's the player's turn
        if let Some(current_entity) = self.current_actor {
            let is_player = self.world.read_storage::<PlayerControlled>().get(current_entity).is_some();

            if !is_player {
                // It's an enemy turn, process AI
                self.process_enemy_turn(current_entity);
                self.update_current_actor();
                return Ok(false);
            }

            // Handle player movement and actions
            let mut dx = 0;
            let mut dy = 0;
            let mut pass = false;

            match key.code {
                KeyCode::Up | KeyCode::Char('k') => dy = -1,
                KeyCode::Down | KeyCode::Char('j') => dy = 1,
                KeyCode::Left | KeyCode::Char('h') => dx = -1,
                KeyCode::Right | KeyCode::Char('l') => dx = 1,
                KeyCode::Char(' ') => pass = true,
                KeyCode::Char('q') => return Ok(true), // Quit
                _ => return Ok(false),
            }

            if pass {
                // Pass turn
                {
                    let mut action_points = self.world.write_storage::<ActionPoints>();
                    if let Some(ap) = action_points.get_mut(current_entity) {
                        ap.current = 0; // Spend all action points
                    }
                }
                self.update_current_actor();
                return Ok(false);
            }

            if dx != 0 || dy != 0 {
                // Try to move or attack
                let target = self.get_entity_at_offset(current_entity, dx, dy);

                if let Some(target_entity) = target {
                    // There's an entity at the target position, try to attack
                    let attack_successful = {
                        let mut stats = self.world.write_storage::<Stats>();
                        let mut action_points = self.world.write_storage::<ActionPoints>();
                        let positions = self.world.read_storage::<Position>();

                        CombatSystem::process_attack(
                            current_entity,
                            target_entity,
                            &mut stats,
                            &mut action_points,
                            &positions,
                        )
                    };

                    if attack_successful {
                        // Attack successful
                        self.check_game_over();
                        self.update_current_actor();
                    }
                } else {
                    // No entity, try to move
                    let move_successful = {
                        let mut positions = self.world.write_storage::<Position>();
                        let mut action_points = self.world.write_storage::<ActionPoints>();
                        let stats = self.world.read_storage::<Stats>();

                        MovementSystem::process_movement(
                            current_entity,
                            dx,
                            dy,
                            &mut positions,
                            &mut action_points,
                            &stats,
                            &self.world,
                        )
                    };

                    if move_successful {
                        // Movement successful
                        self.update_current_actor();
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get entity at an offset from the given entity
    fn get_entity_at_offset(&self, entity: Entity, dx: i32, dy: i32) -> Option<Entity> {
        let positions = self.world.read_storage::<Position>();

        if let Some(pos) = positions.get(entity) {
            let target_x = pos.x + dx;
            let target_y = pos.y + dy;

            let entities = self.world.entities();
            for (other_entity, other_pos) in (&entities, &positions).join() {
                if other_entity != entity && other_pos.x == target_x && other_pos.y == target_y {
                    return Some(other_entity);
                }
            }
        }

        None
    }

    /// Process an enemy's turn with simple AI
    fn process_enemy_turn(&mut self, enemy_entity: Entity) {
        // Simple AI: Find nearest player and move towards or attack
        let nearest_player = {
            let positions = self.world.read_storage::<Position>();
            let player_controlled = self.world.read_storage::<PlayerControlled>();

            if let Some(enemy_pos) = positions.get(enemy_entity) {
                // Find nearest player
                let mut nearest_player: Option<(Entity, i32)> = None;

                let entities = self.world.entities();
                for (entity, pos, _) in (&entities, &positions, &player_controlled).join() {
                    let distance = enemy_pos.distance_to(pos);
                    if nearest_player.is_none() || distance < nearest_player.unwrap().1 {
                        nearest_player = Some((entity, distance));
                    }
                }

                nearest_player
            } else {
                None
            }
        };

        if let Some((player_entity, distance)) = nearest_player {
            if distance == 1 {
                // Adjacent, attack
                let mut stats = self.world.write_storage::<Stats>();
                let mut action_points = self.world.write_storage::<ActionPoints>();
                let positions = self.world.read_storage::<Position>();

                CombatSystem::process_attack(
                    enemy_entity,
                    player_entity,
                    &mut stats,
                    &mut action_points,
                    &positions,
                );
            } else {
                // Move towards player
                let (dx, dy) = {
                    let positions = self.world.read_storage::<Position>();
                    let enemy_pos = positions.get(enemy_entity).unwrap();
                    let player_pos = positions.get(player_entity).unwrap();
                    let dx = (player_pos.x - enemy_pos.x).signum();
                    let dy = (player_pos.y - enemy_pos.y).signum();
                    (dx, dy)
                };

                let mut positions_mut = self.world.write_storage::<Position>();
                let mut action_points = self.world.write_storage::<ActionPoints>();
                let stats = self.world.read_storage::<Stats>();

                // Try to move in the direction of the player
                if !MovementSystem::process_movement(
                    enemy_entity,
                    dx,
                    dy,
                    &mut positions_mut,
                    &mut action_points,
                    &stats,
                    &self.world,
                ) {
                    // If that fails, try moving in just one direction
                    if dx != 0 {
                        MovementSystem::process_movement(
                            enemy_entity,
                            dx,
                            0,
                            &mut positions_mut,
                            &mut action_points,
                            &stats,
                            &self.world,
                        );
                    } else if dy != 0 {
                        MovementSystem::process_movement(
                            enemy_entity,
                            0,
                            dy,
                            &mut positions_mut,
                            &mut action_points,
                            &stats,
                            &self.world,
                        );
                    }
                }
            }
        }

        // Spend remaining action points
        {
            let mut action_points = self.world.write_storage::<ActionPoints>();
            if let Some(ap) = action_points.get_mut(enemy_entity) {
                ap.current = 0;
            }
        }

        self.check_game_over();
    }

    /// Check if the game is over (all players or all enemies dead)
    fn check_game_over(&mut self) {
        let entities = self.world.entities();
        let stats = self.world.read_storage::<Stats>();
        let player_controlled = self.world.read_storage::<PlayerControlled>();
        let enemies = self.world.read_storage::<Enemy>();

        let players_alive = (&entities, &stats, &player_controlled)
            .join()
            .any(|(_, s, _)| s.is_alive());

        let enemies_alive = (&entities, &stats, &enemies)
            .join()
            .any(|(_, s, _)| s.is_alive());

        if !players_alive {
            self.state = GameState::GameOver;
        } else if !enemies_alive {
            self.state = GameState::Victory;
        }
    }

    /// Process AI turns automatically until it's a player's turn
    pub fn process_ai_turns(&mut self) {
        while let Some(current_entity) = self.current_actor {
            let is_player = self.world.read_storage::<PlayerControlled>().get(current_entity).is_some();

            if is_player {
                break;
            }

            self.process_enemy_turn(current_entity);
            self.update_current_actor();

            // Check if game is over
            if self.state != GameState::Playing {
                break;
            }
        }
    }
}

/// Poll for keyboard input with a timeout
/// This function filters out non-keyboard events (like mouse scroll)
pub fn poll_input(timeout: Duration) -> Result<Option<KeyEvent>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => return Ok(Some(key)),
            // Consume and discard non-keyboard events (mouse, resize, etc.)
            _ => return Ok(None),
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = Game::new();
        assert_eq!(game.state, GameState::PartySelection);
    }

    #[test]
    fn test_start_game() {
        let mut game = Game::new();
        let adventurers = vec![
            AdventurerType::Fighter,
            AdventurerType::Scout,
        ];

        game.start_game(adventurers);
        assert_eq!(game.state, GameState::Playing);

        // Check that entities were created
        let entities = game.world.entities();
        let adventurer_storage = game.world.read_storage::<Adventurer>();
        let enemy_storage = game.world.read_storage::<Enemy>();

        let adventurer_count = (&entities, &adventurer_storage).join().count();
        let enemy_count = (&entities, &enemy_storage).join().count();

        assert_eq!(adventurer_count, 2);
        assert!(enemy_count > 0);
    }

    #[test]
    fn test_game_over_detection() {
        let mut game = Game::new();
        game.start_game(vec![AdventurerType::Fighter]);

        // Kill all players
        {
            let entities = game.world.entities();
            let mut stats = game.world.write_storage::<Stats>();
            let player_controlled = game.world.read_storage::<PlayerControlled>();

            for (_entity, stats_comp, _) in (&entities, &mut stats, &player_controlled).join() {
                stats_comp.health = 0;
            }
        }

        game.check_game_over();
        assert_eq!(game.state, GameState::GameOver);
    }
}
