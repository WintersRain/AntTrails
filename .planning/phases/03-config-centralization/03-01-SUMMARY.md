---
phase: 03-config-centralization
plan: 01
subsystem: config
tags: [rust, config-struct, refactoring, default-impl]

# Dependency graph
requires:
  - phase: 02-pheromone-communication
    provides: Pheromone system with constants to centralize
provides:
  - SimConfig struct with 9 nested sub-structs and Default impls
  - Config wiring through App into all 19 system function signatures
  - 4 app.rs spawn constants and 3 tick-interval constants replaced with config fields
affects: [03-02 (replace pheromone/combat/lifecycle constants), 03-03 (replace remaining constants)]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Nested config struct with Default for centralized tuning", "Pass &SimConfig as last param to all system functions", "Underscore prefix _config for unused params during incremental migration"]

key-files:
  created: [src/config.rs]
  modified: [src/main.rs, src/app.rs, src/systems/dig.rs, src/systems/combat.rs, src/systems/movement.rs, src/systems/food.rs, src/systems/aphid.rs, src/systems/pheromone.rs, src/systems/lifecycle.rs, src/systems/hazard.rs, src/systems/water.rs]

key-decisions:
  - "Pass full &SimConfig to system functions (not sub-struct references) to avoid friction when systems need multiple sub-configs"
  - "Use _config underscore prefix during Plan 01 to suppress unused warnings; Plans 02/03 will rename to config as they replace constants"
  - "PheromoneGrid::diffuse gets &PheromoneConfig (sub-struct) since it's a method on a pheromone-specific type"

patterns-established:
  - "Config plumbing pattern: App owns SimConfig, passes &self.config to every system call"
  - "All behavioral constants have a home in config sub-structs; structural/visual constants (FPS, terrain size, colors) stay local"

# Metrics
duration: 6min
completed: 2026-02-08
---

# Phase 3 Plan 01: Config Centralization - Struct and Wiring Summary

**SimConfig with 9 sub-structs (90 total fields) created in config.rs, wired through App into all 19 system functions with 7 app.rs constants already replaced**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-09T02:35:12Z
- **Completed:** 2026-02-09T02:41:00Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- Created src/config.rs with complete SimConfig hierarchy: 10 structs, 90 fields, all Default values exactly matching current hardcoded constants
- Wired SimConfig into App struct and replaced 4 spawn constants (NUM_COLONIES, NUM_FOOD_SOURCES, NUM_APHIDS, NUM_WATER_SOURCES) plus 3 tick-interval magic numbers (10, 3, 50) with config field access
- Added _config: &SimConfig parameter to all 18 system functions plus &PheromoneConfig to PheromoneGrid::diffuse, with imports in all 9 system files

## Task Commits

Each task was committed atomically:

1. **Task 1: Create config.rs with SimConfig and all sub-structs** - `5a0f511` (feat)
2. **Task 2: Wire SimConfig into App and update all system call sites** - `6a530da` (feat)

## Files Created/Modified
- `src/config.rs` - NEW: SimConfig + 9 sub-structs (PheromoneConfig, CombatConfig, LifecycleConfig, MovementConfig, FoodConfig, SpawnConfig, ColonyConfig, WaterConfig, HazardConfig) with Default impls
- `src/main.rs` - Added `mod config;` declaration
- `src/app.rs` - Added config field, replaced 7 constants/magic numbers with config access, passes &self.config to all system calls
- `src/systems/dig.rs` - Added _config: &SimConfig to dig_ai_system, dig_system
- `src/systems/combat.rs` - Added _config: &SimConfig to combat_system, soldier_ai_system, flee_system
- `src/systems/movement.rs` - Added _config: &SimConfig to movement_system
- `src/systems/food.rs` - Added _config: &SimConfig to foraging_system, check_deposit, food_regrow_system
- `src/systems/aphid.rs` - Added _config: &SimConfig to aphid_system
- `src/systems/pheromone.rs` - Added _config: &SimConfig to pheromone_decay_system, pheromone_deposit_system; _config: &PheromoneConfig to PheromoneGrid::diffuse
- `src/systems/lifecycle.rs` - Added _config: &SimConfig to lifecycle_system
- `src/systems/hazard.rs` - Added _config: &SimConfig to cave_in_system
- `src/systems/water.rs` - Added _config: &SimConfig to evaporation_system, rain_system, drowning_system, flee_flood_system

## Decisions Made
- [03-01]: Pass full &SimConfig to system functions (not sub-struct references) to avoid friction when systems need multiple sub-configs
- [03-01]: Use _config underscore prefix during Plan 01 to suppress unused warnings; Plans 02/03 will rename to config as they consume fields
- [03-01]: PheromoneGrid::diffuse gets &PheromoneConfig (sub-struct) since it's a method on a pheromone-specific type, not a free system function

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Config plumbing complete: all system functions accept &SimConfig but still use their own local constants
- Plan 02 can begin immediately: rename _config to config in pheromone/combat/lifecycle systems and replace their const/magic-number usage with config field access
- Plan 03 follows to replace remaining systems (movement, food, spawn, aphid, water, hazard, colony)
- All 90 config fields have Default values matching current behavior exactly -- no behavioral change from this plan

---
*Phase: 03-config-centralization*
*Plan: 01*
*Completed: 2026-02-08*
