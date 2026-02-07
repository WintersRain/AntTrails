---
phase: 01-unfreeze-and-activate
plan: 01
subsystem: movement
tags: [ecs, hecs, movement, state-machine, foraging, combat, pheromones]

# Dependency graph
requires:
  - phase: none
    provides: "First plan -- no prior dependencies"
provides:
  - "Expanded movement_system with explicit match arms for all 8 AntState variants"
  - "Carrying ants navigate toward colony home via foraging_movement()"
  - "Fighting ants pursue danger pheromones via fighting_movement()"
  - "Fleeing ants escape danger via fleeing_movement()"
  - "Following ants track food pheromone trails via foraging_movement()"
  - "No wildcard match arms remain -- Rust exhaustive checking guards future enum additions"
affects: [01-02, 01-03, 02-pheromone-communication, 04-utility-ai-core]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Explicit match arms over wildcards for AntState dispatch"
    - "Option-returning movement functions with random_movement() fallback"
    - "Domain-specific movement delegation (food.rs, combat.rs) from central movement_system"

key-files:
  created: []
  modified:
    - "src/systems/movement.rs"
    - "src/app.rs"
    - "src/systems/food.rs"
    - "src/systems/combat.rs"

key-decisions:
  - "All None returns from delegated movement functions fall back to random_movement(), not (0,0) -- confused ants wander randomly rather than freezing"
  - "Following ants reuse foraging_movement() since it follows food pheromone gradients, which is the desired Following behavior"

patterns-established:
  - "Movement delegation: movement_system dispatches to domain-specific functions in food.rs and combat.rs based on AntState"
  - "No-freeze convention: every code path must produce movement or explicit random wandering, never (0,0) as default"

# Metrics
duration: 2min
completed: 2026-02-07
---

# Phase 1 Plan 1: Wire Orphaned Movement Functions Summary

**Eliminated the wildcard `_ => (0,0)` freeze in movement_system and wired all 8 AntState variants to domain-specific movement functions from food.rs and combat.rs**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-07T05:18:39Z
- **Completed:** 2026-02-07T05:20:38Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Wired Carrying and Following ants to `food::foraging_movement()` so they navigate toward colony home / follow food pheromone trails
- Wired Fighting ants to `combat::fighting_movement()` so they pursue danger pheromone gradients
- Wired Fleeing ants to `combat::fleeing_movement()` so they move away from danger sources
- Eliminated the wildcard `_ => (0, 0)` match arm that froze ants in active behavioral states
- Removed blanket `#![allow(dead_code)]` from food.rs and combat.rs now that functions are called

## Task Commits

Each task was committed atomically:

1. **Task 1: Expand movement_system signature and wire all AntState match arms** - `ce0a600` (feat)
2. **Task 2: Remove dead_code suppression from food.rs and combat.rs** - `93e7db6` (chore)

## Files Created/Modified
- `src/systems/movement.rs` - Expanded movement_system with 8 explicit AntState match arms, new signature accepting pheromones and colonies
- `src/app.rs` - Updated movement_system call site with pheromones and colonies arguments
- `src/systems/food.rs` - Removed `#![allow(dead_code)]` blanket suppression
- `src/systems/combat.rs` - Removed `#![allow(dead_code)]` blanket suppression

## Decisions Made
- All `None` returns from delegated movement functions fall back to `random_movement()`, not `(0, 0)` -- a confused ant should wander randomly rather than freeze in place, matching real ant behavior
- Following ants reuse `foraging_movement()` because it follows food pheromone gradients when not in Carrying state, which is exactly what Following behavior should do

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Movement system now correctly dispatches all behavioral states to their respective movement functions
- Plan 01-02 (activity tuning) can proceed -- ants now move in all states, so activity rates can be observed and tuned
- Plan 01-03 (spatial hashing) can proceed -- movement patterns are now realistic enough to benefit from neighbor lookups
- Pheromone gradient following is active but pheromone saturation (FIX-03) may still limit observable trail behavior until Phase 2

---
*Phase: 01-unfreeze-and-activate*
*Completed: 2026-02-07*
