#![allow(dead_code)]

use hecs::World;

use crate::colony::ColonyState;
use crate::components::{Ant, AntState, ColonyMember, Position};
use crate::config::{PheromoneConfig, SimConfig};
use crate::terrain::Terrain;

/// Maximum pheromone strength
const MAX_PHEROMONE: f32 = 1.0;

/// Per-type decay rates (per tick, multiplicative)
const DECAY_FOOD: f32 = 0.02;    // Half-life ~34 ticks (~1.1s @30fps)
const DECAY_HOME: f32 = 0.005;   // Half-life ~138 ticks (~4.6s @30fps)
const DECAY_DANGER: f32 = 0.05;  // Half-life ~14 ticks (~0.5s @30fps)

/// Snap-to-zero threshold (eliminates lingering near-zero values)
const SNAP_TO_ZERO: f32 = 0.001;

/// Base deposit amounts (before adaptive scaling)
const DEPOSIT_FOOD_BASE: f32 = 0.05;
const DEPOSIT_HOME_BASE: f32 = 0.03;
const DEPOSIT_DANGER_BASE: f32 = 0.10;

/// Diffusion rate: fraction of pheromone that spreads to neighbors per tick
const DIFFUSION_RATE: f32 = 0.05;

/// Home pheromone deposit radius (Manhattan distance from nest)
const HOME_DEPOSIT_RADIUS: f32 = 30.0;

/// Pheromone types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PheromoneType {
    Food,   // Found food, follow me
    Home,   // Path back to nest
    Danger, // Enemy/hazard here
}

/// Pheromone grid stored in terrain
pub struct PheromoneGrid {
    pub width: usize,
    pub height: usize,
    /// Per-tile pheromone levels: [food, home, danger] per colony
    /// Layout: (y * width + x) * MAX_COLONIES * 3 + colony_id * 3 + type
    data: Vec<f32>,
    buffer: Vec<f32>,  // Diffusion scratch buffer (permanent, not per-tick allocated)
    pub max_colonies: usize,
}

impl PheromoneGrid {
    pub fn new(width: usize, height: usize, max_colonies: usize) -> Self {
        let size = width * height * max_colonies * 3;
        Self {
            width,
            height,
            data: vec![0.0; size],
            buffer: vec![0.0; size],
            max_colonies,
        }
    }

