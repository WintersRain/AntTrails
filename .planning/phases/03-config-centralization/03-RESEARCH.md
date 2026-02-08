# Phase 3: Config Centralization - Research

**Researched:** 2026-02-08
**Domain:** Rust config struct design, constant extraction from simulation codebase
**Confidence:** HIGH

## Summary

This phase centralizes approximately 37 named constants and 60+ inline magic numbers from 11 source files into a single `SimConfig` struct with nested sub-structs. The research consisted primarily of a thorough codebase audit to catalog every behavioral constant, determine which constants belong in config versus which should remain hardcoded, and define the exact struct hierarchy with correct types and default values.

The codebase is pure Rust with no external config framework needed. The pattern is straightforward: define structs with `Default` implementations, add a `config: SimConfig` field to `App`, and thread `&SimConfig` through system function signatures. No new dependencies are required.

**Primary recommendation:** Create `src/config.rs` with `SimConfig` and 9 sub-structs, wire it through `App` into each system, then do a mechanical find-and-replace of every constant and magic number with the corresponding config field access.

## Standard Stack

No new libraries needed. This is a pure Rust refactoring task.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (none needed) | - | Config is hardcoded Rust structs with `Default` | CONTEXT.md decision: no file loading this phase |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hardcoded Default | serde + toml | File-based loading adds complexity; deferred to future phase per CONTEXT.md |
| Runtime struct | const/static | Compile-time prevents future hot-reload; CONTEXT.md chose runtime struct |
| ECS resource | &SimConfig param | hecs has no built-in resource system; param passing is simpler and explicit |

## Architecture Patterns

### Recommended Project Structure
```
src/
  config.rs          # NEW: SimConfig + all sub-structs with Default impls
  app.rs             # MODIFIED: add config: SimConfig field, pass &self.config to systems
  systems/
    movement.rs      # MODIFIED: accept &SimConfig (or relevant sub-config)
    pheromone.rs      # MODIFIED: accept &SimConfig
    combat.rs         # MODIFIED: accept &SimConfig
    lifecycle.rs      # MODIFIED: accept &SimConfig
    food.rs           # MODIFIED: accept &SimConfig
    spawn.rs          # MODIFIED: accept &SimConfig
    dig.rs            # MODIFIED: accept &SimConfig
    aphid.rs          # MODIFIED: accept &SimConfig
    water.rs          # MODIFIED: accept &SimConfig
    hazard.rs         # MODIFIED: accept &SimConfig (for cave-in thresholds)
  main.rs            # MODIFIED: add `mod config;`
```

### Pattern 1: Nested Config Struct with Default

**What:** A top-level `SimConfig` containing named sub-structs, each with `impl Default` using current hardcoded values.
**When to use:** When you have many constants organized by system/domain.
**Example:**
```rust
// src/config.rs

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
```

### Pattern 2: Pass &SimConfig Through System Functions

**What:** Each system function receives `&SimConfig` (or a reference to the relevant sub-struct) as a parameter.
**When to use:** When you want explicit dependency without global state.
**Example:**
```rust
// Before:
pub fn combat_system(world: &mut World, pheromones: &mut PheromoneGrid, tick: u64, spatial_grid: &SpatialGrid) {

// After:
pub fn combat_system(world: &mut World, pheromones: &mut PheromoneGrid, tick: u64, spatial_grid: &SpatialGrid, config: &SimConfig) {
```

### Pattern 3: App Owns Config, Passes References

**What:** `App` struct stores `SimConfig` as a field, passes `&self.config` to system calls in `update()`.
**When to use:** Single-threaded game loop where one struct orchestrates all systems.
**Example:**
```rust
pub struct App {
    // ... existing fields ...
    config: SimConfig,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = SimConfig::default();
        // Use config values for initialization
        let colonies = systems::spawn::spawn_colonies(&mut world, &terrain, config.spawn.num_colonies);
        // ...
    }

    fn update(&mut self) {
        systems::combat::combat_system(&mut self.world, &mut self.pheromones, self.tick, &self.spatial_grid, &self.config);
        // ...
    }
}
```

