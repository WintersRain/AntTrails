use hecs::World;

use crate::colony::ColonyState;
use crate::components::{Ant, AntRole, AntState, ColonyMember, Position};
use crate::config::SimConfig;
use crate::terrain::{Terrain, TerrainType};

/// Spawn multiple colonies with queens and initial workers
pub fn spawn_colonies(
    world: &mut World,
    terrain: &Terrain,
    config: &SimConfig,
) -> Vec<ColonyState> {
    let num_colonies = config.spawn.num_colonies;
    let mut colonies = Vec::with_capacity(num_colonies);
    let mut spawn_positions: Vec<(i32, i32)> = Vec::new();

    for colony_id in 0..num_colonies {
        // Find a valid spawn position on the surface, away from other colonies
        if let Some((x, y)) = find_colony_spawn_position(terrain, &spawn_positions, config.spawn.min_colony_distance) {
            spawn_positions.push((x, y));

            // Create colony state
            let colony = ColonyState::new(colony_id as u8, x, y, config.colony.initial_food);

            // Spawn queen at surface
            spawn_ant(world, x, y, colony_id as u8, AntRole::Queen);

            // Spawn initial workers around queen
            for i in 0..config.spawn.initial_workers {
                let offset_x = (i as i32 % 5) - 2;
                let offset_y = i as i32 / 5;
                let worker_x = x + offset_x;
                let worker_y = y + offset_y;

                // Only spawn if position is valid (air or surface)
                if terrain.is_passable(worker_x, worker_y) {
                    spawn_ant(world, worker_x, worker_y, colony_id as u8, AntRole::Worker);
                } else {
                    // Try nearby positions
                    for dy in 0..3 {
                        for dx in -2..=2 {
                            let try_x = x + dx;
                            let try_y = y + dy;
                            if terrain.is_passable(try_x, try_y) {
                                spawn_ant(world, try_x, try_y, colony_id as u8, AntRole::Worker);
                                break;
                            }
                        }
                    }
                }
            }

            colonies.push(colony);
        }
    }

    colonies
}

/// Find a valid spawn position on the surface
fn find_colony_spawn_position(terrain: &Terrain, existing: &[(i32, i32)], min_colony_distance: i32) -> Option<(i32, i32)> {
    // Try random positions until we find a valid one
    for _ in 0..100 {
        let x = fastrand::i32(10..(terrain.width as i32 - 10));

        // Find surface at this x
        let mut surface_y = None;
        for y in 0..terrain.height as i32 {
            if terrain.get(x, y) == Some(TerrainType::Surface) {
                surface_y = Some(y);
                break;
            }
        }

        if let Some(y) = surface_y {
            // Check distance from existing colonies
            let too_close = existing
                .iter()
                .any(|(ex, ey)| (x - ex).abs() + (y - ey).abs() < min_colony_distance);

            if !too_close {
                return Some((x, y));
            }
        }
    }

    // Fallback: just find any surface tile
    for x in (10..terrain.width as i32 - 10).step_by(20) {
        for y in 0..terrain.height as i32 {
            if terrain.get(x, y) == Some(TerrainType::Surface) {
                return Some((x, y));
            }
        }
    }

    None
}

/// Spawn a single ant entity
fn spawn_ant(world: &mut World, x: i32, y: i32, colony_id: u8, role: AntRole) {
    let state = match role {
        AntRole::Queen => AntState::Idle,
        AntRole::Worker => AntState::Wandering,
        AntRole::Soldier => AntState::Wandering,
        AntRole::Egg | AntRole::Larvae => AntState::Idle,
    };

    world.spawn((
        Position { x, y },
        Ant { role, state },
        ColonyMember { colony_id },
    ));
}
