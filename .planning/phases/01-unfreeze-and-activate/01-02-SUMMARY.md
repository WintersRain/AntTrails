---
phase: 01-unfreeze-and-activate
plan: 02
subsystem: movement
tags: [probability-tuning, state-transitions, activity-rates, idle-wandering, dig-ai]

# Dependency graph
requires:
  - phase: 01-01
    provides: "Wired all AntState variants to movement functions -- ants can now move in all states"
provides:
  - "Idle-to-Wandering transition at ~35% per tick, owned by movement.rs"
  - "Reduced dig.rs Idle-to-Wandering to ~2% (no longer competing with movement.rs)"
  - "Reduced Wandering-to-Digging from ~70% to ~20% so ants wander ~5 ticks before digging"
  - "Majority of worker ants visibly active at any given tick"
affects: [01-03, 02-pheromone-communication, 03-config-centralization, 04-utility-ai-core]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Single-owner state transitions: each major state transition owned by one system to prevent competing probabilities"
    - "Probability tuning via fastrand::u8(..) thresholds against 256 range"

key-files:
  created: []
  modified:
    - "src/systems/movement.rs"
    - "src/systems/dig.rs"

key-decisions:
  - "Idle-to-Wandering ownership consolidated to movement.rs (~35%), with dig.rs retaining only ~2% as a non-competing fallback"
  - "Wandering-to-Digging reduced to ~20% so ants wander long enough to discover food and follow pheromone trails"
  - "Kept dig.rs Idle-to-Wandering at 5 rather than 0 to preserve the code path as a safe no-op"

patterns-established:
  - "Single-owner transitions: when two systems can trigger the same state change, one system is primary (~35%) and the other is vestigial (~2%)"
  - "Probability documentation: comments on threshold lines document the intended percentage and behavior"

# Metrics
duration: 1min
completed: 2026-02-07
---

# Phase 1 Plan 2: Activity Probability Tuning Summary

**Tuned Idle-to-Wandering to ~35% (owned by movement.rs) and Wandering-to-Digging to ~20%, making majority of ants visibly active at any tick**

## Performance

- **Duration:** 1 min
- **Started:** 2026-02-07T05:23:28Z
- **Completed:** 2026-02-07T05:24:52Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Increased Idle-to-Wandering probability from 3.9% to 35.2% in movement.rs (threshold 10 -> 90), making idle ants start moving within 2-3 ticks on average
- Reduced Idle-to-Wandering in dig.rs from 11.7% to 2.0% (threshold 30 -> 5), consolidating ownership of this transition to movement.rs
- Reduced Wandering-to-Digging in dig.rs from 70.3% to 19.5% (threshold 180 -> 50), so ants wander ~5 ticks before digging -- enough time for foraging system to detect food and steer ants toward pheromone trails

## Task Commits

Each task was committed atomically:

1. **Task 1: Increase Idle-to-Wandering probability in movement.rs and reduce in dig.rs** - `ea3f662` (feat)

## Files Created/Modified
- `src/systems/movement.rs` - Idle threshold changed from 10 to 90 (~35% chance to start wandering per tick)
- `src/systems/dig.rs` - Idle threshold reduced from 30 to 5 (~2%); Wandering-to-Digging threshold reduced from 180 to 50 (~20%)

## Decisions Made
- Consolidated Idle-to-Wandering ownership to movement.rs at ~35% rather than splitting evenly between two systems -- single-owner prevents probability stacking and makes tuning predictable
- Kept dig.rs Idle-to-Wandering at 5 (not 0) to preserve the code path without meaningfully competing
- Set Wandering-to-Digging at 50/256 (~19.5%) for ~5 tick average wander time, balancing surface visibility with eventual digging

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Activity rates tuned -- ants are now visibly active on the surface, ready for Plan 01-03 (spatial hashing for neighbor lookups)
- Wandering duration of ~5 ticks gives foraging system time to steer ants toward food, which will be more effective after Phase 2 (pheromone tuning)
- These thresholds are magic numbers that Phase 3 (config centralization) will extract into a central config

---
*Phase: 01-unfreeze-and-activate*
*Completed: 2026-02-07*
