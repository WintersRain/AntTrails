use hecs::World;

use crate::colony::ColonyState;
use crate::components::{Ant, AntRole, AntState, CarryItem, Carrying, ColonyMember, FoodSource, Position};
use crate::systems::pheromone::{PheromoneGrid, PheromoneType};
use crate::terrain::Terrain;

/// Food regrow interval (ticks)
const FOOD_REGROW_INTERVAL: u64 = 500;

/// Food amount when spawned
const INITIAL_FOOD_AMOUNT: u16 = 100;

/// Spawn food sources on the surface
pub fn spawn_food_sources(world: &mut World, terrain: &Terrain, count: usize) {
    let mut spawned = 0;
    let mut attempts = 0;

    while spawned < count && attempts < count * 10 {
        attempts += 1;

        let x = fastrand::i32(0..terrain.width as i32);

        // Find surface Y
        let mut y = 0;
        for check_y in 0..terrain.height as i32 {
            if !terrain.is_passable(x, check_y) {
                y = check_y - 1;
                break;
            }
        }

        if y > 0 && terrain.is_passable(x, y) {
            world.spawn((
                Position { x, y },
                FoodSource {
                    amount: INITIAL_FOOD_AMOUNT,
                    regrow_rate: 1,
                },
            ));
            spawned += 1;
        }
    }
}

/// Regrow food at existing food sources
pub fn food_regrow_system(world: &mut World, tick: u64) {
    if tick % FOOD_REGROW_INTERVAL != 0 {
        return;
    }

    for (_entity, food) in world.query::<&mut FoodSource>().iter() {
        if food.amount < INITIAL_FOOD_AMOUNT {
            food.amount = food.amount.saturating_add(food.regrow_rate as u16);
        }
    }
}

/// Workers forage for food
pub fn foraging_system(
    world: &mut World,
    _terrain: &Terrain,
    _pheromones: &PheromoneGrid,
    colonies: &mut [ColonyState],
) {
    // Collect food source positions and amounts
    let mut food_positions: Vec<(i32, i32, hecs::Entity)> = Vec::new();
    for (entity, (pos, food)) in world.query::<(&Position, &FoodSource)>().iter() {
        if food.amount > 0 {
            food_positions.push((pos.x, pos.y, entity));
        }
    }

    // Find ants that can pick up food
    let mut pickups: Vec<(hecs::Entity, hecs::Entity)> = Vec::new(); // (ant, food)
    let mut deposits: Vec<(u8, u8)> = Vec::new(); // (colony_id, amount)

    for (ant_entity, (pos, ant, member)) in
        world.query::<(&Position, &Ant, &ColonyMember)>().iter()
    {
        if ant.role != AntRole::Worker {
            continue;
        }

        match ant.state {
            AntState::Wandering => {
                // Check if at food source
                for (fx, fy, food_entity) in &food_positions {
                    if pos.x == *fx && pos.y == *fy {
                        pickups.push((ant_entity, *food_entity));
                        break;
                    }
                }
            }
            AntState::Carrying => {
                // Check if at home (near colony home position)
                let colony_id = member.colony_id as usize;
                if colony_id < colonies.len() {
                    let home_x = colonies[colony_id].home_x;
                    let home_y = colonies[colony_id].home_y;
                    let dist = (pos.x - home_x).abs() + (pos.y - home_y).abs();
                    if dist <= 3 {
                        deposits.push((member.colony_id, 10));
                    }
                }
            }
            _ => {}
        }
    }

    // Process pickups
    for (ant_entity, food_entity) in pickups {
        // Check food amount first
        let has_food = world
            .get::<&FoodSource>(food_entity)
            .map(|f| f.amount > 0)
            .unwrap_or(false);

        if has_food {
            // Reduce food amount
            if let Ok(mut food) = world.get::<&mut FoodSource>(food_entity) {
                food.amount -= 1;
            }

            // Change ant state to carrying
            if let Ok(mut ant) = world.get::<&mut Ant>(ant_entity) {
                ant.state = AntState::Carrying;
            }
            let _ = world.insert_one(ant_entity, Carrying { item: CarryItem::Food(10) });
        }
    }

    // Process deposits
    for (colony_id, amount) in deposits {
        let colony_id = colony_id as usize;
        if colony_id < colonies.len() {
            colonies[colony_id].food_stored += amount as u32;
        }
    }

    // Carrying state reset is handled in check_deposit function
}

/// Movement AI for foraging ants
pub fn foraging_movement(
    pos: &Position,
    ant: &Ant,
    member: &ColonyMember,
    terrain: &Terrain,
    pheromones: &PheromoneGrid,
    colonies: &[ColonyState],
) -> Option<(i32, i32)> {
    match ant.state {
        AntState::Wandering => {
            // Follow food pheromones if strong enough
            if let Some(dir) =
                crate::systems::pheromone::follow_pheromone(
                    pheromones,
                    pos.x,
                    pos.y,
                    member.colony_id,
                    PheromoneType::Food,
                    terrain,
                )
            {
                if pheromones.get(pos.x, pos.y, member.colony_id, PheromoneType::Food) > 0.01 {
                    return Some(dir);
                }
            }
            None // Use default random movement
        }
        AntState::Carrying => {
            // Move toward home using home pheromones or direct path
            let colony_id = member.colony_id as usize;
            if colony_id < colonies.len() {
                let home_x = colonies[colony_id].home_x;
                let home_y = colonies[colony_id].home_y;

                // Try to move toward home
                let dx = (home_x - pos.x).signum();
                let dy = (home_y - pos.y).signum();

                if dx != 0 || dy != 0 {
                    // Prefer direct path if passable
                    if terrain.is_passable(pos.x + dx, pos.y + dy) {
                        return Some((dx, dy));
                    }
                    // Try just horizontal or vertical
                    if dx != 0 && terrain.is_passable(pos.x + dx, pos.y) {
                        return Some((dx, 0));
                    }
                    if dy != 0 && terrain.is_passable(pos.x, pos.y + dy) {
                        return Some((0, dy));
                    }
                }

                // Fall back to home pheromones
                if let Some(dir) = crate::systems::pheromone::follow_pheromone(
                    pheromones,
                    pos.x,
                    pos.y,
                    member.colony_id,
                    PheromoneType::Home,
                    terrain,
                ) {
                    return Some(dir);
                }
            }
            None
        }
        _ => None,
    }
}

/// Check if ant has deposited food and should stop carrying
pub fn check_deposit(world: &mut World, colonies: &[ColonyState]) {
    let mut to_stop_carrying: Vec<hecs::Entity> = Vec::new();

    for (entity, (pos, ant, member, _carrying)) in
        world.query::<(&Position, &Ant, &ColonyMember, &Carrying)>().iter()
    {
        if ant.state != AntState::Carrying {
            continue;
        }

        let colony_id = member.colony_id as usize;
        if colony_id < colonies.len() {
            let home_x = colonies[colony_id].home_x;
            let home_y = colonies[colony_id].home_y;
            let dist = (pos.x - home_x).abs() + (pos.y - home_y).abs();
            if dist <= 3 {
                to_stop_carrying.push(entity);
            }
        }
    }

    for entity in to_stop_carrying {
        if let Ok(mut ant) = world.get::<&mut Ant>(entity) {
            ant.state = AntState::Wandering;
        }
        let _ = world.remove_one::<Carrying>(entity);
    }
}
