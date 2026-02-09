#![allow(dead_code)]

use hecs::World;

use crate::components::{Ant, AntState, Dead, Drowning, Position};
use crate::config::SimConfig;
use crate::terrain::Terrain;

/// Water cell data
#[derive(Clone, Copy, Default)]
pub struct WaterCell {
    pub depth: u8,
    pub pressure: u8,
    pub flow_dir: (i8, i8),
    pub stagnant: u16,
}

impl WaterCell {
    pub fn is_passable(&self) -> bool {
        self.depth < 6
    }

    pub fn is_dangerous(&self) -> bool {
        self.depth >= 4
    }

    pub fn movement_penalty(&self) -> f32 {
        match self.depth {
            0..=1 => 1.0,
            2 => 0.9,
            3 => 0.75,
            4 => 0.5,
            5 => 0.3,
            _ => 0.0,
        }
    }
}

/// Water grid
pub struct WaterGrid {
    pub width: usize,
    pub height: usize,
    pub max_depth: u8,
    cells: Vec<WaterCell>,
}

impl WaterGrid {
    pub fn new(width: usize, height: usize, max_depth: u8) -> Self {
        Self {
            width,
            height,
            max_depth,
            cells: vec![WaterCell::default(); width * height],
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None;
        }
        Some(y as usize * self.width + x as usize)
    }