### Anti-Patterns to Avoid
- **Global static config:** Using `lazy_static!` or `static` with `Mutex` adds synchronization overhead and makes testing harder. Pass `&SimConfig` explicitly instead.
- **Splitting config across multiple files:** Keeping sub-struct definitions scattered defeats the purpose. All config types live in `src/config.rs`.
- **Over-granular parameter passing:** Passing `&PheromoneConfig` instead of `&SimConfig` to each system seems cleaner but creates friction when a system needs values from multiple sub-structs. Pass the full `&SimConfig` and let each system access what it needs.
- **Changing behavior while centralizing:** This phase must be behavior-preserving. Every default value must exactly match the current hardcoded value. Resist the temptation to "fix" values while moving them.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Config file loading | Custom parser | serde + toml (future phase) | Parsing, validation, error messages are non-trivial |
| Config validation | Custom validators | Debug assertions for now | Full validation is overkill for hardcoded defaults |
| Hot reload | File watcher + reload | Just future-proof the struct design | Runtime struct already enables this path later |

**Key insight:** This phase is a pure mechanical refactoring. The "don't hand-roll" risk is minimal because we're not building infrastructure -- just moving constants into structs.

## Common Pitfalls

### Pitfall 1: Changing Values While Centralizing
**What goes wrong:** Accidentally using different values in the Default impl than what's currently hardcoded, changing simulation behavior.
**Why it happens:** Copy-paste errors, "fixing" values while moving them, type conversions losing precision.
**How to avoid:** For each constant, copy the exact value. Use the same numeric type. Run the simulation before and after to visually confirm identical behavior.
**Warning signs:** Simulation looks different after refactoring. Ants move differently. Colonies grow at different rates.

### Pitfall 2: Missing Inline Magic Numbers
**What goes wrong:** Named `const` values get centralized but inline magic numbers (like `50` in `fastrand::u8(..) < 50`) are missed.
**Why it happens:** `const` declarations are easy to find with grep, but inline numbers in expressions require reading every line.
**How to avoid:** The complete inventory below catalogues every magic number. Use the inventory as a checklist.
**Warning signs:** After centralization, the phrase "magic number" still applies to values in system code.

### Pitfall 3: Signature Churn Causing Merge Conflicts
**What goes wrong:** Adding `&SimConfig` to every system function changes many signatures, creating large diffs and potential merge issues.
**Why it happens:** Many functions are touched, and the call sites in `app.rs` all change too.
**How to avoid:** Change all signatures in one plan, all call sites in the same plan. Don't split the signature changes across multiple plans.
**Warning signs:** Compilation errors from mismatched signatures.

### Pitfall 4: Module Visibility Issues
**What goes wrong:** `config.rs` types aren't accessible from system modules, or system modules can't import sub-structs.
**Why it happens:** Rust's module system requires explicit `pub` and `use` declarations.
**How to avoid:** Make `SimConfig` and all sub-structs `pub`. Add `mod config;` to `main.rs` and `use crate::config::SimConfig;` in each system file that needs it.
**Warning signs:** Compiler errors about private types or unresolved imports.

### Pitfall 5: Forgetting Constants Used in app.rs
**What goes wrong:** `app.rs` has its own constants (`NUM_COLONIES`, `NUM_FOOD_SOURCES`, etc.) and inline magic numbers (tick interval checks like `self.tick % 10`, `self.tick % 3`, `self.tick % 50`) that also need centralization.
**Why it happens:** Focus on system files causes oversight of the orchestrator file.
**How to avoid:** The inventory below includes app.rs constants and inline numbers. Use the full inventory.
**Warning signs:** app.rs still contains `const` declarations or inline behavioral numbers after Phase 3.

## Complete Constant Inventory

This is the authoritative list of every constant and magic number to centralize. Values marked [LEAVE] should NOT be centralized per CONTEXT.md scope boundary (render/structural, not behavioral).

### File: src/app.rs

