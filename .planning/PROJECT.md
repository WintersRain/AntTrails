# AntTrails

## What This Is

A terminal-based ant colony ecosystem simulator in Rust where multiple autonomous colonies compete for territory and resources. Hundreds of individual ants make contextual decisions, develop specializations, and communicate through pheromones — producing colony-level intelligence that emerges from simple individual rules, not top-down scripting.

## Core Value

Emergent behavior: ants do things the developer didn't explicitly program. The simulation surprises its creator.

## Requirements

### Validated

- ✓ Procedural terrain generation (Perlin noise, surface/soil/rock/caves) — existing
- ✓ ECS architecture with hecs — existing
- ✓ Terminal rendering with ratatui/crossterm — existing
- ✓ Multiple competing colonies (3 colonies with distinct colors) — existing
- ✓ Ant lifecycle (queen → egg → larva → worker/soldier, aging, death) — existing
- ✓ Basic pheromone system (food/home/danger trails, decay) — existing
- ✓ Combat system (soldiers vs workers, damage, death) — existing
- ✓ Water physics (DF-style depth, pressure, flow, drowning, rain) — existing
- ✓ Environmental hazards (cave-ins, falling dirt) — existing
- ✓ Aphid farming mechanic — existing
- ✓ Camera/viewport with scrolling — existing
- ✓ Speed controls and pause — existing
- ✓ Stats panel with colony info — existing

### Active

- [ ] Fix broken movement/AI integration (carrying ants freeze, orphaned foraging logic)
- [ ] Tune activity probabilities (ants currently too idle)
- [ ] Contextual decision-making (ants weigh situation: food proximity, danger, colony needs)
- [ ] Ant specialization (individuals develop preferences/skills based on experience)
- [ ] Colony-level adaptive strategy (shift worker/soldier ratios, expand toward food, retreat from threats)
- [ ] Richer pheromone communication (more signal types, recruitment, ant-to-ant signaling)
- [ ] Performance optimization for 1000+ ants (spatial hashing)

### Out of Scope

- Save/load functionality — explicitly deferred in original plan
- GUI/graphical rendering — this is a terminal sim
- Player direct control of individual ants — emergent behavior is the point, not micromanagement
- Networking/multiplayer — single-player observation experience
- Sound — terminal environment

## Context

The project was built through an AI-assisted 12-phase plan. Phases 1-11 are complete and the code compiles cleanly (~1,929 LOC across 19 source files). However, a critical bug exists: the movement system doesn't handle the `Carrying` ant state, causing food-carrying ants to freeze permanently. Additionally, a complete `foraging_movement()` function in `food.rs` is orphaned — never called from anywhere. Activity probabilities are extremely conservative (3-12% per tick), making ants appear lifeless.

The architecture is clean ECS (hecs) with systems for movement, digging, food, combat, pheromones, water, hazards, aphids, lifecycle, and spawning. The foundation is solid but the AI layer is where investment is needed.

**Tech stack:** Rust, ratatui 0.29, crossterm 0.28, hecs 0.10, noise 0.9, fastrand 2.0

## Constraints

- **Language**: Rust — existing codebase, no migration
- **Rendering**: Terminal via ratatui — all visuals must work in text mode
- **Performance**: Must handle 1000+ entities at 30 FPS in terminal
- **Architecture**: ECS pattern with hecs — maintain existing component/system separation

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| hecs over bevy_ecs | Lightweight, no framework overhead for terminal app | ✓ Good |
| Perlin noise terrain | Natural-looking caves and surface variation | ✓ Good |
| DF-style water (0-7 depth) | Rich emergent water behavior, proven design | ✓ Good |
| Probability-based ant AI | Simple but produces emergent behavior at scale | ⚠️ Revisit — probabilities too low, needs contextual decision layer |

---
*Last updated: 2026-02-05 after initialization*
