#![allow(dead_code)]

use hecs::World;

use crate::colony::ColonyState;
use crate::components::{Ant, AntRole, Aphid, ColonyMember, Position};
use crate::config::SimConfig;
use crate::terrain::Terrain;

/// Food produced by aphid per tick when farmed
const APHID_FOOD_RATE: f32 = 0.1;

/// Ticks to claim an aphid
const CLAIM_TICKS: u32 = 50;

/// Distance to consider "near" an aphid
const NEARBY_DISTANCE: i32 = 2;

/// Spawn aphids underground near plant roots (surface)
pub fn spawn_aphids(world: &mut World, terrain: &Terrain, count: usize) {
    let mut spawned = 0;
    let mut attempts = 0;

    while spawned < count && attempts < count * 20 {
        attempts += 1;

        let x = fastrand::i32(0..terrain.width as i32);

        // Find surface Y, then go slightly underground
        let mut surface_y = 0;
        for y in 0..terrain.height as i32 {
            if !terrain.is_passable(x, y) {
                surface_y = y;
                break;
            }
        }

        // Aphids live on roots, 3-10 tiles below surface
        let depth = fastrand::i32(3..=10);
        let y = surface_y + depth;

        // Must be in a passable space (cave or tunnel)
        if y < terrain.height as i32 && terrain.is_passable(x, y) {
            world.spawn((
                Position { x, y },
                Aphid {
                    food_per_tick: APHID_FOOD_RATE,
                    colony_owner: None,
                },
            ));
            spawned += 1;
        }
    }
}

/// Aphid farming system - ants near aphids claim and farm them
pub fn aphid_system(world: &mut World, colonies: &mut [ColonyState], _config: &SimConfig) {
    // Collect ant positions by colony
    let mut ant_positions: Vec<(i32, i32, u8)> = Vec::new();
    for (_entity, (pos, ant, member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        if matches!(ant.role, AntRole::Worker | AntRole::Soldier) {
            ant_positions.push((pos.x, pos.y, member.colony_id));
        }
    }

    // Process each aphid
    let mut food_production: Vec<(u8, f32)> = Vec::new();
    let mut ownership_changes: Vec<(hecs::Entity, Option<u8>)> = Vec::new();

    for (entity, (pos, aphid)) in world.query::<(&Position, &Aphid)>().iter() {
        // Find nearby ants by colony
        let mut nearby_counts: [u32; 6] = [0; 6];

        for (ax, ay, colony_id) in &ant_positions {
            let dist = (pos.x - ax).abs() + (pos.y - ay).abs();
            if dist <= NEARBY_DISTANCE {
                let idx = (*colony_id as usize).min(5);
                nearby_counts[idx] += 1;
            }
        }

        // Determine ownership
        let mut max_count = 0;
        let mut max_colony: Option<u8> = None;
        for (i, count) in nearby_counts.iter().enumerate() {
            if *count > max_count {
                max_count = *count;
                max_colony = Some(i as u8);
            }
        }

        // Update ownership if different
        if max_colony != aphid.colony_owner && max_count > 0 {
            ownership_changes.push((entity, max_colony));
        } else if max_count == 0 && aphid.colony_owner.is_some() {
            // No ants nearby, aphid becomes wild again
            ownership_changes.push((entity, None));
        }

        // Produce food for owner
        if let Some(owner) = aphid.colony_owner {
            food_production.push((owner, aphid.food_per_tick));
        }
    }

    // Apply ownership changes
    for (entity, new_owner) in ownership_changes {
        if let Ok(mut aphid) = world.get::<&mut Aphid>(entity) {
            aphid.colony_owner = new_owner;
        }
    }

    // Apply food production
    for (colony_id, amount) in food_production {
        let idx = colony_id as usize;
        if idx < colonies.len() {
            // Accumulate fractional food
            colonies[idx].food_stored += amount as u32;
        }
    }
}

/// Render aphids - 'a' in green (wild) or colony color (owned)
pub fn aphid_char(aphid: &Aphid) -> (char, Option<u8>) {
    ('a', aphid.colony_owner)
}