**Named constants:**
| Constant | Value | Type | Target Sub-struct | Field Name |
|----------|-------|------|-------------------|------------|
| `TARGET_FPS` | `30` | `u64` | [LEAVE] | Frame timing, structural |
| `FRAME_DURATION` | derived from TARGET_FPS | `Duration` | [LEAVE] | Frame timing, structural |
| `NUM_COLONIES` | `3` | `usize` | `SpawnConfig` | `num_colonies` |
| `NUM_FOOD_SOURCES` | `15` | `usize` | `FoodConfig` | `num_food_sources` |
| `NUM_APHIDS` | `10` | `usize` | `SpawnConfig` | `num_aphids` |
| `NUM_WATER_SOURCES` | `5` | `usize` | `WaterConfig` | `num_water_sources` |

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 59 | `200, 100` | `Terrain::generate(200, 100, seed)` | [LEAVE] | Terrain dimensions, structural |
| Line 86 | `8` | `SpatialGrid::new(..., 8)` | [LEAVE] | Spatial grid cell size, structural |
| Line 230 | `10` | `self.tick % 10` (cave-in interval) | `HazardConfig` | `cave_in_interval` |
| Line 235 | `3` | `self.tick % 3` (water physics interval) | `WaterConfig` | `water_flow_interval` |
| Line 242 | `50` | `self.tick % 50` (evaporation interval) | `WaterConfig` | `evaporation_interval` |

### File: src/systems/pheromone.rs

**Named constants:**
| Constant | Value | Type | Target Sub-struct | Field Name |
|----------|-------|------|-------------------|------------|
| `MAX_PHEROMONE` | `1.0` | `f32` | `PheromoneConfig` | `max_strength` |
| `DECAY_FOOD` | `0.02` | `f32` | `PheromoneConfig` | `decay_food` |
| `DECAY_HOME` | `0.005` | `f32` | `PheromoneConfig` | `decay_home` |
| `DECAY_DANGER` | `0.05` | `f32` | `PheromoneConfig` | `decay_danger` |
| `SNAP_TO_ZERO` | `0.001` | `f32` | `PheromoneConfig` | `snap_to_zero` |
| `DEPOSIT_FOOD_BASE` | `0.05` | `f32` | `PheromoneConfig` | `deposit_food` |
| `DEPOSIT_HOME_BASE` | `0.03` | `f32` | `PheromoneConfig` | `deposit_home` |
| `DEPOSIT_DANGER_BASE` | `0.10` | `f32` | `PheromoneConfig` | `deposit_danger` |
| `DIFFUSION_RATE` | `0.05` | `f32` | `PheromoneConfig` | `diffusion_rate` |
| `HOME_DEPOSIT_RADIUS` | `30.0` | `f32` | `PheromoneConfig` | `home_deposit_radius` |

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 213 | `0.01` | `strength > 0.01` (weighted gradient threshold) | `PheromoneConfig` | `gradient_threshold` |
| Line 278 | `20.0` | digging home deposit radius | `PheromoneConfig` | `dig_deposit_radius` |
| Line 280 | `0.5` | digging deposit multiplier | `PheromoneConfig` | `dig_deposit_multiplier` |

### File: src/systems/combat.rs

**Named constants:**
| Constant | Value | Type | Target Sub-struct | Field Name |
|----------|-------|------|-------------------|------------|
| `BASE_DAMAGE` | `10` | `u8` | `CombatConfig` | `base_damage` |
| `COMBAT_INTERVAL` | `5` | `u64` | `CombatConfig` | `combat_interval` |

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 29 | `30` | soldier strength | `CombatConfig` | `soldier_strength` |
| Line 30 | `10` | worker strength | `CombatConfig` | `worker_strength` |
| Line 31 | `5` | other strength | `CombatConfig` | `other_strength` |
| Line 90 | `0.5` | danger pheromone deposit on combat | `CombatConfig` | `danger_deposit_amount` |
| Line 103 | `10` | random damage range upper bound | `CombatConfig` | `damage_random_range` |
| Line 127 | `50` | default fighter health | `CombatConfig` | `default_health` |
| Line 133 | `10` | default fighter strength | `CombatConfig` | `default_fighter_strength` |
| Line 155 | `0.1` | danger threshold to start fighting | `CombatConfig` | `fight_danger_threshold` |
| Line 158 | `0.05` | danger threshold to stop fighting | `CombatConfig` | `stop_fight_threshold` |
| Line 186 | `0.3` | danger threshold to start fleeing | `CombatConfig` | `flee_danger_threshold` |
| Line 188 | `0.1` | danger threshold to stop fleeing | `CombatConfig` | `stop_flee_threshold` |
| Line 182,229,235 | `6` | max colonies iterated for danger check | `CombatConfig` | `max_colonies_scan` |

