---
phase: 01-unfreeze-and-activate
plan: 03
subsystem: simulation-core
tags: [spatial-hash, combat, performance, hecs, ecs]

# Dependency graph
requires:
  - phase: 01-unfreeze-and-activate/01-01
    provides: "Working movement system with all AntState variants wired"
provides:
  - "SpatialGrid module for O(1) neighbor lookups (src/spatial.rs)"
  - "Combat system using spatial queries instead of O(N^2) nested loop"
  - "Per-tick spatial grid rebuild in App::update()"
affects:
  - "04-utility-ai-core (SenseData perception layer will reuse SpatialGrid for proximity queries)"
  - "Any future system needing neighbor lookups (pheromone influence, food detection, etc.)"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Spatial hash grid: cell_size=8, rebuilt per tick, query returns owned Vec"
    - "Pair deduplication: canonical (min, max) entity ordering to avoid double-counting"

key-files:
  created:
    - "src/spatial.rs"
  modified:
    - "src/main.rs"
    - "src/app.rs"
    - "src/systems/combat.rs"

key-decisions:
  - "Cell size 8 for 200x100 map creates 25x13=325 cells, averaging ~1.5 ants per cell at 500 ants"
  - "query_nearby returns owned Vec (not iterator) to avoid lifetime issues with ECS mutation patterns"
  - "Used Vec for processed_pairs instead of HashSet -- linear scan is fine for expected <5 combatant pairs per tick"

patterns-established:
  - "Spatial grid rebuild pattern: clear() then insert() all entities at start of each tick in App::update()"
  - "Shared spatial infrastructure: any system can accept &SpatialGrid for neighbor lookups without rebuilding"

# Metrics
duration: 4min
completed: 2026-02-07
---

# Phase 1 Plan 3: Spatial Hash Grid Summary

**Spatial hash grid (cell_size=8, 325 cells) replaces O(N^2) combat loop with O(N*K) spatial queries, shared infrastructure for future neighbor lookups**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-07T05:23:32Z
- **Completed:** 2026-02-07T05:27:12Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Created SpatialGrid module with new/clear/insert/query_nearby API, partitioning 200x100 map into 25x13 cells
- Integrated spatial grid into App lifecycle: created once, rebuilt every tick with all ant positions
- Replaced O(N^2) nested combat loop with spatial grid lookups -- from 250,000 comparisons to ~14 per ant at 500 ants
- Added pair deduplication to prevent double-counting when both entities in a pair find each other via query_nearby

## Task Commits

Each task was committed atomically:

1. **Task 1: Create SpatialGrid module** - `3d8bc6b` (feat)
2. **Task 2: Integrate SpatialGrid into App and replace O(N^2) combat loop** - `8db874f` (feat)

## Files Created/Modified
- `src/spatial.rs` - SpatialGrid struct with insert, clear, query_nearby for O(1) neighbor lookups
- `src/main.rs` - Added `mod spatial;` declaration
- `src/app.rs` - SpatialGrid field in App, initialized in new(), rebuilt per tick, passed to combat_system
- `src/systems/combat.rs` - combat_system uses SpatialGrid for neighbor lookups instead of O(N^2) loop

## Decisions Made
- Cell size 8: Creates 325 cells for 200x100 map. At 500 ants, averages 1.5 per cell. Checking 9 cells per query = ~14 entity comparisons vs O(N)=500
- Owned Vec return from query_nearby: Simpler than iterators, avoids lifetime issues with ECS mutation patterns, Vec is small (~14 entries typical)
- Vec for processed_pairs: Linear scan is adequate for expected density (<5 actual combat pairs per tick). HashSet would be premature optimization
- Pair deduplication via canonical (min, max) entity ordering using hecs::Entity's Ord trait

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 1 (Unfreeze & Activate) is now complete: all 3 plans (01-01, 01-02, 01-03) are done
- SpatialGrid is shared infrastructure ready for Phase 4 (Utility AI SenseData perception layer)
- Phase 2 (Pheromone Communication) can proceed -- it focuses on pheromone balance and gradient following

---
*Phase: 01-unfreeze-and-activate*
*Completed: 2026-02-07*
