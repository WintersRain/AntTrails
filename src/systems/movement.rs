use hecs::World;

use crate::colony::ColonyState;
use crate::components::{Ant, AntRole, AntState, ColonyMember, Position};
use crate::config::SimConfig;
use crate::systems::pheromone::PheromoneGrid;
use crate::terrain::{Terrain, TerrainType};

/// Move ants based on their state
pub fn movement_system(
    world: &mut World,
    terrain: &Terrain,
    pheromones: &PheromoneGrid,
    colonies: &[ColonyState],
    config: &SimConfig,
) {
    // Collect moves to apply (can't mutate while iterating)
    let mut moves: Vec<(hecs::Entity, i32, i32)> = Vec::new();

    for (entity, (pos, ant, member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        // Skip immobile entities
        if matches!(ant.role, AntRole::Egg | AntRole::Larvae) {
            continue;
        }

        // Queens move rarely
        if ant.role == AntRole::Queen && fastrand::u8(..) > config.movement.queen_move_threshold {
            continue;
        }

        // Determine movement based on state
        let (dx, dy) = match ant.state {
            AntState::Wandering => random_movement(),
            AntState::Digging => dig_movement(pos, terrain),
            AntState::Returning => climb_movement(pos, terrain),
            AntState::Idle => {
                if fastrand::u8(..) < config.movement.idle_move_threshold {
                    random_movement()
                } else {
                    (0, 0)
                }
            }
            AntState::Carrying => {
                match crate::systems::food::foraging_movement(
                    pos, ant, member, terrain, pheromones, colonies, config,
                ) {
                    Some(dir) => dir,
                    None => random_movement(),
                }
            }
            AntState::Fighting => {
                match crate::systems::combat::fighting_movement(pos, member, pheromones) {
                    Some(dir) => dir,
                    None => random_movement(),
                }
            }
            AntState::Fleeing => {
                match crate::systems::combat::fleeing_movement(pos, pheromones, config) {
                    Some(dir) => dir,
                    None => random_movement(),
                }
            }
            AntState::Following => {
                match crate::systems::food::foraging_movement(
                    pos, ant, member, terrain, pheromones, colonies, config,
                ) {
                    Some(dir) => dir,
                    None => random_movement(),
                }
            }
        };

        if dx != 0 || dy != 0 {
            let new_x = pos.x + dx;
            let new_y = pos.y + dy;

            // Check if new position is valid
            if terrain.is_passable(new_x, new_y) {
                moves.push((entity, new_x, new_y));
            }
        }
    }

    // Apply moves
    for (entity, new_x, new_y) in moves {
        if let Ok(mut pos) = world.get::<&mut Position>(entity) {
            pos.x = new_x;
            pos.y = new_y;
        }
    }
}

/// Generate random movement direction
fn random_movement() -> (i32, i32) {
    // Bias slightly downward for digging behavior later
    let directions = [
        (0, -1), // up
        (0, 1),  // down
        (0, 1),  // down (extra weight)
        (-1, 0), // left
        (1, 0),  // right
        (-1, 1), // down-left
        (1, 1),  // down-right
        (0, 0),  // stay (sometimes)
    ];

    directions[fastrand::usize(..directions.len())]
}

/// Movement for digging ants - prefer moving into newly dug spaces
fn dig_movement(pos: &Position, terrain: &Terrain) -> (i32, i32) {
    // Priority order for digging movement: down, down-diagonal, sideways
    let directions = [
        (0, 1),  // down
        (-1, 1), // down-left
        (1, 1),  // down-right
        (-1, 0), // left
        (1, 0),  // right
    ];

    // Find first passable direction (after dig system has run)
    for (dx, dy) in directions {
        let nx = pos.x + dx;
        let ny = pos.y + dy;

        // Move into tunnel or air (recently dug)
        if matches!(
            terrain.get(nx, ny),
            Some(TerrainType::Air) | Some(TerrainType::Tunnel)
        ) {
            return (dx, dy);
        }
    }

    // If blocked, stay in place (will try to dig more next tick)
    (0, 0)
}

/// Movement for returning ants - climb back toward surface
fn climb_movement(pos: &Position, terrain: &Terrain) -> (i32, i32) {
    // Priority order for climbing: up, up-diagonal, sideways
    let directions = [
        (0, -1),  // up (priority)
        (-1, -1), // up-left
        (1, -1),  // up-right
        (-1, 0),  // left
        (1, 0),   // right
    ];

    // Find first passable direction going upward
    for (dx, dy) in directions {
        let nx = pos.x + dx;
        let ny = pos.y + dy;

        // Move into any passable space (air, tunnel, or surface)
        if terrain.is_passable(nx, ny) {
            return (dx, dy);
        }
    }

    // If blocked going up, try random lateral movement
    let lateral = [(-1, 0), (1, 0)];
    for (dx, dy) in lateral {
        let nx = pos.x + dx;
        let ny = pos.y + dy;
        if terrain.is_passable(nx, ny) && fastrand::bool() {
            return (dx, dy);
        }
    }

    // Stuck, stay in place
    (0, 0)
}