    pub fn get(&self, x: i32, y: i32) -> WaterCell {
        self.index(x, y)
            .map(|i| self.cells[i])
            .unwrap_or_default()
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut WaterCell> {
        self.index(x, y).map(|i| &mut self.cells[i])
    }

    pub fn depth(&self, x: i32, y: i32) -> u8 {
        self.get(x, y).depth
    }

    pub fn add_water(&mut self, x: i32, y: i32, amount: u8) {
        let max = self.max_depth;
        if let Some(cell) = self.get_mut(x, y) {
            cell.depth = cell.depth.saturating_add(amount).min(max);
            cell.stagnant = 0;
        }
    }

    pub fn remove_water(&mut self, x: i32, y: i32, amount: u8) {
        if let Some(cell) = self.get_mut(x, y) {
            cell.depth = cell.depth.saturating_sub(amount);
        }
    }

    pub fn transfer(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32, amount: u8) {
        let from_depth = self.depth(from_x, from_y);
        let to_depth = self.depth(to_x, to_y);

        if from_depth >= amount && to_depth + amount <= self.max_depth {
            self.remove_water(from_x, from_y, amount);
            self.add_water(to_x, to_y, amount);

            // Set flow direction
            if let Some(cell) = self.get_mut(from_x, from_y) {
                cell.flow_dir = ((to_x - from_x) as i8, (to_y - from_y) as i8);
                cell.stagnant = 0;
            }
        }
    }
}

/// Calculate water pressure based on column height
pub fn calculate_pressure(water: &mut WaterGrid, terrain: &Terrain) {
    let max_depth = water.max_depth;
    for x in 0..water.width as i32 {
        for y in 0..water.height as i32 {
            let depth = water.depth(x, y);
            if depth == 0 {
                if let Some(cell) = water.get_mut(x, y) {
                    cell.pressure = 0;
                }
                continue;
            }

            // Calculate pressure from water column above
            let mut pressure = depth;
            let mut check_y = y - 1;

            while check_y >= 0 {
                let above_depth = water.depth(x, check_y);
                if above_depth == 0 || !terrain.is_passable(x, check_y) {
                    break;
                }
                pressure = pressure.saturating_add(above_depth);
                check_y -= 1;
            }

            if let Some(cell) = water.get_mut(x, y) {
                cell.pressure = pressure.min(max_depth);
            }
        }
    }
}

/// Water flow system - DF-style pressure-based flow
pub fn water_flow_system(water: &mut WaterGrid, terrain: &Terrain) {
    // Process in checkerboard pattern to avoid order-dependent artifacts
    for pass in 0..2 {
        for y in 0..water.height as i32 {
            for x in 0..water.width as i32 {
                if (x + y) % 2 != pass as i32 {
                    continue;
                }

                let cell = water.get(x, y);
                if cell.depth == 0 {
                    continue;
                }

                // Neighbor priorities: down > down-diagonal > sideways > up
                let neighbors = [
                    (x, y + 1, 2i32),      // Down (priority)
                    (x - 1, y + 1, 1),     // Down-left
                    (x + 1, y + 1, 1),     // Down-right
                    (x - 1, y, 0),         // Left
                    (x + 1, y, 0),         // Right
                    (x, y - 1, -1),        // Up (only under pressure)
                ];

                for (nx, ny, priority) in neighbors {
                    if !terrain.is_passable(nx, ny) {
                        continue;
                    }

                    let neighbor = water.get(nx, ny);

                    let should_flow = if priority > 0 {
                        // Downward: flow if room available
                        neighbor.depth < water.max_depth
                    } else if priority == 0 {
                        // Sideways: flow if neighbor has lower pressure and depth
                        neighbor.pressure < cell.pressure && neighbor.depth < cell.depth
                    } else {
                        // Upward: only under significant pressure
                        cell.pressure > neighbor.pressure + 2 && neighbor.depth < water.max_depth
                    };

                    if should_flow {
                        water.transfer(x, y, nx, ny, 1);
                        break;
                    }
                }
            }
        }
    }
}

/// Evaporation system - shallow exposed water evaporates
pub fn evaporation_system(water: &mut WaterGrid, terrain: &Terrain, config: &SimConfig) {
    for y in 0..water.height as i32 {
        for x in 0..water.width as i32 {
            let cell = water.get(x, y);

            if cell.depth > 0 && cell.depth <= config.water.evaporation_max_depth {
                // Check if exposed to air above
                let exposed = y == 0 || (terrain.is_passable(x, y - 1) && water.depth(x, y - 1) == 0);

                if exposed {
                    if let Some(cell) = water.get_mut(x, y) {
                        cell.stagnant += 1;

                        // Evaporate after being stagnant
                        if cell.stagnant > config.water.stagnant_evaporation_ticks {
                            cell.depth = cell.depth.saturating_sub(1);
                            cell.stagnant = 0;
                        }
                    }
                }
            }
        }
    }
}

/// Rain event
pub struct RainEvent {
    pub intensity: u8,
    pub duration: u32,
    pub coverage: f32,
}

/// Rain system
pub fn rain_system(water: &mut WaterGrid, terrain: &Terrain, event: &mut Option<RainEvent>, config: &SimConfig) {
    // Random chance to start rain
    if event.is_none() && fastrand::u32(..config.water.rain_chance) == 0 {
        *event = Some(RainEvent {
            intensity: fastrand::u8(config.water.rain_intensity_min..=config.water.rain_intensity_max),
            duration: fastrand::u32(config.water.rain_duration_min..config.water.rain_duration_max),
            coverage: fastrand::f32() * (config.water.rain_coverage_max - config.water.rain_coverage_min) + config.water.rain_coverage_min,
        });
    }

    if let Some(rain) = event {
        // Add water to surface
        for x in 0..water.width as i32 {
            if fastrand::f32() < rain.coverage {
                // Find surface Y
                for y in 0..water.height as i32 {
                    if !terrain.is_passable(x, y) {
                        // Add water just above surface
                        if y > 0 {
                            water.add_water(x, y - 1, rain.intensity);
                        }
                        break;
                    }
                }
            }
        }

        rain.duration = rain.duration.saturating_sub(1);
        if rain.duration == 0 {
            *event = None;
        }
    }
}

/// Drowning system - ants in deep water drown
pub fn drowning_system(world: &mut World, water: &WaterGrid, config: &SimConfig) {
    let mut to_start_drowning: Vec<hecs::Entity> = Vec::new();
    let mut to_stop_drowning: Vec<hecs::Entity> = Vec::new();
    let mut to_kill: Vec<hecs::Entity> = Vec::new();
    let mut to_increment: Vec<hecs::Entity> = Vec::new();

    for (entity, (pos, _ant)) in world.query::<(&Position, &Ant)>().iter() {
        let depth = water.depth(pos.x, pos.y);

        if depth >= config.water.dangerous_threshold {
            // In dangerous water
            if let Ok(drowning) = world.get::<&Drowning>(entity) {
                let drown_threshold = match depth {
                    7 => config.water.drown_threshold_7,
                    6 => config.water.drown_threshold_6,
                    5 => config.water.drown_threshold_5,
                    4 => config.water.drown_threshold_4,
                    _ => 999,
                };

                if drowning.ticks_submerged >= drown_threshold {
                    to_kill.push(entity);
                } else {
                    to_increment.push(entity);
                }
            } else {
                to_start_drowning.push(entity);
            }
        } else if world.get::<&Drowning>(entity).is_ok() {
            to_stop_drowning.push(entity);
        }
    }

    for entity in to_start_drowning {
        let _ = world.insert_one(entity, Drowning { ticks_submerged: 1 });
    }

    for entity in to_increment {
        if let Ok(mut drowning) = world.get::<&mut Drowning>(entity) {
            drowning.ticks_submerged += 1;
        }
    }

    for entity in to_stop_drowning {
        let _ = world.remove_one::<Drowning>(entity);
    }

    for entity in to_kill {
        let _ = world.insert_one(entity, Dead);
    }
}

/// Ants flee from rising water
pub fn flee_flood_system(world: &mut World, water: &WaterGrid, config: &SimConfig) {
    let mut to_flee: Vec<hecs::Entity> = Vec::new();

    for (entity, (pos, ant)) in world.query::<(&Position, &Ant)>().iter() {
        let depth = water.depth(pos.x, pos.y);

        if depth >= config.water.flee_flood_depth && ant.state != AntState::Fleeing && ant.state != AntState::Returning {
            to_flee.push(entity);
        }
    }

    for entity in to_flee {
        if let Ok(mut ant) = world.get::<&mut Ant>(entity) {
            ant.state = AntState::Returning; // Use returning to go up
        }
    }
}

/// Spawn water sources (aquifers, springs)
pub fn spawn_water_sources(water: &mut WaterGrid, terrain: &Terrain, count: usize) {
    let mut spawned = 0;
    let mut attempts = 0;

    while spawned < count && attempts < count * 20 {
        attempts += 1;

        let x = fastrand::i32(0..water.width as i32);
        let y = fastrand::i32((water.height as i32 / 2)..water.height as i32);

        // Place water in caves underground
        if terrain.is_passable(x, y) {
            water.add_water(x, y, fastrand::u8(3..=7));
            spawned += 1;
        }
    }
}
