# Phase 3: Config Centralization - Context

**Gathered:** 2026-02-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Gather all scattered magic constants (37 named + 60+ hardcoded across 11 files) into a single tunable config struct so that tweaking ant behavior requires editing one location, not hunting across the codebase. No new behavior -- same simulation, same values, just organized.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion

User explicitly deferred all implementation decisions to Claude. The following are Claude's recommended approaches:

**Scope boundary:**
- Centralize all behavioral/simulation constants (pheromone rates, combat values, lifecycle timers, movement probabilities, food amounts, spawn counts)
- Leave hardcoded: render constants (panel width, colors), terrain generation noise parameters, spatial grid cell size, frame timing
- Heuristic: if changing the value would change ant behavior, centralize it. If it's purely structural/visual, leave it.

**Config structure:**
- Single `SimConfig` struct with nested per-system sub-structs
- Organized by system: `config.pheromone.decay_food`, `config.combat.base_damage`, `config.lifecycle.worker_lifespan`, etc.
- Sub-structs: PheromoneConfig, CombatConfig, LifecycleConfig, MovementConfig, FoodConfig, SpawnConfig, ColonyConfig, WaterConfig, HazardConfig
- Each sub-struct derives Default with current hardcoded values

**Runtime access:**
- Runtime struct (not compile-time const) to enable future hot-reload potential
- Passed as `&SimConfig` parameter to systems that need it (not global static, not ECS resource)
- Stored as field on App struct, passed down to system functions
- Phase 4+ systems will receive config from the start

**Defaults & format:**
- Hardcoded Rust defaults only (impl Default for SimConfig and all sub-structs)
- No file loading (TOML/JSON) in this phase -- keep it simple
- Current values become the defaults, so behavior is unchanged
- File-based config loading could be a future enhancement

</decisions>

<specifics>
## Specific Ideas

No specific requirements -- open to standard approaches. User described this as a "vibe coded project" and wants Claude to make sensible engineering decisions throughout.

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>

---

*Phase: 03-config-centralization*
*Context gathered: 2026-02-08*
