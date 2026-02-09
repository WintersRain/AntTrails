# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-05)

**Core value:** Emergent behavior -- ants do things the developer didn't explicitly program. The simulation surprises its creator.
**Current focus:** Phase 3 in progress: Config Centralization (1/3 plans done).

## Current Position

Phase: 3 of 7 (Config Centralization)
Plan: 1 of 3 in current phase
Status: In progress
Last activity: 2026-02-08 -- Completed 03-01-PLAN.md (SimConfig struct and wiring)

Progress: [#######.....................] 29% (7/24 plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 7
- Average duration: 2.7min
- Total execution time: 0.32 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-unfreeze-and-activate | 3/3 | 7min | 2.3min |
| 02-pheromone-communication | 3/3 | 8min | 2.7min |
| 03-config-centralization | 1/3 | 6min | 6.0min |

**Recent Trend:**
- Last 5 plans: 02-01 (4min), 02-02 (2min), 02-03 (2min), 03-01 (6min)
- Trend: 03-01 longer due to touching 11 files across entire codebase

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
- [02-01]: Keep old get_gradient() alongside new get_gradient_weighted() for backward compatibility with combat.rs
- [02-01]: Digging ants use tighter 20-tile radius at half deposit rate for concentrated nest marking
- [02-01]: Adaptive deposit formula: base * (1.0 - current/MAX_PHEROMONE) prevents single-source saturation
- [02-02]: Decay runs every tick (not tick%10) since per-type rates already account for per-tick execution
- [02-02]: Detection threshold 0.01 allows ants to sense diffusion fringes 3-5 tiles from trail center
- [02-03]: Default show_pheromones to true so user sees trails immediately -- core visual feedback of Phase 2
- [02-03]: Pheromone RGB cap at 120 (not 255) preserves foreground terrain character readability
- [02-03]: Visual threshold 0.05 (vs 0.01 detection) to avoid noisy clutter from diffusion fringes
- [03-01]: Pass full &SimConfig to system functions (not sub-struct references) to avoid friction when systems need multiple sub-configs
- [03-01]: Use _config underscore prefix during Plan 01 to suppress unused warnings; Plans 02/03 rename to config as they consume fields
- [03-01]: PheromoneGrid::diffuse gets &PheromoneConfig (sub-struct) since it's a method on a pheromone-specific type

### Pending Todos

None.

### Blockers/Concerns

- ~~Critical: `foraging_movement()`, `fighting_movement()`, `fleeing_movement()` exist in source but are never called -- Phase 1 must wire these~~ RESOLVED in 01-01
- ~~Critical: Movement system `_ => (0,0)` wildcard freezes ants in Carrying/Fighting/Following/Fleeing states~~ RESOLVED in 01-01
- ~~Risk: Idle-to-Wandering probability too low (3.9%) and two systems competing for the transition~~ RESOLVED in 01-02
- ~~Performance: O(N^2) combat loop causes frame drops at 500+ ants~~ RESOLVED in 01-03
- ~~Risk: Pheromone deposit 0.05/tick with decay 0.001/tick causes saturation (no gradient)~~ RESOLVED in 02-01 (per-type decay, adaptive deposit)
- ~~Note: app.rs call site for pheromone_deposit_system needs colonies parameter~~ RESOLVED in 02-02
- ~~Note: Current probability thresholds are magic numbers -- Phase 3 (config centralization) will extract them~~ IN PROGRESS: config struct and wiring done in 03-01, constants replacement in 03-02 and 03-03

## Session Continuity

Last session: 2026-02-08
Stopped at: Completed 03-01-PLAN.md (SimConfig struct and wiring)
Resume file: None