### File: src/systems/lifecycle.rs

**Named constants:**
| Constant | Value | Type | Target Sub-struct | Field Name |
|----------|-------|------|-------------------|------------|
| `EGG_HATCH_TIME` | `200` | `u32` | `LifecycleConfig` | `egg_hatch_time` |
| `LARVAE_MATURE_TIME` | `300` | `u32` | `LifecycleConfig` | `larvae_mature_time` |
| `QUEEN_LAY_INTERVAL` | `100` | `u32` | `LifecycleConfig` | `queen_lay_interval` |
| `FOOD_PER_EGG` | `10` | `u32` | `LifecycleConfig` | `food_per_egg` |
| `WORKER_LIFESPAN` | `5000` | `u32` | `LifecycleConfig` | `worker_lifespan` |
| `SOLDIER_LIFESPAN` | `3000` | `u32` | `LifecycleConfig` | `soldier_lifespan` |
| `QUEEN_LIFESPAN` | `50000` | `u32` | `LifecycleConfig` | `queen_lifespan` |
| `FOOD_CONSUME_INTERVAL` | `50` | `u32` | `LifecycleConfig` | `food_consume_interval` |
| `LARVAE_FOOD_COST` | `2` | `u32` | `LifecycleConfig` | `larvae_food_cost` |
| `ANT_FOOD_COST` | `1` | `u32` | `LifecycleConfig` | `ant_food_cost` |

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 122 | `204` | worker/soldier ratio threshold (204/255 = ~80% worker) | `LifecycleConfig` | `worker_ratio_threshold` |

### File: src/systems/food.rs

**Named constants:**
| Constant | Value | Type | Target Sub-struct | Field Name |
|----------|-------|------|-------------------|------------|
| `FOOD_REGROW_INTERVAL` | `500` | `u64` | `FoodConfig` | `regrow_interval` |
| `INITIAL_FOOD_AMOUNT` | `100` | `u16` | `FoodConfig` | `initial_amount` |

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 39 | `1` | food regrow_rate for spawned sources | `FoodConfig` | `regrow_rate` |
| Line 102 | `3` | deposit distance threshold (manhattan) | `FoodConfig` | `deposit_distance` |
| Line 103 | `10` | food deposit amount per trip | `FoodConfig` | `food_per_deposit` |
| Line 129 | `10` | food carried per pickup (CarryItem::Food(10)) | `FoodConfig` | `food_per_pickup` |
| Line 166 | `0.01` | food pheromone follow threshold | `FoodConfig` | `food_pheromone_threshold` |
| Line 231 | `3` | deposit check distance (same as L102) | (same as `deposit_distance`) | - |

### File: src/systems/spawn.rs

**Named constants:**
| Constant | Value | Type | Target Sub-struct | Field Name |
|----------|-------|------|-------------------|------------|
| `INITIAL_WORKERS` | `10` | `usize` | `SpawnConfig` | `initial_workers` |
| `MIN_COLONY_DISTANCE` | `40` | `i32` | `SpawnConfig` | `min_colony_distance` |

### File: src/systems/dig.rs

**Named constants:**
| Constant | Value | Type | Target Sub-struct | Field Name |
|----------|-------|------|-------------------|------------|
| `DIG_CHANCE` | `8` | `u8` | `MovementConfig` | `dig_chance` |
| `REINFORCE_CHANCE` | `3` | `u8` | `MovementConfig` | `reinforce_chance` |

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 132 | `50` | wandering-to-digging probability threshold | `MovementConfig` | `start_dig_chance` |
| Line 142 | `15` | underground return chance | `MovementConfig` | `underground_return_chance` |
| Line 143 | `3` | surface return chance | `MovementConfig` | `surface_return_chance` |
| Line 158 | `30` | returning-to-digging distraction chance | `MovementConfig` | `dig_distraction_chance` |
| Line 167 | `5` | idle-to-wandering chance (dig.rs) | `MovementConfig` | `idle_to_wander_chance_dig` |

