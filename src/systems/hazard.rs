use hecs::World;

use crate::components::{Dead, Position};
use crate::config::SimConfig;
use crate::terrain::{Terrain, TerrainType};

/// Check for and process cave-ins
/// A tile is unstable if it's soil with too much air around/below it
/// Tunnels (ant-reinforced passages) prevent adjacent tiles from collapsing
pub fn cave_in_system(terrain: &mut Terrain, world: &mut World, _config: &SimConfig) {
    let width = terrain.width as i32;
    let height = terrain.height as i32;

    // Find unstable tiles that should collapse
    let mut collapses: Vec<(i32, i32)> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let tile = terrain.get(x, y);

            // Only soil can collapse (dense soil is more stable)
            if !matches!(tile, Some(TerrainType::Soil) | Some(TerrainType::SoilDense)) {
                continue;
            }

            // Skip if adjacent to a tunnel (ants reinforced it)
            if is_tunnel_supported(terrain, x, y) {
                continue;
            }

            // Check if tile below is air or tunnel (unsupported)
            let below = terrain.get(x, y + 1);
            if matches!(below, Some(TerrainType::Air) | Some(TerrainType::Tunnel)) {
                // Count air/tunnel neighbors
                let open_count = count_open_neighbors(terrain, x, y);

                // Dense soil is more stable
                let stability_bonus = if tile == Some(TerrainType::SoilDense) {
                    2
                } else {
                    0
                };

                // Collapse if too many open neighbors (unstable)
                // More open space = higher chance of collapse
                let collapse_chance = match open_count.saturating_sub(stability_bonus) {
                    0..=2 => 0,
                    3 => 1,
                    4 => 3,
                    5 => 10,
                    _ => 25,
                };

                if fastrand::u8(..) < collapse_chance {
                    collapses.push((x, y));
                }
            }
        }
    }

    // Process collapses - dirt falls down
    for (x, y) in collapses {
        // Find where the dirt lands (falls through air, stops at tunnel or solid)
        let mut land_y = y + 1;
        while land_y < height {
            let below = terrain.get(x, land_y);
            // Keep falling through air only, stop at tunnel or solid
            if below != Some(TerrainType::Air) {
                break;
            }
            land_y += 1;
        }
        land_y -= 1; // Back up to last air tile

        if land_y > y {
            // Move dirt from (x, y) to (x, land_y)
            let dirt_type = terrain.get(x, y).unwrap_or(TerrainType::Soil);
            terrain.set(x, y, TerrainType::Air);
            terrain.set(x, land_y, dirt_type);

            // Kill any ants at the landing spot
            kill_ants_at(world, x, land_y);
        }
    }
}

/// Count open (air or tunnel) tiles adjacent to a position
fn count_open_neighbors(terrain: &Terrain, x: i32, y: i32) -> u8 {
    let neighbors = [
        (x - 1, y - 1),
        (x, y - 1),
        (x + 1, y - 1),
        (x - 1, y),
        (x + 1, y),
        (x - 1, y + 1),
        (x, y + 1),
        (x + 1, y + 1),
    ];

    neighbors
        .iter()
        .filter(|(nx, ny)| {
            matches!(
                terrain.get(*nx, *ny),
                Some(TerrainType::Air) | Some(TerrainType::Tunnel)
            )
        })
        .count() as u8
}

/// Check if a tile is adjacent to a tunnel (ant-reinforced)
fn is_tunnel_supported(terrain: &Terrain, x: i32, y: i32) -> bool {
    let neighbors = [
        (x - 1, y),
        (x + 1, y),
        (x, y - 1),
        (x, y + 1),
    ];

    neighbors
        .iter()
        .any(|(nx, ny)| terrain.get(*nx, *ny) == Some(TerrainType::Tunnel))
}

/// Mark ants at a position as dead (crushed by falling dirt)
fn kill_ants_at(world: &mut World, x: i32, y: i32) {
    let mut to_kill: Vec<hecs::Entity> = Vec::new();

    for (entity, pos) in world.query::<&Position>().iter() {
        if pos.x == x && pos.y == y {
            to_kill.push(entity);
        }
    }

    for entity in to_kill {
        // Add Dead component to mark for removal
        let _ = world.insert_one(entity, Dead);
    }
}

/// Remove all entities marked as Dead
pub fn cleanup_dead(world: &mut World) {
    let dead: Vec<hecs::Entity> = world.query::<&Dead>().iter().map(|(e, _)| e).collect();

    for entity in dead {
        let _ = world.despawn(entity);
    }
}
