use hecs::World;

use crate::components::{Ant, AntRole, AntState, ColonyMember, Position};
use crate::terrain::{Terrain, TerrainType};

/// Digging speed: lower = slower (1 in N chance per tick)
const DIG_CHANCE: u8 = 8; // ~12% chance to dig each tick

/// Chance to reinforce adjacent walls (1 in N)
const REINFORCE_CHANCE: u8 = 3; // ~33% chance to reinforce a wall

/// Process digging actions for ants in Digging state
pub fn dig_system(world: &mut World, terrain: &mut Terrain) {
    // Collect dig actions
    let mut digs: Vec<(i32, i32)> = Vec::new();

    for (_entity, (pos, ant, _member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        // Only workers can dig
        if ant.role != AntRole::Worker {
            continue;
        }

        // Only dig if in digging state
        if ant.state != AntState::Digging {
            continue;
        }

        // Slow down digging - only dig occasionally
        if fastrand::u8(..) >= DIG_CHANCE {
            continue;
        }

        // Find adjacent diggable tile (prefer downward)
        let dig_targets = [
            (pos.x, pos.y + 1),     // down (priority)
            (pos.x - 1, pos.y + 1), // down-left
            (pos.x + 1, pos.y + 1), // down-right
            (pos.x - 1, pos.y),     // left
            (pos.x + 1, pos.y),     // right
        ];

        for (tx, ty) in dig_targets {
            if terrain.is_diggable(tx, ty) {
                digs.push((tx, ty));
                break;
            }
        }
    }

    // Apply digs and reinforce tunnels
    for (x, y) in digs {
        // Dig creates a tunnel (reinforced passage that won't collapse)
        terrain.set(x, y, TerrainType::Tunnel);

        // Ants reinforce adjacent soil walls to prevent cave-ins
        reinforce_adjacent(terrain, x, y);
    }
}

/// Reinforce adjacent soil tiles to prevent cave-ins
fn reinforce_adjacent(terrain: &mut Terrain, x: i32, y: i32) {
    let neighbors = [
        (x - 1, y),     // left
        (x + 1, y),     // right
        (x, y - 1),     // up
        (x - 1, y - 1), // up-left
        (x + 1, y - 1), // up-right
    ];

    for (nx, ny) in neighbors {
        // Only reinforce soil that's adjacent to tunnels
        if terrain.is_diggable(nx, ny) && fastrand::u8(..) < REINFORCE_CHANCE {
            // Mark as dense soil (more stable)
            terrain.set(nx, ny, TerrainType::SoilDense);
        }
    }
}

/// AI system to decide when workers should dig
pub fn dig_ai_system(world: &mut World, terrain: &Terrain) {
    // Collect state changes
    let mut state_changes: Vec<(hecs::Entity, AntState)> = Vec::new();

    for (entity, (pos, ant, member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        // Only workers
        if ant.role != AntRole::Worker {
            continue;
        }

        let new_state = decide_worker_state(pos, ant, member, terrain);
        if new_state != ant.state {
            state_changes.push((entity, new_state));
        }
    }

    // Apply state changes
    for (entity, new_state) in state_changes {
        if let Ok(mut ant) = world.get::<&mut Ant>(entity) {
            ant.state = new_state;
        }
    }
}

/// Decide what state a worker should be in
fn decide_worker_state(
    pos: &Position,
    ant: &Ant,
    _member: &ColonyMember,
    terrain: &Terrain,
) -> AntState {
    // Check if there's diggable terrain nearby (below or to sides)
    let can_dig_down = terrain.is_diggable(pos.x, pos.y + 1);
    let can_dig_left = terrain.is_diggable(pos.x - 1, pos.y);
    let can_dig_right = terrain.is_diggable(pos.x + 1, pos.y);
    let can_dig_down_left = terrain.is_diggable(pos.x - 1, pos.y + 1);
    let can_dig_down_right = terrain.is_diggable(pos.x + 1, pos.y + 1);

    let can_dig =
        can_dig_down || can_dig_left || can_dig_right || can_dig_down_left || can_dig_down_right;

    // Check if standing on solid ground or surface
    let on_ground = !terrain.is_passable(pos.x, pos.y + 1)
        || terrain.get(pos.x, pos.y) == Some(TerrainType::Surface);

    // Check if we're deep underground (more likely to return)
    let is_underground = terrain.get(pos.x, pos.y) == Some(TerrainType::Tunnel);
    let is_on_surface = terrain.get(pos.x, pos.y) == Some(TerrainType::Surface);

    match ant.state {
        AntState::Wandering => {
            // Moderate chance to start digging (~19.5%) -- ants wander ~5 ticks before digging
            if can_dig && on_ground && fastrand::u8(..) < 50 {
                AntState::Digging
            } else {
                AntState::Wandering
            }
        }
        AntState::Digging => {
            // Keep digging if we can, otherwise go back to wandering
            if can_dig {
                // Chance to stop and return to surface increases with depth
                let return_chance = if is_underground { 15 } else { 3 };
                if fastrand::u8(..) < return_chance {
                    AntState::Returning
                } else {
                    AntState::Digging
                }
            } else {
                // Can't dig, go back up
                AntState::Returning
            }
        }
        AntState::Returning => {
            // Keep returning until we reach surface
            if is_on_surface {
                // Arrived at surface, start wandering again
                AntState::Wandering
            } else if can_dig && on_ground && fastrand::u8(..) < 30 {
                // Sometimes get distracted and dig again
                AntState::Digging
            } else {
                AntState::Returning
            }
        }
        AntState::Idle => {
            // Start wandering (low chance -- movement.rs owns this transition at ~35%)
            if fastrand::u8(..) < 5 {
                AntState::Wandering
            } else {
                AntState::Idle
            }
        }
        other => other, // Keep other states as-is for now
    }
}
