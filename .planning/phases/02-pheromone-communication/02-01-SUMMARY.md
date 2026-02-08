---
phase: 02-pheromone-communication
plan: 01
subsystem: simulation
tags: [pheromone, diffusion, gradient, decay, adaptive-deposit, double-buffer]

# Dependency graph
requires:
  - phase: 01-unfreeze-and-activate
    provides: "Wired foraging_movement() -> follow_pheromone() -> get_gradient() call chain"
provides:
  - "Per-type pheromone decay (food 0.02, home 0.005, danger 0.05)"
  - "Adaptive deposit preventing saturation (stabilizes at ~0.71)"
  - "Double-buffer diffusion spreading pheromone to 8 neighbors at 5%/tick"
  - "Proximity-scaled home pheromone deposit (fades beyond 30 tiles)"
  - "Weighted random gradient following (strength^2 probability)"
affects:
  - 02-02 (wiring pheromone systems into game loop)
  - 02-03 (pheromone visualization/rendering)
  - 03-config-centralization (new constants to extract)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Double-buffer swap for spatial diffusion (std::mem::swap)"
    - "Adaptive deposit formula: base * (1.0 - current/max)"
    - "Weighted random selection with strength^2 emphasis"

key-files:
  created: []
  modified:
    - src/systems/pheromone.rs

key-decisions:
  - "Keep old get_gradient() method alongside new get_gradient_weighted() for backward compatibility"
  - "Digging ants deposit home pheromone at half rate within 20 tiles (not 30) for tighter nest marking"
  - "Replaced DEPOSIT_AMOUNT references with new base constants as part of constant migration"

patterns-established:
  - "Adaptive deposit: prevents any single source from saturating a cell"
  - "Proximity scaling: distance-gated pheromone deposit for spatial relevance"

# Metrics
duration: 4min
completed: 2026-02-07
---

# Phase 2 Plan 01: Pheromone System Core Rewrite Summary

**Per-type decay rates, adaptive deposit preventing saturation, double-buffer diffusion, proximity-scaled home deposit, and weighted random gradient following -- all in pheromone.rs**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-08T04:43:05Z
- **Completed:** 2026-02-08T04:47:00Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Replaced broken DECAY_RATE (0.001) and DEPOSIT_AMOUNT (0.05) with per-type constants that prevent saturation
- Added adaptive deposit formula (base * (1.0 - current/max)) so single-ant trails stabilize at ~0.71 instead of 1.0
- Implemented double-buffer diffusion spreading pheromone to 8 neighbors at 5% per tick with no per-tick allocation
- Home pheromone deposit now proximity-scaled: fades to zero beyond 30 tiles from nest
- Weighted random gradient selection replaces greedy "pick strongest neighbor" approach

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite PheromoneGrid struct, constants, decay, and diffusion** - `47ca539` (feat)
2. **Task 2: Add adaptive deposit, proximity-based home deposit, and weighted gradient** - `db5f0b5` (feat)

## Files Created/Modified
- `src/systems/pheromone.rs` - Complete rewrite of pheromone system core: new constants, buffer field, adaptive deposit, per-type decay, diffusion, weighted gradient

## Decisions Made
- Kept old `get_gradient()` method for backward compatibility (combat.rs may reference it); will be retired in future phase
- Digging ants use a tighter 20-tile radius (vs 30 for wandering) at half deposit rate for concentrated nest marking
- Temporarily replaced DEPOSIT_AMOUNT references with new base constants in Task 1 to maintain compilation before Task 2's full rewrite

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed DEPOSIT_AMOUNT reference after constant removal**
- **Found during:** Task 1 (constant replacement)
- **Issue:** Removing DEPOSIT_AMOUNT broke pheromone_deposit_system() which still referenced it
- **Fix:** Replaced with DEPOSIT_HOME_BASE and DEPOSIT_FOOD_BASE as temporary bridge (fully rewritten in Task 2)
- **Files modified:** src/systems/pheromone.rs
- **Verification:** cargo build succeeded
- **Committed in:** 47ca539 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary bridge fix between constant removal and function rewrite. No scope creep.

## Issues Encountered
None beyond the deviation above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Pheromone math foundation is correct and ready for wiring
- Plan 02-02 must update app.rs call site to pass colonies parameter (currently produces expected compilation error)
- All new methods (deposit_adaptive, get_gradient_weighted, diffuse) are ready to be called from the game loop

---
*Phase: 02-pheromone-communication*
*Completed: 2026-02-07*
