//! ECS Systems for One Merry Band
//!
//! This module contains all the systems that operate on components to implement
//! game logic such as movement, combat, and turn management.

use specs::{Entities, Entity, Join, ReadStorage, System, WriteStorage, World};
use crate::components::*;
use crate::level::Level;

/// System for managing turn order based on entity speed
pub struct TurnOrderSystem;

impl<'a> System<'a> for TurnOrderSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Stats>,
        WriteStorage<'a, ActionPoints>,
    );

    fn run(&mut self, (entities, stats, mut action_points): Self::SystemData) {
        // Refresh action points for all entities at the start of each round
        for (_entity, _stats, ap) in (&entities, &stats, &mut action_points).join() {
            if ap.current == 0 {
                ap.refresh();
            }
        }
    }
}

/// Represents a movement request
pub struct MoveAction {
    pub entity: Entity,
    pub dx: i32,
    pub dy: i32,
}

/// Represents an attack action
pub struct AttackAction {
    pub attacker: Entity,
    pub target: Entity,
}

/// Game actions that can be performed
pub enum GameAction {
    Move(MoveAction),
    Attack(AttackAction),
    Pass(Entity),
}

/// Resource for storing pending actions
pub struct ActionQueue {
    pub actions: Vec<GameAction>,
}

impl ActionQueue {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    pub fn push(&mut self, action: GameAction) {
        self.actions.push(action);
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }
}

impl Default for ActionQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// System for processing movement actions
pub struct MovementSystem;

impl MovementSystem {
    /// Process a single movement action
    pub fn process_movement(
        entity: Entity,
        dx: i32,
        dy: i32,
        positions: &mut WriteStorage<Position>,
        action_points: &mut WriteStorage<ActionPoints>,
        _stats: &ReadStorage<Stats>,
        world: &World,
    ) -> bool {
        // Check if entity has enough action points
        if let Some(ap) = action_points.get_mut(entity) {
            if !ap.can_spend(1) {
                return false;
            }

            // Get current position
            if let Some(pos) = positions.get(entity) {
                let new_x = pos.x + dx;
                let new_y = pos.y + dy;

                // Check if the new position is valid
                if let Some(level) = world.try_fetch::<Level>() {
                    if !level.is_walkable(new_x, new_y) {
                        return false;
                    }

                    // Check if another entity is already at this position
                    // We need to use join() instead of get_mut to avoid double-borrow
                    use specs::Join;
                    let occupied = (&*positions)
                        .join()
                        .any(|other_pos| other_pos.x == new_x && other_pos.y == new_y);

                    if occupied {
                        return false;
                    }
                }

                // Now move the entity - get mutable access
                if let Some(pos_mut) = positions.get_mut(entity) {
                    pos_mut.x = new_x;
                    pos_mut.y = new_y;

                    // Spend action point
                    ap.spend(1);
                    return true;
                }
            }
        }

        false
    }
}

/// System for processing combat actions
pub struct CombatSystem;

impl CombatSystem {
    /// Process a single attack action
    pub fn process_attack(
        attacker: Entity,
        target: Entity,
        stats: &mut WriteStorage<Stats>,
        action_points: &mut WriteStorage<ActionPoints>,
        positions: &ReadStorage<Position>,
    ) -> bool {
        // Check if attacker has enough action points (attacking costs 2 AP)
        if let Some(attacker_ap) = action_points.get_mut(attacker) {
            if !attacker_ap.can_spend(2) {
                return false;
            }

            // Check if target is adjacent
            if let (Some(attacker_pos), Some(target_pos)) =
                (positions.get(attacker), positions.get(target)) {

                let distance = attacker_pos.distance_to(target_pos);
                if distance != 1 {
                    return false; // Not adjacent
                }

                // Get attacker damage
                if let Some(attacker_stats) = stats.get(attacker) {
                    let damage = attacker_stats.damage;

                    // Apply damage to target
                    if let Some(target_stats) = stats.get_mut(target) {
                        target_stats.take_damage(damage);

                        // Spend action points
                        attacker_ap.spend(2);
                        return true;
                    }
                }
            }
        }

        false
    }
}

/// System for removing dead entities
pub struct DeathSystem;

impl<'a> System<'a> for DeathSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Stats>,
    );

    fn run(&mut self, (entities, stats): Self::SystemData) {
        // Collect dead entities
        let dead: Vec<Entity> = (&entities, &stats)
            .join()
            .filter(|(_, stats)| !stats.is_alive())
            .map(|(entity, _)| entity)
            .collect();

        // Delete dead entities
        for entity in dead {
            let _ = entities.delete(entity);
        }
    }
}

/// Helper function to get the next entity to act based on action points and speed
pub fn get_next_actor(
    entities: &Entities,
    stats: &ReadStorage<Stats>,
    action_points: &ReadStorage<ActionPoints>,
) -> Option<Entity> {
    let mut best_entity: Option<Entity> = None;
    let mut best_speed = -1;

    for (entity, stats_comp, ap) in (entities, stats, action_points).join() {
        if ap.current > 0 && stats_comp.is_alive() {
            if stats_comp.speed > best_speed {
                best_speed = stats_comp.speed;
                best_entity = Some(entity);
            }
        }
    }

    best_entity
}

/// Check if any entity has action points remaining
pub fn has_active_actors(
    entities: &Entities,
    stats: &ReadStorage<Stats>,
    action_points: &ReadStorage<ActionPoints>,
) -> bool {
    (entities, stats, action_points)
        .join()
        .any(|(_, stats_comp, ap)| ap.current > 0 && stats_comp.is_alive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, WorldExt};

    #[test]
    fn test_action_queue() {
        let mut queue = ActionQueue::new();
        assert!(queue.actions.is_empty());

        queue.clear();
        assert!(queue.actions.is_empty());
    }

    #[test]
    fn test_get_next_actor() {
        let mut world = World::new();
        world.register::<Stats>();
        world.register::<ActionPoints>();

        let fast = world
            .create_entity()
            .with(Stats::new(10, 15, 5))
            .with(ActionPoints::new(2))
            .build();

        let slow = world
            .create_entity()
            .with(Stats::new(10, 5, 5))
            .with(ActionPoints::new(2))
            .build();

        let entities = world.entities();
        let stats = world.read_storage::<Stats>();
        let action_points = world.read_storage::<ActionPoints>();

        let next = get_next_actor(&entities, &stats, &action_points);
        assert_eq!(next, Some(fast));
    }

    #[test]
    fn test_has_active_actors() {
        let mut world = World::new();
        world.register::<Stats>();
        world.register::<ActionPoints>();

        world
            .create_entity()
            .with(Stats::new(10, 10, 5))
            .with(ActionPoints::new(2))
            .build();

        let entities = world.entities();
        let stats = world.read_storage::<Stats>();
        let action_points = world.read_storage::<ActionPoints>();

        assert!(has_active_actors(&entities, &stats, &action_points));
    }
}