### File: src/systems/movement.rs

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 25 | `5` | queen movement threshold (queen moves if random u8 <= 5) | `MovementConfig` | `queen_move_threshold` |
| Line 35 | `90` | idle movement threshold | `MovementConfig` | `idle_move_threshold` |

### File: src/systems/aphid.rs

**Named constants:**
| Constant | Value | Type | Target Sub-struct | Field Name |
|----------|-------|------|-------------------|------------|
| `APHID_FOOD_RATE` | `0.1` | `f32` | `SpawnConfig` | `aphid_food_rate` |
| `CLAIM_TICKS` | `50` | `u32` | `SpawnConfig` | `aphid_claim_ticks` |
| `NEARBY_DISTANCE` | `2` | `i32` | `SpawnConfig` | `aphid_nearby_distance` |

### File: src/systems/water.rs

**Named constants:**
| Constant | Value | Type | Target Sub-struct | Field Name |
|----------|-------|------|-------------------|------------|
| `MAX_WATER_DEPTH` | `7` | `u8` | `WaterConfig` | `max_depth` |

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 22 | `6` | water passable threshold (depth < 6) | `WaterConfig` | `passable_threshold` |
| Line 26 | `4` | water dangerous threshold (depth >= 4) | `WaterConfig` | `dangerous_threshold` |
| Line 199 | `2` | evaporation max depth threshold | `WaterConfig` | `evaporation_max_depth` |
| Line 208 | `500` | stagnant ticks before evaporation | `WaterConfig` | `stagnant_evaporation_ticks` |
| Line 229 | `10000` | rain chance denominator (1 in 10000) | `WaterConfig` | `rain_chance` |
| Line 231 | `1..=3` | rain intensity range | `WaterConfig` | `rain_intensity_min`, `rain_intensity_max` |
| Line 232 | `200..1000` | rain duration range | `WaterConfig` | `rain_duration_min`, `rain_duration_max` |
| Line 233 | `0.5, 0.3` | rain coverage formula (rand*0.5 + 0.3 = 0.3..0.8) | `WaterConfig` | `rain_coverage_min`, `rain_coverage_max` |
| Line 271 | `4` | depth to start drowning | (same as `dangerous_threshold`) | - |
| Line 274-279 | `1,3,10,30,999` | drown thresholds by depth | `WaterConfig` | `drown_threshold_7`, `drown_threshold_6`, `drown_threshold_5`, `drown_threshold_4` |
| Line 321 | `2` | flee flood threshold | `WaterConfig` | `flee_flood_depth` |

### File: src/systems/hazard.rs

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 38-39 | `2, 0` | dense soil stability bonus | `HazardConfig` | `dense_stability_bonus` |
| Line 46-51 | `0,1,3,10,25` | collapse chances by open neighbor count | `HazardConfig` | `collapse_chance_3`, `collapse_chance_4`, `collapse_chance_5`, `collapse_chance_6plus` |

### File: src/colony.rs

**Inline magic numbers:**
| Location | Value | Context | Target Sub-struct | Field Name |
|----------|-------|---------|-------------------|------------|
| Line 31 | `100` | initial food stored per colony | `ColonyConfig` | `initial_food` |

### Values to LEAVE (not centralize)

Per CONTEXT.md scope boundary (purely structural/visual, not behavioral):

| File | Value | Reason |
|------|-------|--------|
| app.rs | `TARGET_FPS=30`, `FRAME_DURATION` | Frame timing |
| app.rs | `200, 100` (terrain dimensions) | Terrain generation |
| app.rs | `8` (spatial grid cell size) | Spatial optimization |
| render.rs | `36` (panel width), all `Color::*` values | Rendering/visual |
| terrain.rs | All noise parameters (`0.02`, `0.05`, `0.7`, etc.) | Terrain generation noise |
| colony.rs | `COLONY_COLORS` array | Rendering/visual |
| water.rs | `movement_penalty()` thresholds | Tightly coupled to depth logic, could go either way -- recommend leaving for now |
| pheromone.rs | `cardinal_weight`, `diagonal_weight` in `diffuse()` | Internal algorithm constants |