    fn index(&self, x: i32, y: i32, colony: u8, ptype: PheromoneType) -> Option<usize> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None;
        }
        let base = (y as usize * self.width + x as usize) * self.max_colonies * 3;
        let colony_offset = (colony as usize).min(self.max_colonies - 1) * 3;
        let type_offset = match ptype {
            PheromoneType::Food => 0,
            PheromoneType::Home => 1,
            PheromoneType::Danger => 2,
        };
        Some(base + colony_offset + type_offset)
    }

    pub fn get(&self, x: i32, y: i32, colony: u8, ptype: PheromoneType) -> f32 {
        self.index(x, y, colony, ptype)
            .map(|i| self.data[i])
            .unwrap_or(0.0)
    }

    pub fn deposit(&mut self, x: i32, y: i32, colony: u8, ptype: PheromoneType, amount: f32) {
        if let Some(i) = self.index(x, y, colony, ptype) {
            self.data[i] = (self.data[i] + amount).min(MAX_PHEROMONE);
        }
    }

    /// Adaptive deposit: effective amount decreases as cell concentration rises
    /// Formula: effective = base * (1.0 - current / MAX_PHEROMONE)
    pub fn deposit_adaptive(
        &mut self, x: i32, y: i32, colony: u8,
        ptype: PheromoneType, base_amount: f32,
    ) {
        if let Some(i) = self.index(x, y, colony, ptype) {
            let current = self.data[i];
            let effective = base_amount * (1.0 - current / MAX_PHEROMONE);
            self.data[i] = (current + effective).min(MAX_PHEROMONE);
        }
    }

    pub fn decay_all(&mut self) {
        // Data layout: strides of 3 per colony = [food, home, danger]
        // Process in strides of 3 to apply per-type rates
        for chunk in self.data.chunks_exact_mut(3) {
            // Food (index 0)
            chunk[0] *= 1.0 - DECAY_FOOD;
            if chunk[0] < SNAP_TO_ZERO { chunk[0] = 0.0; }
            // Home (index 1)
            chunk[1] *= 1.0 - DECAY_HOME;
            if chunk[1] < SNAP_TO_ZERO { chunk[1] = 0.0; }
            // Danger (index 2)
            chunk[2] *= 1.0 - DECAY_DANGER;
            if chunk[2] < SNAP_TO_ZERO { chunk[2] = 0.0; }
        }
    }

    /// Spread pheromone to 8 neighbors using double-buffer swap
    pub fn diffuse(&mut self, _config: &PheromoneConfig) {
        // Zero the buffer
        for v in self.buffer.iter_mut() {
            *v = 0.0;
        }

        let cardinal_weight: f32 = 1.0;
        let diagonal_weight: f32 = 0.707; // ~1/sqrt(2)
        let total_weight: f32 = 4.0 * cardinal_weight + 4.0 * diagonal_weight;

        let directions: [(i32, i32); 8] = [
            (0, -1), (0, 1), (-1, 0), (1, 0),     // Cardinal
            (-1, -1), (1, -1), (-1, 1), (1, 1),    // Diagonal
        ];

        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                for colony in 0..self.max_colonies as u8 {
                    for ptype in [PheromoneType::Food, PheromoneType::Home, PheromoneType::Danger] {
                        if let Some(i) = self.index(x, y, colony, ptype) {
                            let val = self.data[i];
                            if val < SNAP_TO_ZERO { continue; }

                            let spread = val * DIFFUSION_RATE;
                            self.buffer[i] += val - spread; // Cell keeps most of its value

                            // Spread to neighbors
                            for (dx, dy) in &directions {
                                if let Some(ni) = self.index(x + dx, y + dy, colony, ptype) {
                                    let weight = if dx.abs() + dy.abs() == 1 {
                                        cardinal_weight
                                    } else {
                                        diagonal_weight
                                    };
                                    self.buffer[ni] += spread * weight / total_weight;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Swap buffers (O(1) pointer swap, no allocation)
        std::mem::swap(&mut self.data, &mut self.buffer);
    }

    /// Get strongest pheromone direction for a colony
    pub fn get_gradient(
        &self,
        x: i32,
        y: i32,
        colony: u8,
        ptype: PheromoneType,
    ) -> Option<(i32, i32)> {
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
        let mut best_strength = self.get(x, y, colony, ptype);

        for (dx, dy) in directions {
            let strength = self.get(x + dx, y + dy, colony, ptype);
            if strength > best_strength {
                best_strength = strength;
                best_dir = Some((dx, dy));
            }
        }

        best_dir
    }

    /// Weighted random gradient selection: probability proportional to strength^2
    /// This replaces greedy "pick strongest" which fails under saturation
    pub fn get_gradient_weighted(
        &self, x: i32, y: i32, colony: u8, ptype: PheromoneType,
    ) -> Option<(i32, i32)> {
        let directions = [
            (0, -1), (0, 1), (-1, 0), (1, 0),
            (-1, -1), (1, -1), (-1, 1), (1, 1),
        ];

        // Collect neighbors with non-negligible pheromone
        let mut candidates: Vec<((i32, i32), f32)> = Vec::new();

        for (dx, dy) in directions {
            let strength = self.get(x + dx, y + dy, colony, ptype);
            if strength > 0.01 {
                candidates.push(((dx, dy), strength));
            }
        }

        if candidates.is_empty() {
            return None;
        }

        // Weighted random: probability proportional to strength^2
        // Squaring emphasizes stronger trails while allowing some exploration
        let total: f32 = candidates.iter().map(|(_, s)| s * s).sum();
        let mut roll = fastrand::f32() * total;

        for ((dx, dy), s) in &candidates {
            roll -= s * s;
            if roll <= 0.0 {
                return Some((*dx, *dy));
            }
        }

        // Fallback to last candidate (floating-point edge case)
        candidates.last().map(|((dx, dy), _)| (*dx, *dy))
    }
}

/// Decay all pheromones
pub fn pheromone_decay_system(pheromones: &mut PheromoneGrid, _config: &SimConfig) {
    pheromones.decay_all();
}

/// Ants deposit pheromones as they walk
pub fn pheromone_deposit_system(
    world: &World, pheromones: &mut PheromoneGrid, colonies: &[ColonyState], _config: &SimConfig,
) {
    for (_entity, (pos, ant, member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        let colony_id = member.colony_id;

        match ant.state {
            // Carrying ants lay FOOD pheromone (they found food, others should follow)
            AntState::Carrying => {
                pheromones.deposit_adaptive(
                    pos.x, pos.y, colony_id,
                    PheromoneType::Food, DEPOSIT_FOOD_BASE,
                );
            }
            // Wandering/Returning ants lay HOME pheromone near nest only
            AntState::Wandering | AntState::Returning => {
                if let Some(colony) = colonies.get(colony_id as usize) {
                    let dist = ((pos.x - colony.home_x).abs()
                        + (pos.y - colony.home_y).abs()) as f32;
                    let proximity = (1.0 - dist / HOME_DEPOSIT_RADIUS).max(0.0);
                    if proximity > 0.0 {
                        pheromones.deposit_adaptive(
                            pos.x, pos.y, colony_id,
                            PheromoneType::Home, DEPOSIT_HOME_BASE * proximity,
                        );
                    }
                }
            }
            // Digging ants leave faint home pheromone near nest
            AntState::Digging => {
                if let Some(colony) = colonies.get(colony_id as usize) {
                    let dist = ((pos.x - colony.home_x).abs()
                        + (pos.y - colony.home_y).abs()) as f32;
                    let proximity = (1.0 - dist / 20.0).max(0.0);
                    if proximity > 0.0 {
                        pheromones.deposit_adaptive(
                            pos.x, pos.y, colony_id,
                            PheromoneType::Home, DEPOSIT_HOME_BASE * 0.5 * proximity,
                        );
                    }
                }
            }
            // Other states don't deposit (combat system handles danger pheromone)
            _ => {}
        }
    }
}

/// Get movement direction based on pheromone following
pub fn follow_pheromone(
    pheromones: &PheromoneGrid,
    x: i32,
    y: i32,
    colony: u8,
    ptype: PheromoneType,
    terrain: &Terrain,
) -> Option<(i32, i32)> {
    if let Some((dx, dy)) = pheromones.get_gradient_weighted(x, y, colony, ptype) {
        if terrain.is_passable(x + dx, y + dy) {
            return Some((dx, dy));
        }
    }
    None
}
