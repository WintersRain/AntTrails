#[derive(Clone, Debug)]
pub struct SimConfig {
    pub pheromone: PheromoneConfig,
    pub combat: CombatConfig,
    pub lifecycle: LifecycleConfig,
    pub movement: MovementConfig,
    pub food: FoodConfig,
    pub spawn: SpawnConfig,
    pub colony: ColonyConfig,
    pub water: WaterConfig,
    pub hazard: HazardConfig,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            pheromone: PheromoneConfig::default(),
            combat: CombatConfig::default(),
            lifecycle: LifecycleConfig::default(),
            movement: MovementConfig::default(),
            food: FoodConfig::default(),
            spawn: SpawnConfig::default(),
            colony: ColonyConfig::default(),
            water: WaterConfig::default(),
            hazard: HazardConfig::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PheromoneConfig {
    pub max_strength: f32,
    pub decay_food: f32,
    pub decay_home: f32,
    pub decay_danger: f32,
    pub snap_to_zero: f32,
    pub deposit_food: f32,
    pub deposit_home: f32,
    pub deposit_danger: f32,
    pub diffusion_rate: f32,
    pub home_deposit_radius: f32,
    pub dig_deposit_radius: f32,
    pub dig_deposit_multiplier: f32,
    pub gradient_threshold: f32,
}

impl Default for PheromoneConfig {
    fn default() -> Self {
        Self {
            max_strength: 1.0,
            decay_food: 0.02,
            decay_home: 0.005,
            decay_danger: 0.05,
            snap_to_zero: 0.001,
            deposit_food: 0.05,
            deposit_home: 0.03,
            deposit_danger: 0.10,
            diffusion_rate: 0.05,
            home_deposit_radius: 30.0,
            dig_deposit_radius: 20.0,
            dig_deposit_multiplier: 0.5,
            gradient_threshold: 0.01,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CombatConfig {
    pub base_damage: u8,
    pub combat_interval: u64,
    pub soldier_strength: u8,
    pub worker_strength: u8,
    pub other_strength: u8,
    pub danger_deposit_amount: f32,
    pub damage_random_range: u8,
    pub default_health: u8,
    pub default_fighter_strength: u8,
    pub fight_danger_threshold: f32,
    pub stop_fight_threshold: f32,
    pub flee_danger_threshold: f32,
    pub stop_flee_threshold: f32,
    pub max_colonies_scan: u8,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            base_damage: 10,
            combat_interval: 5,
            soldier_strength: 30,
            worker_strength: 10,
            other_strength: 5,
            danger_deposit_amount: 0.5,
            damage_random_range: 10,
            default_health: 50,
            default_fighter_strength: 10,
            fight_danger_threshold: 0.1,
            stop_fight_threshold: 0.05,
            flee_danger_threshold: 0.3,
            stop_flee_threshold: 0.1,
            max_colonies_scan: 6,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LifecycleConfig {
    pub egg_hatch_time: u32,
    pub larvae_mature_time: u32,
    pub queen_lay_interval: u32,
    pub food_per_egg: u32,
    pub worker_lifespan: u32,
    pub soldier_lifespan: u32,
    pub queen_lifespan: u32,
    pub food_consume_interval: u32,
    pub larvae_food_cost: u32,
    pub ant_food_cost: u32,
    pub worker_ratio_threshold: u8,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            egg_hatch_time: 200,
            larvae_mature_time: 300,
            queen_lay_interval: 100,
            food_per_egg: 10,
            worker_lifespan: 5000,
            soldier_lifespan: 3000,
            queen_lifespan: 50000,
            food_consume_interval: 50,
            larvae_food_cost: 2,
            ant_food_cost: 1,
            worker_ratio_threshold: 204, // 204/255 ~ 80% workers
        }
    }
}

#[derive(Clone, Debug)]
pub struct MovementConfig {
    pub queen_move_threshold: u8,
    pub idle_move_threshold: u8,
    pub dig_chance: u8,
    pub reinforce_chance: u8,
    pub start_dig_chance: u8,
    pub underground_return_chance: u8,
    pub surface_return_chance: u8,
    pub dig_distraction_chance: u8,
    pub idle_to_wander_chance_dig: u8,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            queen_move_threshold: 5,
            idle_move_threshold: 90,
            dig_chance: 8,
            reinforce_chance: 3,
            start_dig_chance: 50,
            underground_return_chance: 15,
            surface_return_chance: 3,
            dig_distraction_chance: 30,
            idle_to_wander_chance_dig: 5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FoodConfig {
    pub num_food_sources: usize,
    pub initial_amount: u16,
    pub regrow_interval: u64,
    pub regrow_rate: u8,
    pub deposit_distance: i32,
    pub food_per_deposit: u8,
    pub food_per_pickup: u8,
    pub food_pheromone_threshold: f32,
}

impl Default for FoodConfig {
    fn default() -> Self {
        Self {
            num_food_sources: 15,
            initial_amount: 100,
            regrow_interval: 500,
            regrow_rate: 1,
            deposit_distance: 3,
            food_per_deposit: 10,
            food_per_pickup: 10,
            food_pheromone_threshold: 0.01,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpawnConfig {
    pub num_colonies: usize,
    pub num_aphids: usize,
    pub initial_workers: usize,
    pub min_colony_distance: i32,
    pub aphid_food_rate: f32,
    pub aphid_claim_ticks: u32,
    pub aphid_nearby_distance: i32,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            num_colonies: 3,
            num_aphids: 10,
            initial_workers: 10,
            min_colony_distance: 40,
            aphid_food_rate: 0.1,
            aphid_claim_ticks: 50,
            aphid_nearby_distance: 2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColonyConfig {
    pub initial_food: u32,
}

impl Default for ColonyConfig {
    fn default() -> Self {
        Self {
            initial_food: 100,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WaterConfig {
    pub max_depth: u8,
    pub num_water_sources: usize,
    pub passable_threshold: u8,
    pub dangerous_threshold: u8,
    pub evaporation_max_depth: u8,
    pub stagnant_evaporation_ticks: u16,
    pub rain_chance: u32,
    pub rain_intensity_min: u8,
    pub rain_intensity_max: u8,
    pub rain_duration_min: u32,
    pub rain_duration_max: u32,
    pub rain_coverage_min: f32,
    pub rain_coverage_max: f32,
    pub drown_threshold_7: u32,
    pub drown_threshold_6: u32,
    pub drown_threshold_5: u32,
    pub drown_threshold_4: u32,
    pub flee_flood_depth: u8,
    pub water_flow_interval: u64,
    pub evaporation_interval: u64,
}

impl Default for WaterConfig {
    fn default() -> Self {
        Self {
            max_depth: 7,
            num_water_sources: 5,
            passable_threshold: 6,
            dangerous_threshold: 4,
            evaporation_max_depth: 2,
            stagnant_evaporation_ticks: 500,
            rain_chance: 10000,
            rain_intensity_min: 1,
            rain_intensity_max: 3,
            rain_duration_min: 200,
            rain_duration_max: 1000,
            rain_coverage_min: 0.3,
            rain_coverage_max: 0.8,
            drown_threshold_7: 1,
            drown_threshold_6: 3,
            drown_threshold_5: 10,
            drown_threshold_4: 30,
            flee_flood_depth: 2,
            water_flow_interval: 3,
            evaporation_interval: 50,
        }
    }
}

#[derive(Clone, Debug)]
pub struct HazardConfig {
    pub cave_in_interval: u64,
    pub dense_stability_bonus: u8,
    pub collapse_chance_3: u8,
    pub collapse_chance_4: u8,
    pub collapse_chance_5: u8,
    pub collapse_chance_6plus: u8,
}

impl Default for HazardConfig {
    fn default() -> Self {
        Self {
            cave_in_interval: 10,
            dense_stability_bonus: 2,
            collapse_chance_3: 1,
            collapse_chance_4: 3,
            collapse_chance_5: 10,
            collapse_chance_6plus: 25,
        }
    }
}
