# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-05)

**Core value:** Emergent behavior -- ants do things the developer didn't explicitly program. The simulation surprises its creator.
**Current focus:** Phase 1 complete. Ready for Phase 2: Pheromone Communication

## Current Position

Phase: 1 of 7 (Unfreeze & Activate) -- COMPLETE
Plan: 3 of 3 in current phase
Status: Phase complete
Last activity: 2026-02-07 -- Completed 01-03-PLAN.md (Spatial hash grid for O(1) neighbor lookups)

Progress: [###.......................] 12% (3/24 plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 3
- Average duration: 2.3min
- Total execution time: 0.12 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-unfreeze-and-activate | 3/3 | 7min | 2.3min |

**Recent Trend:**
- Last 5 plans: 01-01 (2min), 01-02 (1min), 01-03 (4min)
- Trend: stable

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: Front-load movement fixes and activity tuning (Phase 1) because user has never seen simulation work -- ants must move before anything else matters
- [Roadmap]: Centralize config (Phase 3) before Utility AI (Phase 4) so new AI constants go into config from the start rather than scattering more magic numbers
- [Roadmap]: Hand-roll all AI systems (no external libraries) per research recommendation -- domain-specific logic, simple scoring, hecs-incompatible crates
- [01-01]: All None returns from delegated movement functions fall back to random_movement(), not (0,0) -- confused ants wander randomly rather than freezing
- [01-01]: Following ants reuse foraging_movement() since it follows food pheromone gradients, which is the desired Following behavior
- [01-02]: Idle-to-Wandering ownership consolidated to movement.rs (~35%), dig.rs retains only ~2% as non-competing fallback
- [01-02]: Wandering-to-Digging reduced to ~20% so ants wander ~5 ticks before digging, enabling food discovery and pheromone trail following
- [01-03]: Cell size 8 for spatial grid creates 325 cells for 200x100 map, averaging ~1.5 ants per cell at 500 ants
- [01-03]: query_nearby returns owned Vec to avoid lifetime issues with ECS mutation patterns
- [01-03]: Vec for processed_pairs (not HashSet) -- linear scan adequate for expected <5 combatant pairs per tick

### Pending Todos

None yet.

### Blockers/Concerns

- ~~Critical: `foraging_movement()`, `fighting_movement()`, `fleeing_movement()` exist in source but are never called -- Phase 1 must wire these~~ RESOLVED in 01-01
- ~~Critical: Movement system `_ => (0,0)` wildcard freezes ants in Carrying/Fighting/Following/Fleeing states~~ RESOLVED in 01-01
- ~~Risk: Idle-to-Wandering probability too low (3.9%) and two systems competing for the transition~~ RESOLVED in 01-02
- ~~Performance: O(N^2) combat loop causes frame drops at 500+ ants~~ RESOLVED in 01-03
- Risk: Pheromone deposit 0.05/tick with decay 0.001/tick causes saturation (no gradient) -- Phase 2 addresses
- Note: Current probability thresholds are magic numbers -- Phase 3 (config centralization) will extract them

## Session Continuity

Last session: 2026-02-07
Stopped at: Completed 01-03-PLAN.md (Phase 1 complete)
Resume file: None