## Code Examples

### Complete SimConfig Definition
```rust
// src/config.rs

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
```

### Wiring Through App
```rust
// In app.rs - add to struct
pub struct App {
    // ... existing fields ...
    config: SimConfig,
}

// In App::new()
let config = SimConfig::default();
let colonies = systems::spawn::spawn_colonies(
    &mut world, &terrain, config.spawn.num_colonies,
);
systems::food::spawn_food_sources(
    &mut world, &terrain, config.food.num_food_sources,
);

// In App::update() -- pass &self.config to every system call
systems::combat::combat_system(
    &mut self.world, &mut self.pheromones,
    self.tick, &self.spatial_grid, &self.config,
);
```

### System Function Signature Change Pattern
```rust
// Before (combat.rs):
pub fn combat_system(world: &mut World, pheromones: &mut PheromoneGrid, tick: u64, spatial_grid: &SpatialGrid) {
    if tick % COMBAT_INTERVAL != 0 {

// After:
pub fn combat_system(world: &mut World, pheromones: &mut PheromoneGrid, tick: u64, spatial_grid: &SpatialGrid, config: &SimConfig) {
    if tick % config.combat.combat_interval != 0 {
```

### PheromoneGrid Needs Config for Decay
```rust
// The PheromoneGrid::decay_all() method currently reads module-level constants.
// It needs to accept config parameters:

// Before:
pub fn decay_all(&mut self) {
    for chunk in self.data.chunks_exact_mut(3) {
        chunk[0] *= 1.0 - DECAY_FOOD;

// After:
pub fn decay_all(&mut self, config: &PheromoneConfig) {
    for chunk in self.data.chunks_exact_mut(3) {
        chunk[0] *= 1.0 - config.decay_food;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Scattered `const` declarations | Centralized config struct | This phase | Single source of truth for tuning |
| Compile-time constants | Runtime Default struct | This phase | Enables future hot-reload |
| Magic numbers in expressions | Named config fields | This phase | Self-documenting, findable |

## Open Questions

1. **PheromoneGrid method signatures**
   - What we know: `PheromoneGrid::decay_all()` and `PheromoneGrid::diffuse()` currently use module-level constants directly. They need config values passed in.
   - What's unclear: Should we pass `&PheromoneConfig` to these methods, or `&SimConfig`?
   - Recommendation: Pass `&PheromoneConfig` to PheromoneGrid methods since the struct methods are pheromone-specific. System-level functions get `&SimConfig`.

2. **ColonyState::new() needs config**
   - What we know: `ColonyState::new()` hardcodes `food_stored: 100`. This needs to become `config.colony.initial_food`.
   - What's unclear: Should ColonyState::new take full config or just the initial_food value?
   - Recommendation: Pass just the `initial_food: u32` value to keep the colony module's dependency minimal.

3. **WaterCell methods use inline thresholds**
   - What we know: `WaterCell::is_passable()` uses `6`, `is_dangerous()` uses `4`, `movement_penalty()` uses depth thresholds.
   - What's unclear: These are tightly coupled to the WaterCell struct. Extracting them requires WaterCell methods to take config or be replaced with free functions.
   - Recommendation: Leave WaterCell methods as-is for now. The system-level functions that read these thresholds (drowning, flee) should use config values. Alternatively, convert WaterCell methods to accept config values -- but this creates awkward ergonomics since WaterCell is a simple data struct.

## Sources

### Primary (HIGH confidence)
- Direct codebase audit of all 15 source files in `E:/VS Code Projects/AntTrails/src/`
- Every constant and magic number verified by reading the actual code line-by-line

### Secondary (MEDIUM confidence)
- CONTEXT.md decisions for scope boundary and config design choices
- STATE.md for project history and accumulated decisions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new libraries needed, pure Rust refactoring
- Architecture: HIGH - Pattern is straightforward struct + Default + parameter passing
- Constant inventory: HIGH - Every source file read line-by-line, all values catalogued
- Pitfalls: HIGH - Based on direct analysis of the specific codebase

**Research date:** 2026-02-08
**Valid until:** Indefinite (codebase-specific research, no external dependency concerns)
