use hecs::World;

use crate::components::{Ant, AntRole, AntState, ColonyMember, Dead, Fighter, Position};
use crate::spatial::SpatialGrid;
use crate::systems::pheromone::{PheromoneGrid, PheromoneType};

/// Base damage for combat
const BASE_DAMAGE: u8 = 10;

/// Combat check interval (ticks)
const COMBAT_INTERVAL: u64 = 5;

/// Combat system - ants from different colonies fight when adjacent
pub fn combat_system(world: &mut World, pheromones: &mut PheromoneGrid, tick: u64, spatial_grid: &SpatialGrid) {
    if tick % COMBAT_INTERVAL != 0 {
        return;
    }

    // Collect all combatant positions
    let mut combatants: Vec<(hecs::Entity, i32, i32, u8, AntRole, u8)> = Vec::new(); // entity, x, y, colony, role, strength

    for (entity, (pos, ant, member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        // Only workers and soldiers fight
        if !matches!(ant.role, AntRole::Worker | AntRole::Soldier) {
            continue;
        }

        let strength = match ant.role {
            AntRole::Soldier => 30,
            AntRole::Worker => 10,
            _ => 5,
        };

        combatants.push((entity, pos.x, pos.y, member.colony_id, ant.role, strength));
    }

    // Find adjacent enemies using spatial grid and resolve combat
    let mut damage_to_apply: Vec<(hecs::Entity, u8, u8)> = Vec::new(); // entity, damage, attacker_colony
    let mut danger_deposits: Vec<(i32, i32, u8)> = Vec::new();
    let mut processed_pairs: Vec<(hecs::Entity, hecs::Entity)> = Vec::new();

    for &(entity_a, x_a, y_a, colony_a, role_a, strength_a) in &combatants {
        for (entity_b, x_b, y_b, colony_b) in spatial_grid.query_nearby(x_a, y_a) {
            // Skip same colony
            if colony_a == colony_b {
                continue;
            }

            // Skip if we already processed this pair (avoid double-counting)
            let pair = if entity_a < entity_b {
                (entity_a, entity_b)
            } else {
                (entity_b, entity_a)
            };
            if processed_pairs.contains(&pair) {
                continue;
            }

            // Check if adjacent (including diagonals)
            let dist = (x_a - x_b).abs().max((y_a - y_b).abs());
            if dist > 1 {
                continue;
            }

            // Find entity_b's combat stats from combatants list
            if let Some(&(_, _, _, _, role_b, strength_b)) = combatants.iter().find(|(e, _, _, _, _, _)| *e == entity_b) {
                // Combat! Each deals damage to the other
                let damage_a = calculate_damage(strength_a, role_a);
                let damage_b = calculate_damage(strength_b, role_b);

                damage_to_apply.push((entity_b, damage_a, colony_a));
                damage_to_apply.push((entity_a, damage_b, colony_b));

                // Deposit danger pheromones
                danger_deposits.push((x_a, y_a, colony_a));
                danger_deposits.push((x_b, y_b, colony_b));

                processed_pairs.push(pair);
            }
        }
    }

    // Apply damage
    for (entity, damage, _attacker_colony) in damage_to_apply {
        apply_damage(world, entity, damage);
    }

    // Deposit danger pheromones
    for (x, y, colony) in danger_deposits {
        pheromones.deposit(x, y, colony, PheromoneType::Danger, 0.5);
    }
}

/// Calculate damage dealt
fn calculate_damage(strength: u8, role: AntRole) -> u8 {
    let base = match role {
        AntRole::Soldier => BASE_DAMAGE * 2,
        AntRole::Worker => BASE_DAMAGE,
        _ => BASE_DAMAGE / 2,
    };

    // Add randomness and strength bonus
    let roll = fastrand::u8(0..10);
    let strength_bonus = strength / 10;
    base.saturating_add(roll).saturating_add(strength_bonus).saturating_sub(5)
}

/// Apply damage to an ant
fn apply_damage(world: &mut World, entity: hecs::Entity, damage: u8) {
    // Check if entity has Fighter component
    let current_health = world
        .get::<&Fighter>(entity)
        .ok()
        .map(|f| f.health);

    match current_health {
        Some(health) => {
            let new_health = health.saturating_sub(damage);
            if new_health == 0 {
                let _ = world.insert_one(entity, Dead);
            } else if let Ok(mut fighter) = world.get::<&mut Fighter>(entity) {
                fighter.health = new_health;
            }
        }
        None => {
            // Add Fighter component with default health
            let health = 50u8.saturating_sub(damage);
            if health == 0 {
                let _ = world.insert_one(entity, Dead);
            } else {
                let _ = world.insert_one(
                    entity,
                    Fighter {
                        strength: 10,
                        health,
                    },
                );
            }
        }
    }
}

/// Soldiers patrol and respond to danger pheromones
pub fn soldier_ai_system(world: &mut World, pheromones: &PheromoneGrid) {
    let mut state_changes: Vec<(hecs::Entity, AntState)> = Vec::new();

    for (entity, (pos, ant, member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        if ant.role != AntRole::Soldier {
            continue;
        }

        // Check for danger pheromones
        let danger = pheromones.get(pos.x, pos.y, member.colony_id, PheromoneType::Danger);

        if danger > 0.1 && ant.state != AntState::Fighting {
            // Move toward danger
            state_changes.push((entity, AntState::Fighting));
        } else if danger < 0.05 && ant.state == AntState::Fighting {
            // Return to wandering
            state_changes.push((entity, AntState::Wandering));
        }
    }

    for (entity, new_state) in state_changes {
        if let Ok(mut ant) = world.get::<&mut Ant>(entity) {
            ant.state = new_state;
        }
    }
}

/// Workers flee from enemies
pub fn flee_system(world: &mut World, pheromones: &PheromoneGrid) {
    let mut state_changes: Vec<(hecs::Entity, AntState)> = Vec::new();

    for (entity, (pos, ant, _member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        if ant.role != AntRole::Worker {
            continue;
        }

        // Check for danger pheromones (from any colony - means combat)
        let mut danger = 0.0f32;
        for c in 0..6u8 {
            danger = danger.max(pheromones.get(pos.x, pos.y, c, PheromoneType::Danger));
        }

        if danger > 0.3 && ant.state != AntState::Fleeing && ant.state != AntState::Carrying {
            state_changes.push((entity, AntState::Fleeing));
        } else if danger < 0.1 && ant.state == AntState::Fleeing {
            state_changes.push((entity, AntState::Wandering));
        }
    }

    for (entity, new_state) in state_changes {
        if let Ok(mut ant) = world.get::<&mut Ant>(entity) {
            ant.state = new_state;
        }
    }
}

/// Movement for fighting soldiers - move toward danger
pub fn fighting_movement(
    pos: &Position,
    member: &ColonyMember,
    pheromones: &PheromoneGrid,
) -> Option<(i32, i32)> {
    // Move toward danger pheromones
    pheromones.get_gradient(pos.x, pos.y, member.colony_id, PheromoneType::Danger)
}

/// Movement for fleeing workers - move away from danger
pub fn fleeing_movement(pos: &Position, pheromones: &PheromoneGrid) -> Option<(i32, i32)> {
    // Find direction with least danger
    let directions = [
        (0, -1),
        (0, 1),
        (-1, 0),
        (1, 0),
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1),
    ];

    let mut best_dir = None;
    let mut min_danger = f32::MAX;

    // Sum danger from all colonies at current position
    let mut current_danger = 0.0f32;
    for c in 0..6u8 {
        current_danger += pheromones.get(pos.x, pos.y, c, PheromoneType::Danger);
    }

    for (dx, dy) in directions {
        let mut danger = 0.0f32;
        for c in 0..6u8 {
            danger += pheromones.get(pos.x + dx, pos.y + dy, c, PheromoneType::Danger);
        }

        if danger < min_danger && danger < current_danger {
            min_danger = danger;
            best_dir = Some((dx, dy));
        }
    }

    best_dir
}
