use hecs::World;

use crate::colony::ColonyState;
use crate::components::{Age, Ant, AntRole, AntState, ColonyMember, Dead, Position};
use crate::config::SimConfig;

// Lifecycle timing (in ticks)
const EGG_HATCH_TIME: u32 = 200;
const LARVAE_MATURE_TIME: u32 = 300;
const QUEEN_LAY_INTERVAL: u32 = 100;
const FOOD_PER_EGG: u32 = 10;

// Lifespan (in ticks)
const WORKER_LIFESPAN: u32 = 5000;
const SOLDIER_LIFESPAN: u32 = 3000;
const QUEEN_LIFESPAN: u32 = 50000;

// Food consumption
const FOOD_CONSUME_INTERVAL: u32 = 50;
const LARVAE_FOOD_COST: u32 = 2;
const ANT_FOOD_COST: u32 = 1;

/// Main lifecycle system - handles aging, hatching, maturing, and death
pub fn lifecycle_system(world: &mut World, colonies: &mut [ColonyState], tick: u64, _config: &SimConfig) {
    // Process queen egg-laying
    queen_lay_eggs(world, colonies, tick);

    // Process egg hatching
    hatch_eggs(world, tick);

    // Process larvae maturing
    mature_larvae(world, tick);

    // Process aging and natural death
    age_and_die(world, tick);

    // Process food consumption
    if tick % FOOD_CONSUME_INTERVAL as u64 == 0 {
        consume_food(world, colonies);
    }
}

/// Queens lay eggs if colony has enough food
fn queen_lay_eggs(world: &mut World, colonies: &mut [ColonyState], tick: u64) {
    if tick % QUEEN_LAY_INTERVAL as u64 != 0 {
        return;
    }

    // Collect egg spawn info
    let mut eggs_to_spawn: Vec<(i32, i32, u8)> = Vec::new();

    for (_entity, (pos, ant, member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        if ant.role != AntRole::Queen {
            continue;
        }

        let colony_id = member.colony_id as usize;
        if colony_id >= colonies.len() {
            continue;
        }

        // Check if colony has enough food
        if colonies[colony_id].food_stored >= FOOD_PER_EGG {
            colonies[colony_id].food_stored -= FOOD_PER_EGG;
            eggs_to_spawn.push((pos.x, pos.y, member.colony_id));
        }
    }

    // Spawn eggs near queens
    for (x, y, colony_id) in eggs_to_spawn {
        // Spawn egg adjacent to queen
        let offsets = [(0, 1), (1, 0), (-1, 0), (0, -1), (1, 1), (-1, 1)];
        let (ox, oy) = offsets[fastrand::usize(..offsets.len())];

        world.spawn((
            Position { x: x + ox, y: y + oy },
            Ant {
                role: AntRole::Egg,
                state: AntState::Idle,
            },
            ColonyMember { colony_id },
            Age {
                ticks: 0,
                max_ticks: EGG_HATCH_TIME,
            },
        ));
    }
}

/// Eggs hatch into larvae after enough time
fn hatch_eggs(world: &mut World, _tick: u64) {
    let mut to_hatch: Vec<hecs::Entity> = Vec::new();

    for (entity, (ant, age)) in world.query::<(&Ant, &Age)>().iter() {
        if ant.role == AntRole::Egg && age.ticks >= age.max_ticks {
            to_hatch.push(entity);
        }
    }

    for entity in to_hatch {
        if let Ok(mut ant) = world.get::<&mut Ant>(entity) {
            ant.role = AntRole::Larvae;
        }
        if let Ok(mut age) = world.get::<&mut Age>(entity) {
            age.ticks = 0;
            age.max_ticks = LARVAE_MATURE_TIME;
        }
    }
}

/// Larvae mature into workers or soldiers
fn mature_larvae(world: &mut World, _tick: u64) {
    let mut to_mature: Vec<hecs::Entity> = Vec::new();

    for (entity, (ant, age)) in world.query::<(&Ant, &Age)>().iter() {
        if ant.role == AntRole::Larvae && age.ticks >= age.max_ticks {
            to_mature.push(entity);
        }
    }

    for entity in to_mature {
        // 80% workers, 20% soldiers
        let new_role = if fastrand::u8(..) < 204 {
            AntRole::Worker
        } else {
            AntRole::Soldier
        };

        let lifespan = match new_role {
            AntRole::Worker => WORKER_LIFESPAN,
            AntRole::Soldier => SOLDIER_LIFESPAN,
            _ => WORKER_LIFESPAN,
        };

        if let Ok(mut ant) = world.get::<&mut Ant>(entity) {
            ant.role = new_role;
            ant.state = AntState::Wandering;
        }
        if let Ok(mut age) = world.get::<&mut Age>(entity) {
            age.ticks = 0;
            age.max_ticks = lifespan;
        }
    }
}

/// Age all ants and kill those past their lifespan
fn age_and_die(world: &mut World, _tick: u64) {
    let mut to_die: Vec<hecs::Entity> = Vec::new();
    let mut to_age: Vec<hecs::Entity> = Vec::new();

    for (entity, (ant, age)) in world.query::<(&Ant, &Age)>().iter() {
        // Queens, workers, soldiers age
        if matches!(
            ant.role,
            AntRole::Queen | AntRole::Worker | AntRole::Soldier
        ) {
            if age.ticks >= age.max_ticks {
                to_die.push(entity);
            } else {
                to_age.push(entity);
            }
        } else {
            // Eggs and larvae age too
            to_age.push(entity);
        }
    }

    // Age entities
    for entity in to_age {
        if let Ok(mut age) = world.get::<&mut Age>(entity) {
            age.ticks += 1;
        }
    }

    // Mark dead entities
    for entity in to_die {
        let _ = world.insert_one(entity, Dead);
    }
}

/// Consume food from colonies based on population
fn consume_food(world: &mut World, colonies: &mut [ColonyState]) {
    // Count population per colony
    let mut food_needed: Vec<u32> = vec![0; colonies.len()];

    for (_entity, (ant, member)) in world.query::<(&Ant, &ColonyMember)>().iter() {
        let colony_id = member.colony_id as usize;
        if colony_id >= colonies.len() {
            continue;
        }

        let cost = match ant.role {
            AntRole::Larvae => LARVAE_FOOD_COST,
            AntRole::Queen | AntRole::Worker | AntRole::Soldier => ANT_FOOD_COST,
            AntRole::Egg => 0, // Eggs don't consume food
        };

        food_needed[colony_id] += cost;
    }

    // Deduct food
    for (i, colony) in colonies.iter_mut().enumerate() {
        if i < food_needed.len() {
            colony.food_stored = colony.food_stored.saturating_sub(food_needed[i]);
        }
    }
}

/// Add Age component to queens that don't have one
pub fn ensure_queen_ages(world: &mut World) {
    let mut queens_without_age: Vec<hecs::Entity> = Vec::new();

    for (entity, ant) in world.query::<&Ant>().iter() {
        if ant.role == AntRole::Queen && world.get::<&Age>(entity).is_err() {
            queens_without_age.push(entity);
        }
    }

    for entity in queens_without_age {
        let _ = world.insert_one(
            entity,
            Age {
                ticks: 0,
                max_ticks: QUEEN_LIFESPAN,
            },
        );
    }
}
