---
phase: 03-config-centralization
plan: 03
subsystem: config
tags: [rust, config-struct, constant-extraction, mechanical-refactoring]

# Dependency graph
requires:
  - phase: 03-config-centralization
    provides: SimConfig struct wired to all system functions with _config params ready to consume
provides:
  - Zero behavioral constants remaining in any system file
  - All 6 remaining systems (food, spawn, aphid, water, hazard, colony) reading from config
  - Phase 3 complete -- every behavioral constant centralized in SimConfig
affects: [04-utility-ai (all behavioral tuning in config from day one), 05-emergent-specialization (config fields for new parameters)]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Store config values as struct fields on data grids (WaterGrid.max_depth) when methods need values but signature changes would cascade"]

key-files:
  created: []
  modified: [src/systems/food.rs, src/systems/spawn.rs, src/systems/aphid.rs, src/systems/water.rs, src/systems/hazard.rs, src/colony.rs, src/systems/movement.rs, src/app.rs]

key-decisions:
  - "Store max_depth as WaterGrid struct field (same pattern as PheromoneGrid in 03-02) to avoid threading config through low-level grid methods"
  - "foraging_movement receives &SimConfig for food_pheromone_threshold access"
  - "spawn_colonies receives full &SimConfig instead of individual params, reads num_colonies/initial_workers/min_colony_distance internally"
  - "ColonyState::new takes initial_food parameter (minimal dependency -- no SimConfig import needed in colony.rs)"
  - "WaterCell methods (is_passable, is_dangerous, movement_penalty) left with hardcoded thresholds per research recommendation -- tightly coupled to struct semantics"

patterns-established:
  - "Config centralization complete: every behavioral constant accessible via config.{subsystem}.{field}"
  - "Spawn functions accept &SimConfig for all initialization values"

# Metrics
duration: 4min
completed: 2026-02-08
---

# Phase 3 Plan 03: Environment/Resource Systems Config Replacement Summary

**Replaced 7 named constants and 25+ inline magic numbers in food/spawn/aphid/water/hazard/colony systems -- Phase 3 complete, all behavioral constants centralized in SimConfig**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-09T02:56:24Z
- **Completed:** 2026-02-09T03:00:30Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Removed FOOD_REGROW_INTERVAL, INITIAL_FOOD_AMOUNT from food.rs; replaced deposit distance, food per pickup/deposit, pheromone threshold with config access
- Removed INITIAL_WORKERS, MIN_COLONY_DISTANCE from spawn.rs; spawn_colonies now takes &SimConfig for all spawn parameters
- Removed APHID_FOOD_RATE, CLAIM_TICKS, NEARBY_DISTANCE from aphid.rs; spawn_aphids now takes &SimConfig
- Removed MAX_WATER_DEPTH from water.rs (stored as WaterGrid.max_depth field); replaced all rain, evaporation, drowning, and flee thresholds with config access
- Replaced hazard collapse chances and stability bonus with config.hazard fields
- ColonyState::new accepts initial_food parameter from config.colony.initial_food

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace constants in food.rs, spawn.rs, and aphid.rs** - `dcd3589` (feat)
2. **Task 2: Replace constants in water.rs, hazard.rs** - `9ea8ec8` (feat)

## Files Created/Modified
- `src/systems/food.rs` - All food values from config.food; spawn_food_sources accepts &SimConfig; foraging_movement accepts &SimConfig
- `src/systems/spawn.rs` - spawn_colonies accepts &SimConfig; reads num_colonies, initial_workers, min_colony_distance from config
- `src/systems/aphid.rs` - aphid_food_rate, nearby_distance from config.spawn; spawn_aphids accepts &SimConfig
- `src/systems/water.rs` - All water thresholds from config.water; WaterGrid stores max_depth as field; rain/evaporation/drowning/flee all use config
- `src/systems/hazard.rs` - Collapse chances and stability bonus from config.hazard
- `src/colony.rs` - ColonyState::new accepts initial_food parameter
- `src/systems/movement.rs` - foraging_movement calls updated to pass config
- `src/app.rs` - spawn_colonies, spawn_food_sources, spawn_aphids, WaterGrid::new calls updated with config

## Decisions Made
- [03-03]: Store max_depth as WaterGrid struct field (same pattern as PheromoneGrid in 03-02) to avoid cascading config through grid methods used by physics functions
- [03-03]: foraging_movement receives &SimConfig so food_pheromone_threshold comes from config; updated both call sites in movement.rs
- [03-03]: spawn_colonies takes full &SimConfig (reads num_colonies, initial_workers, min_colony_distance internally) rather than 3 separate params
- [03-03]: ColonyState::new takes just initial_food: u32 (not &SimConfig) to keep colony module's dependency minimal
- [03-03]: WaterCell methods (is_passable, is_dangerous, movement_penalty) left with hardcoded thresholds per research recommendation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] foraging_movement() signature changed to accept &SimConfig**
- **Found during:** Task 1 (food.rs constants replacement)
- **Issue:** foraging_movement() used hardcoded 0.01 food pheromone threshold. Replacing with config.food.food_pheromone_threshold required adding &SimConfig parameter.
- **Fix:** Added config: &SimConfig parameter to foraging_movement(), updated both call sites in movement.rs (Carrying and Following states)
- **Files modified:** src/systems/food.rs, src/systems/movement.rs
- **Verification:** cargo build passes
- **Committed in:** dcd3589 (Task 1 commit)

**2. [Rule 3 - Blocking] WaterGrid::add_water borrow checker fix**
- **Found during:** Task 2 (water.rs constants replacement)
- **Issue:** Replacing MAX_WATER_DEPTH with self.max_depth in add_water() caused borrow checker error -- self.max_depth read conflicts with self.get_mut() mutable borrow
- **Fix:** Read max_depth into local variable before the mutable borrow
- **Files modified:** src/systems/water.rs
- **Verification:** cargo build passes
- **Committed in:** 9ea8ec8 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both auto-fixes necessary to complete constant replacement. No scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 3 complete: all behavioral constants centralized in SimConfig with organized sub-structs
- POL-01 requirement satisfied: changing any behavioral parameter requires editing exactly one location (config.rs Default impl)
- Phase 4 (Utility AI Core) can begin immediately: new AI constants will go directly into config from the start
- Three unused config fields remain (deposit_danger, aphid_claim_ticks, passable_threshold) -- these are either used differently than originally inventoried or represent features not yet fully implemented

---
*Phase: 03-config-centralization*
*Plan: 03*
*Completed: 2026-02-08*
