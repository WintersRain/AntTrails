---
phase: 03-config-centralization
plan: 02
subsystem: config
tags: [rust, config-struct, constant-extraction, mechanical-refactoring]

# Dependency graph
requires:
  - phase: 03-config-centralization
    provides: SimConfig struct with 9 sub-structs and &SimConfig wiring to all system functions
provides:
  - Zero behavioral constants remaining in pheromone.rs, combat.rs, lifecycle.rs, movement.rs, dig.rs
  - All 5 core behavior systems reading from config sub-structs
  - PheromoneGrid stores max_strength and gradient_threshold as fields (set from config at construction)
affects: [03-03 (remaining system files still have local constants), 04-utility-ai (all behavioral tuning now in config)]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Store config values as struct fields when method signature changes would cascade across module boundaries"]

key-files:
  created: []
  modified: [src/systems/pheromone.rs, src/systems/combat.rs, src/systems/lifecycle.rs, src/systems/movement.rs, src/systems/dig.rs, src/app.rs]

key-decisions:
  - "Store max_strength and gradient_threshold as PheromoneGrid fields instead of changing deposit/get_gradient_weighted signatures (avoids cascading to food.rs which is out of scope)"
  - "fleeing_movement() now accepts &SimConfig for max_colonies_scan access"
  - "ensure_queen_ages() now accepts &SimConfig for queen_lifespan access"

patterns-established:
  - "Config field access pattern: system functions use config.{subsystem}.{field} for all behavioral values"
  - "PheromoneGrid fields pattern: values needed by methods (deposit, get_gradient_weighted) stored as struct fields set from config at construction"

# Metrics
duration: 3min
completed: 2026-02-08
---

# Phase 3 Plan 02: Core System Constants Replacement Summary

**Removed 22 named constants and 20 inline magic numbers from 5 core behavior systems, all now reading from config.pheromone/combat/lifecycle/movement sub-structs**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-09T02:44:27Z
- **Completed:** 2026-02-09T02:47:30Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Removed all 10 named constants from pheromone.rs (MAX_PHEROMONE, DECAY_FOOD/HOME/DANGER, SNAP_TO_ZERO, DEPOSIT_FOOD/HOME/DANGER_BASE, DIFFUSION_RATE, HOME_DEPOSIT_RADIUS) plus 3 inline magic numbers (0.01, 20.0, 0.5)
- Removed 2 named constants from combat.rs (BASE_DAMAGE, COMBAT_INTERVAL) plus 11 inline magic numbers (soldier/worker/other strength, danger deposit, damage range, default health/strength, fight/flee thresholds, max colonies scan)
- Removed all 10 named constants from lifecycle.rs plus 1 inline magic number (204 worker ratio threshold)
- Replaced 2 inline magic numbers in movement.rs (queen_move_threshold 5, idle_move_threshold 90)
- Removed 2 named constants from dig.rs (DIG_CHANCE, REINFORCE_CHANCE) plus 5 inline magic numbers (start_dig_chance, underground/surface return chance, dig distraction, idle-to-wander)

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace constants in pheromone.rs and combat.rs** - `6a7cbe5` (feat)
2. **Task 2: Replace constants in lifecycle.rs, movement.rs, and dig.rs** - `f7161ac` (feat)

## Files Created/Modified
- `src/systems/pheromone.rs` - All pheromone values from config; PheromoneGrid stores max_strength and gradient_threshold as fields
- `src/systems/combat.rs` - All combat values from config.combat; calculate_damage and apply_damage accept &SimConfig
- `src/systems/lifecycle.rs` - All lifecycle timers and food costs from config.lifecycle; ensure_queen_ages accepts &SimConfig
- `src/systems/movement.rs` - Queen and idle movement thresholds from config.movement; fleeing_movement call updated with config
- `src/systems/dig.rs` - All dig probabilities from config.movement; reinforce_adjacent and decide_worker_state accept &SimConfig
- `src/app.rs` - PheromoneGrid::new call updated with &config.pheromone; ensure_queen_ages call updated with &config

## Decisions Made
- [03-02]: Store max_strength and gradient_threshold as PheromoneGrid struct fields (set from &PheromoneConfig at construction) rather than changing deposit()/get_gradient_weighted() signatures, which would cascade to food.rs (out of scope for this plan)
- [03-02]: fleeing_movement() receives &SimConfig to access max_colonies_scan; fighting_movement() unchanged (uses get_gradient which doesn't need config)
- [03-02]: ensure_queen_ages() receives &SimConfig so queen_lifespan comes from config instead of module-level constant

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] PheromoneGrid::new signature changed to accept &PheromoneConfig**
- **Found during:** Task 1 (pheromone.rs constants replacement)
- **Issue:** MAX_PHEROMONE and gradient_threshold (0.01) were used in PheromoneGrid methods (deposit, deposit_adaptive, get_gradient_weighted) called from multiple files. Changing those method signatures would cascade to food.rs (out of scope).
- **Fix:** Added max_strength and gradient_threshold as fields on PheromoneGrid, set from &PheromoneConfig at construction. Updated PheromoneGrid::new signature and app.rs call site.
- **Files modified:** src/systems/pheromone.rs, src/app.rs
- **Verification:** cargo build passes, all existing callers of deposit/get_gradient_weighted work unchanged
- **Committed in:** 6a7cbe5 (Task 1 commit)

**2. [Rule 3 - Blocking] fleeing_movement() signature changed to accept &SimConfig**
- **Found during:** Task 1 (combat.rs constants replacement)
- **Issue:** fleeing_movement() used hardcoded `6` for max colonies scan loop. Replacing with config.combat.max_colonies_scan required adding &SimConfig parameter.
- **Fix:** Added config: &SimConfig parameter to fleeing_movement(), updated call site in movement.rs
- **Files modified:** src/systems/combat.rs, src/systems/movement.rs
- **Verification:** cargo build passes
- **Committed in:** 6a7cbe5 (Task 1 commit)

**3. [Rule 3 - Blocking] ensure_queen_ages() signature changed to accept &SimConfig**
- **Found during:** Task 2 (lifecycle.rs constants replacement)
- **Issue:** ensure_queen_ages() used QUEEN_LIFESPAN constant directly. After removing it, needed config access.
- **Fix:** Added config: &SimConfig parameter, updated call site in app.rs
- **Files modified:** src/systems/lifecycle.rs, src/app.rs
- **Verification:** cargo build passes
- **Committed in:** f7161ac (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (3 blocking)
**Impact on plan:** All auto-fixes necessary to complete constant replacement without cascading to out-of-scope files. No scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- All 5 core behavior systems (pheromone, combat, lifecycle, movement, dig) now read from config exclusively
- Plan 03 can begin immediately to replace remaining constants in food, spawn, aphid, water, hazard, and colony systems
- Remaining dead_code warnings on config fields are expected until Plan 03 consumes them

---
*Phase: 03-config-centralization*
*Plan: 02*
*Completed: 2026-02-08*
