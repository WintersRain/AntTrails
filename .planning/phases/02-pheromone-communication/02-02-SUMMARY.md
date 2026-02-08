---
phase: 02-pheromone-communication
plan: 02
subsystem: simulation
tags: [pheromone, decay, diffusion, gradient, foraging, game-loop]

# Dependency graph
requires:
  - phase: 02-01
    provides: "Rewritten PheromoneGrid with adaptive deposit, per-type decay, diffusion method"
  - phase: 01-01
    provides: "foraging_movement() wired into movement system with follow_pheromone()"
provides:
  - "Correct pheromone system call order: decay -> diffuse -> deposit (every tick)"
  - "pheromone_deposit_system receives colonies for proximity-based home deposit"
  - "Lowered foraging detection threshold (0.01) for diffused gradient response"
  - "Compile error from 02-01 resolved (arg count mismatch)"
affects: [02-03-pheromone-visualization, 03-config-centralization]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "System call ordering: decay -> diffuse -> deposit for correct gradient formation"
    - "Per-tick pheromone processing (no frame-gating) for smooth gradients"

key-files:
  created: []
  modified:
    - src/app.rs
    - src/systems/food.rs

key-decisions:
  - "Decay runs every tick (not gated by tick%10) since per-type rates already account for per-tick execution"
  - "Detection threshold 0.01 allows ants to sense diffusion fringes 3-5 tiles from trail center"

patterns-established:
  - "Pheromone pipeline order: decay -> diffuse -> deposit in game loop"

# Metrics
duration: 2min
completed: 2026-02-07
---

# Phase 2 Plan 2: Pheromone System Wiring Summary

**Wired rewritten pheromone pipeline into game loop with decay-diffuse-deposit ordering and lowered foraging threshold for gradient-fringe detection**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-08T04:52:03Z
- **Completed:** 2026-02-08T04:54:06Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Fixed compile error from 02-01 by passing `&self.colonies` to `pheromone_deposit_system`
- Established correct system call order: decay -> diffuse -> deposit (replacing deposit-then-conditional-decay)
- Removed tick%10 gate on pheromone decay -- now runs every tick with the higher per-type rates from 02-01
- Added `self.pheromones.diffuse()` call between decay and deposit for spatial gradient spreading
- Lowered foraging pheromone detection threshold from 0.1 to 0.01 so ants detect diffusion fringes

## Task Commits

Each task was committed atomically:

1. **Task 1: Update app.rs pheromone system call order** - `25bbbe6` (feat)
2. **Task 2: Lower foraging pheromone detection threshold in food.rs** - `4683cb9` (feat)

## Files Created/Modified
- `src/app.rs` - Updated Phase 4 pheromone section: decay -> diffuse -> deposit order, per-tick execution, colonies arg
- `src/systems/food.rs` - Lowered foraging detection threshold from 0.1 to 0.01 in Wandering arm

## Decisions Made
- Decay runs every tick (not tick%10) since per-type rates (0.02 food, 0.005 home, 0.05 alarm) already account for per-tick execution
- Detection threshold of 0.01 chosen to match diffusion fringe values (single-ant trails equilibrate at ~0.71, diffused fringe 3-5 tiles out is 0.01-0.1)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full pheromone pipeline is active: decay -> diffuse -> deposit runs every tick
- Gradients form correctly but are not yet visible to the user
- Ready for Plan 02-03: Pheromone visualization overlay
- No blockers or concerns

---
*Phase: 02-pheromone-communication*
*Completed: 2026-02-07*
