# Phase 1: Unfreeze & Activate - Research

**Researched:** 2026-02-06
**Domain:** ECS movement system bug fixes, activity tuning, spatial indexing (Rust / hecs)
**Confidence:** HIGH

## Summary

Phase 1 addresses three interconnected problems that make the simulation feel dead: (1) ants in Carrying, Fighting, Following, and Fleeing states freeze at their current position due to a wildcard `_ => (0,0)` catch-all in the movement system, (2) orphaned movement functions (`foraging_movement`, `fighting_movement`, `fleeing_movement`) exist with correct logic but are never called from anywhere, and (3) activity probabilities are so low (3-12% per tick) that the majority of ants sit idle at any given moment.

The fix requires three coordinated changes: wire the orphaned movement functions into the movement system with explicit match arms for every `AntState` variant (eliminating the wildcard), tune idle-to-active transition probabilities upward so ants spend more time doing things, and implement a spatial hash grid so that the combat system and future sensing systems do not degrade to O(N^2) at 500+ ants.

**Primary recommendation:** Fix the wiring first (Plan 01-01), tune activity second (Plan 01-02), add spatial indexing third (Plan 01-03). The wiring fix is the highest-impact change -- it immediately makes Carrying/Fighting/Fleeing ants move purposefully instead of freezing.

## Standard Stack

### Core

No new dependencies are needed for Phase 1. All fixes are changes to existing Rust code within the existing `hecs` + `fastrand` + `ratatui` stack.

| Library | Version | Purpose | Relevance to Phase 1 |
|---------|---------|---------|----------------------|
| hecs | 0.10.x | ECS framework | All entity queries and component mutations happen through hecs. Stay on 0.10.x. |
| fastrand | 2.0 | RNG for probabilities | Used in activity probability rolls (the values we are tuning). No upgrade needed. |

### Supporting

None required. The spatial hash grid will be hand-rolled -- it is a simple `Vec<Vec<Entity>>` indexed by `(x / cell_size, y / cell_size)`. No external crate needed for a grid-based integer-coordinate simulation.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled spatial grid | `flat_spatial` 0.6.1 | Designed for continuous 2D space with f64 coords. Our positions are integer grid cells. The overhead of converting i32 -> f64 and the library's generality are wasted. A 20-line `Vec<Vec<Entity>>` is simpler and faster for this use case. |
| Hand-rolled spatial grid | `kiddo` 5.2.4 | kd-tree for nearest-neighbor in continuous space. Same mismatch -- we have integer grid cells, not float coordinates. |

## Architecture Patterns

### Recommended Project Structure (Changes Only)

```
src/
  systems/
    movement.rs     # MODIFY: Wire all AntState arms, call orphaned functions
    food.rs         # EXISTING: Contains foraging_movement() to be wired
    combat.rs       # EXISTING: Contains fighting_movement(), fleeing_movement() to be wired
    dig.rs          # MODIFY: Tune activity probabilities in decide_worker_state()
    mod.rs          # MODIFY: Add spatial module (if separate file)
  spatial.rs        # NEW: SpatialGrid struct (or inline in a systems file)
  app.rs            # MODIFY: Create SpatialGrid, rebuild per tick, pass to combat system
```

### Pattern 1: Explicit Match Arms (Eliminate Wildcards on AntState)

**What:** Replace every `_ => (0,0)` or `_ => {}` match on `AntState` with explicit handling for all 8 variants: Idle, Wandering, Digging, Returning, Carrying, Fighting, Following, Fleeing.

**When to use:** Every `match ant.state` in every system file.

**Why:** Rust's exhaustive match checking is the primary defense against the freeze bug. When a new state is added in the future, the compiler forces every system to handle it. Wildcards silence this protection.

**Current code (movement.rs:23-42):**
```rust
let (dx, dy) = match ant.state {
    AntState::Wandering => random_movement(),
    AntState::Digging => dig_movement(pos, terrain),
    AntState::Returning => climb_movement(pos, terrain),
    AntState::Idle => {
        if fastrand::u8(..) < 10 {
            random_movement()
        } else {
            (0, 0)
        }
    }
    _ => (0, 0),  // <-- BUG: Freezes Carrying, Fighting, Following, Fleeing
};
```

**Required fix pattern:**
```rust
let (dx, dy) = match ant.state {
    AntState::Wandering => { /* existing random_movement() */ },
    AntState::Digging => { /* existing dig_movement() */ },
    AntState::Returning => { /* existing climb_movement() */ },
    AntState::Idle => { /* existing idle logic */ },
    AntState::Carrying => { /* call foraging_movement() from food.rs */ },
    AntState::Fighting => { /* call fighting_movement() from combat.rs */ },
    AntState::Fleeing => { /* call fleeing_movement() from combat.rs */ },
    AntState::Following => { /* follow food pheromone via foraging_movement() */ },
};
```

### Pattern 2: Movement Function Delegation

**What:** The movement system becomes a dispatcher that delegates to specialized movement functions defined in their respective domain modules (food.rs, combat.rs, etc.), rather than implementing all movement logic inline.

**When to use:** When wiring orphaned movement functions.

**Why:** Each domain module owns its movement logic. The movement system only needs to know "which function to call for which state" and to apply terrain passability checks. This keeps the movement system small and each domain module self-contained.

**Integration challenge:** The orphaned functions have different signatures than what the movement system currently expects:

| Function | Location | Signature | Returns |
|----------|----------|-----------|---------|
| `foraging_movement()` | food.rs:147 | `(pos, ant, member, terrain, pheromones, colonies)` | `Option<(i32, i32)>` |
| `fighting_movement()` | combat.rs:190 | `(pos, member, pheromones)` | `Option<(i32, i32)>` |
| `fleeing_movement()` | combat.rs:200 | `(pos, pheromones)` | `Option<(i32, i32)>` |

The current `movement_system()` signature is `(world, terrain)` -- it does not receive `pheromones` or `colonies`. **The movement system signature must be expanded** to include `&PheromoneGrid` and `&[ColonyState]` so it can pass these to the delegated functions.

**New movement_system signature:**
```rust
pub fn movement_system(
    world: &mut World,
    terrain: &Terrain,
    pheromones: &PheromoneGrid,
    colonies: &[ColonyState],
) { ... }
```

**Call site change in app.rs:162:**
```rust
// Current:
systems::movement::movement_system(&mut self.world, &self.terrain);
// New:
systems::movement::movement_system(
    &mut self.world,
    &self.terrain,
    &self.pheromones,
    &self.colonies,
);
```

### Pattern 3: SpatialGrid for O(1) Neighbor Lookups

**What:** A grid of cells where each cell stores a list of entity IDs at that grid coordinate. Rebuilt from scratch each tick.

**When to use:** For any system that needs "entities near position (x, y)" -- currently combat, eventually foraging and sensing.

**Design:**
```rust
pub struct SpatialGrid {
    cells: Vec<Vec<(hecs::Entity, i32, i32, u8)>>,  // entity, x, y, colony_id
    width: usize,
    height: usize,
    cell_size: i32,  // e.g., 8 tiles per cell
}
```

**Why cell_size = 8:** The combat system checks adjacency (distance <= 1). A cell size of 8 means we only need to check the current cell and 8 neighbors (9 cells) to find all entities within 8 tiles. This is more than sufficient for combat adjacency checks. For future sensing systems (utility AI sense radius of 5-8 tiles), the same grid works without modification.

**Rebuild cost:** O(N) per tick where N = number of entities. For 1000 ants on a 200x100 map with cell_size=8, the grid is 25x13 = 325 cells. Rebuilding is one pass through all positioned entities.

**Lookup cost:** O(K) per query where K = average entities per cell. For 1000 ants in 325 cells, K averages ~3. Checking 9 cells = ~27 entity comparisons per query, vs. the current O(N) = 1000 comparisons.

### Anti-Patterns to Avoid

- **Wildcard matches on AntState:** Already explained above. The `_ =>` pattern is the root cause of the freeze bug. Do not reintroduce it.
- **Monolithic movement function:** Do not put all movement logic for all states inline in movement_system. Delegate to domain modules.
- **Rebuilding spatial grid per query:** Build once at the start of the tick, use across all systems. Do not rebuild for each system that needs it.
- **Over-engineering the spatial grid:** This is a simple grid of entity lists, not a quadtree or R-tree. Integer coordinates on a bounded grid make this trivial. Do not reach for external crates.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Exhaustive match checking | Runtime state validation | Rust's built-in exhaustive match | The compiler already does this perfectly -- just stop suppressing it with wildcards |
| Pheromone gradient following | Custom pathfinding | Existing `follow_pheromone()` in pheromone.rs | Already implemented and tested; just needs to be called |
| Fighting movement | Custom enemy-seeking | Existing `fighting_movement()` in combat.rs | Already reads danger pheromone gradients correctly |
| Fleeing movement | Custom flee logic | Existing `fleeing_movement()` in combat.rs | Already finds minimum-danger direction |
| Carrying-to-home movement | Custom home-seeking | Existing `foraging_movement()` in food.rs | Already computes direction toward colony home and falls back to pheromone following |

**Key insight:** The codebase already has the correct movement logic for every AntState. The only problem is that the dispatcher (movement_system) does not call these functions. This is purely a wiring bug, not a missing-feature problem. Do NOT write new movement logic -- wire the existing functions.

## Common Pitfalls

### Pitfall 1: Changing Movement System Signature Without Updating the Call Site

**What goes wrong:** You add `pheromones: &PheromoneGrid` to `movement_system()` but forget to update the call in `app.rs:162`. The compiler will catch this, but if you work on movement.rs in isolation and do not compile app.rs, you will not see the error until integration.

**Why it happens:** The movement system is called from one place (`app.rs:162`), and the orphaned functions are in different files (food.rs, combat.rs).

**How to avoid:** After modifying the `movement_system` signature, immediately update `app.rs` and compile the full project. Do not treat movement.rs in isolation.

**Warning signs:** Any compilation error mentioning "expected 2 arguments, found 4" or similar.

### Pitfall 2: Forgetting the hecs Borrow Rules When Adding Component Queries

**What goes wrong:** The movement system currently queries `(&Position, &Ant)`. To call `foraging_movement()`, you also need `&ColonyMember`. Changing the query to `(&Position, &Ant, &ColonyMember)` will exclude entities that do not have a `ColonyMember` component. If any ants were spawned without `ColonyMember`, they silently disappear from the movement system.

**Why it happens:** hecs queries are conjunctive -- all components must be present. Adding a component to the query narrows the result set.

**How to avoid:** Check that all ant entities are spawned with `ColonyMember` (they are -- see spawn.rs:110-114). But verify this assumption: search for any `world.spawn` that creates an `Ant` without `ColonyMember`.

**Warning signs:** Ants that exist in the world but never move after the change.

### Pitfall 3: Option Return Type Handling for Orphaned Functions

**What goes wrong:** The orphaned functions (`foraging_movement`, `fighting_movement`, `fleeing_movement`) return `Option<(i32, i32)>`, while the current movement match arms return `(i32, i32)` directly. If you unwrap the Option without a fallback, ants in those states will panic when the function returns None. If you default None to (0,0), you reintroduce the freeze for edge cases.

**Why it happens:** The orphaned functions return None when they cannot determine a direction (no pheromone gradient, no path to home). This is a valid case -- the ant genuinely does not know where to go.

**How to avoid:** When the delegated function returns None, fall back to `random_movement()` rather than `(0,0)`. A confused ant should wander, not freeze. This matches real ant behavior -- an ant that loses a pheromone trail explores randomly until it picks one up again.

**Warning signs:** Ants that freeze in specific situations (e.g., carrying food but with no home pheromone trail laid yet).

### Pitfall 4: Activity Probability Tuning Creating Oscillation

**What goes wrong:** You increase the Idle-to-Wandering probability from 3.9% (`10/256`) to 50%, and now ants flicker between Idle and Wandering every frame because the Wandering-to-Idle (or Wandering-to-Digging) transition is also very high. The simulation looks frantic rather than alive.

**Why it happens:** Multiple state transition probabilities interact. `dig_ai_system()` line 132 has an 70% chance (`180/256`) to switch from Wandering to Digging. If you also make Idle-to-Wandering high, ants cycle Idle->Wandering->Digging->Returning->Surface->Wandering->Digging at breakneck speed.

**How to avoid:** Tune probabilities as a system, not individually. Map out the full state transition graph with probabilities:

```
Current probabilities:
  Idle -> Wandering:      10/256  =  3.9% per tick
  Wandering -> Digging:  180/256  = 70.3% per tick (when can_dig && on_ground)
  Digging -> Returning:   15/256  =  5.9% per tick (underground) / 3/256 = 1.2% (surface)
  Returning -> Wandering: 100% when reaches surface
  Idle -> Wandering (dig_ai): 30/256 = 11.7% per tick

Issues:
  1. Idle -> Wandering in movement.rs: 3.9% is too low
  2. Idle -> Wandering in dig.rs: 11.7% is also low
  3. Both independently try to transition Idle ants, which is confusing
  4. Wandering -> Digging at 70.3% is VERY high -- nearly every wandering ant immediately digs
```

**Recommended targets (need in-sim validation):**
- Idle -> Wandering: ~30-40% per tick (one system should own this, not two)
- Wandering -> Digging: ~20-30% per tick (reduce from 70% so ants actually wander/forage)
- Wandering -> Foraging: Should be comparable to digging when food pheromone present
- Active at any moment: >60% of worker ants should be in a non-Idle state

### Pitfall 5: Combat System O(N^2) Masked by COMBAT_INTERVAL

**What goes wrong:** You implement the spatial grid for the movement phase but forget to update `combat_system()` in combat.rs, which has the most egregious O(N^2) pattern (lines 42-56: nested loop over all combatants). The combat interval of 5 ticks masks the problem at low ant counts but still causes frame spikes at 500+ ants.

**Why it happens:** The spatial grid is naturally associated with the movement system, so it is easy to forget that combat is the system that benefits most from it.

**How to avoid:** Plan 01-03 should update BOTH the combat system and any future neighbor-query code. The spatial grid should be a shared resource in App, rebuilt once per tick, and passed to every system that does proximity checks.

**Warning signs:** Frame time spikes every 5th tick (COMBAT_INTERVAL) that get worse as ant count grows.

## Code Examples

Verified patterns from direct codebase analysis:

### Example 1: Wiring foraging_movement into movement_system

The movement system needs to collect more data during its query to call the orphaned functions. Here is the structural pattern:

```rust
// In movement_system, the query must expand:
// Current:  world.query::<(&Position, &Ant)>()
// Required: world.query::<(&Position, &Ant, &ColonyMember)>()

// For Carrying state:
AntState::Carrying => {
    // Call the existing foraging_movement function
    match crate::systems::food::foraging_movement(
        pos, ant, member, terrain, pheromones, colonies
    ) {
        Some(dir) => dir,
        None => random_movement(),  // Fallback: wander, don't freeze
    }
}
```

Note: `foraging_movement` in food.rs:155 already handles `AntState::Carrying` explicitly -- it computes direction toward `colonies[colony_id].home_x/home_y` with terrain passability checks and pheromone fallback. This is the correct behavior for carrying ants.

### Example 2: Wiring fighting_movement and fleeing_movement

```rust
// For Fighting state:
AntState::Fighting => {
    match crate::systems::combat::fighting_movement(pos, member, pheromones) {
        Some(dir) => dir,
        None => random_movement(),  // No gradient? Patrol randomly.
    }
}

// For Fleeing state:
AntState::Fleeing => {
    match crate::systems::combat::fleeing_movement(pos, pheromones) {
        Some(dir) => dir,
        None => random_movement(),  // Can't find safe direction? Random escape.
    }
}
```

### Example 3: SpatialGrid Implementation

```rust
pub struct SpatialGrid {
    cells: Vec<Vec<(hecs::Entity, i32, i32, u8)>>,
    width: usize,   // grid width in cells
    height: usize,  // grid height in cells
    cell_size: i32,
}

impl SpatialGrid {
    pub fn new(world_width: usize, world_height: usize, cell_size: i32) -> Self {
        let width = (world_width as i32 / cell_size + 1) as usize;
        let height = (world_height as i32 / cell_size + 1) as usize;
        Self {
            cells: vec![Vec::new(); width * height],
            width,
            height,
            cell_size,
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    pub fn insert(&mut self, entity: hecs::Entity, x: i32, y: i32, colony_id: u8) {
        let cx = (x / self.cell_size) as usize;
        let cy = (y / self.cell_size) as usize;
        if cx < self.width && cy < self.height {
            self.cells[cy * self.width + cx].push((entity, x, y, colony_id));
        }
    }

    /// Query all entities in a cell and its 8 neighbors
    pub fn query_nearby(&self, x: i32, y: i32) -> impl Iterator<Item = &(hecs::Entity, i32, i32, u8)> {
        let cx = (x / self.cell_size) as isize;
        let cy = (y / self.cell_size) as isize;
        let w = self.width as isize;
        let h = self.height as isize;

        // Collect neighboring cells (up to 9)
        let mut results: Vec<&(hecs::Entity, i32, i32, u8)> = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && nx < w && ny >= 0 && ny < h {
                    let idx = ny as usize * self.width + nx as usize;
                    results.extend(self.cells[idx].iter());
                }
            }
        }
        results.into_iter()
    }
}
```

### Example 4: Rebuilding SpatialGrid Per Tick (in app.rs)

```rust
// In App struct, add:
spatial_grid: SpatialGrid,

// In App::new(), add:
spatial_grid: SpatialGrid::new(terrain.width, terrain.height, 8),

// In App::update(), at start of each tick:
self.spatial_grid.clear();
for (_entity, (pos, ant, member)) in self.world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
    self.spatial_grid.insert(_entity, pos.x, pos.y, member.colony_id);
}
```

### Example 5: Updated combat_system Using SpatialGrid

```rust
// Replace the O(N^2) nested loop in combat_system:
// OLD (combat.rs:42-56):
//   for i in 0..combatants.len() {
//       for j in (i + 1)..combatants.len() { ... }
//   }
//
// NEW:
for (entity_a, x_a, y_a, colony_a, role_a, strength_a) in &combatants {
    for &(entity_b, x_b, y_b, colony_b) in spatial_grid.query_nearby(*x_a, *y_a) {
        if colony_a == &colony_b { continue; }
        let dist = (x_a - x_b).abs().max((y_a - y_b).abs());
        if dist > 1 { continue; }
        // ... combat resolution (unchanged)
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `_ => (0,0)` wildcard in movement | Explicit match arms for every AntState variant | This phase | Eliminates freeze bug for all states |
| Orphaned movement functions | Movement system delegates to domain-specific functions | This phase | Carrying ants go home, fighting ants pursue, fleeing ants escape |
| 3-12% idle-to-active probability | 30-40% per tick, tuned as a system | This phase | Majority of ants visibly active |
| O(N^2) combat neighbor scan | SpatialGrid with O(K) per-cell lookup | This phase | 30 FPS sustained at 500+ ants |

**Deprecated/outdated:**
- The `_ => (0,0)` pattern in movement.rs -- must be removed, not refactored.
- The `#![allow(dead_code)]` at the top of food.rs and combat.rs -- these exist because `foraging_movement`, `fighting_movement`, and `fleeing_movement` are never called. After wiring them, the dead_code warnings will naturally resolve. Remove the `#![allow(dead_code)]` attributes afterward to keep the compiler helping us find disconnected code in the future.

## Detailed Bug Analysis

### BUG FIX-01: Carrying Ants Freeze (movement.rs:41)

**Root cause:** `movement_system()` line 41: `_ => (0, 0)` matches `AntState::Carrying` and returns zero movement.

**Existing fix code:** `foraging_movement()` in food.rs:147-215 already handles `AntState::Carrying` at line 174. It computes a direction vector toward `colonies[colony_id].home_x/home_y` using `signum()`, checks terrain passability for direct/horizontal/vertical paths, and falls back to following `PheromoneType::Home` gradients.

**Fix:** Add `AntState::Carrying` arm to the match in `movement_system()` that calls `foraging_movement()`. The movement system needs additional parameters: `pheromones: &PheromoneGrid` and `colonies: &[ColonyState]`.

**Additional concern:** When `foraging_movement()` returns `Some(dir)` for Carrying, the returned direction is NOT checked against terrain passability inside `foraging_movement()` for all paths (the direct path IS checked at line 187, but the pheromone fallback at line 200-208 delegates to `follow_pheromone()` which does its own passability check). The movement system already applies its own passability check at line 49 (`terrain.is_passable(new_x, new_y)`), so there is no double-check needed -- but be aware that terrain checks happen in two places.

### BUG FIX-02: Fighting and Fleeing Ants Freeze (movement.rs:41)

**Root cause:** Same wildcard catch-all. `AntState::Fighting` and `AntState::Fleeing` both hit `_ => (0, 0)`.

**Existing fix code:**
- `fighting_movement()` in combat.rs:190-197: Calls `pheromones.get_gradient()` for `PheromoneType::Danger` to move the soldier toward the danger source.
- `fleeing_movement()` in combat.rs:200-235: Checks all 8 directions, sums danger pheromone from all colonies at each neighbor, and returns the direction with minimum danger that is less than current position's danger.

**Fix:** Add explicit match arms for `AntState::Fighting` and `AntState::Fleeing` in `movement_system()` that call these functions. The `fighting_movement()` needs `(pos, member, pheromones)` and `fleeing_movement()` needs `(pos, pheromones)`.

**Subtle issue with fighting_movement:** It follows the danger pheromone gradient to find enemies. But danger pheromones are deposited by the combat system (combat.rs:78-80) only when combat actually occurs. If no combat has happened yet, there is no danger pheromone to follow, and `fighting_movement()` returns None. The fallback should be `random_movement()` so soldiers patrol rather than freeze.

### BUG FIX-04: Following State Has No Movement Logic Anywhere

**Root cause:** `AntState::Following` is defined in components.rs:54 but has no movement function anywhere in the codebase. It is not handled in `movement_system()` (caught by wildcard), not in `foraging_movement()` (its match returns None for non-Wandering/non-Carrying states), not in `fighting_movement()`, not in `fleeing_movement()`.

**Design decision needed:** What should Following mean? Looking at the codebase, it is likely intended for ants following a pheromone trail to food (distinct from Wandering which is random). The `foraging_movement()` function handles `AntState::Wandering` by following food pheromone trails when signal is above 0.1 (food.rs:157-171).

**Recommended fix:** Wire `AntState::Following` to use the same food-pheromone-following logic as `foraging_movement()` handles for Wandering. Alternatively, treat Following as an alias for "pheromone-guided wandering" -- call `foraging_movement()` with the ant's current state, or add a dedicated arm. Since Following is not currently assigned by any AI system, it is a lower-priority fix -- but the match arm MUST exist to eliminate the wildcard.

## Activity Probability Analysis

### Current State Transition Map

```
                  dig_ai 11.7%              dig_ai 70.3%
    Idle ──────────────────────> Wandering ──────────────────> Digging
     ^    movement.rs 3.9%          |                            |
     |                              |                      dig_ai 5.9%
     |                              |                      (underground)
     |                              v                            |
     |                         Carrying                          v
     |                     (freeze: wildcard)              Returning
     |                              |                            |
     |                              |                      dig_ai 100%
     |                              v                      (at surface)
     |                      Food deposited                       |
     |                              |                            |
     +--------- (food.rs) ---------+                            |
     |                                                           |
     +-----------------------------------------------------------+

     Fighting/Fleeing: Set by combat/flee AI systems, then freeze (wildcard)
```

### Identified Probability Issues

1. **Idle -> Wandering is TOO LOW:** Two systems fight over this transition. `movement_system()` line 35 gives 3.9% per tick. `dig_ai_system()` line 167 gives 11.7% per tick. Combined probability of staying Idle per tick: `(1 - 0.039) * (1 - 0.117) = 0.848`, meaning 15.2% chance to become Wandering per tick. At 30 FPS, median time to leave Idle = ~4.2 ticks = 0.14 seconds. This seems fast in theory, but the issue is the OTHER direction: Wandering immediately transitions to Digging at 70.3%, and Digging eventually returns to surface where many ants cycle back through Idle. The effective Idle population depends on the full cycle time.

2. **Wandering -> Digging is TOO HIGH:** At 70.3%, nearly every wandering ant immediately starts digging on its first tick if it is near diggable terrain (which is most of the map surface). This means "Wandering" barely exists as a visible state. Ants go Idle -> Wandering (1 tick) -> Digging (1 tick) -> stays underground. The surface looks empty.

3. **No explicit Wandering -> Foraging transition:** The `foraging_system()` in food.rs changes Wandering ants to Carrying when they are standing ON a food source (exact position match, food.rs:91). But there is no transition that makes ants WALK TOWARD food. The `foraging_movement()` function would do this (follow food pheromone trail), but it is never called. So ants only pick up food by random chance of walking onto the exact tile.

4. **Carrying ants never reach home:** Even when an ant picks up food and transitions to Carrying, the wildcard freezes it at the pickup location. The food deposit check in `check_deposit()` (food.rs:218-245) looks for ants within distance 3 of home, but frozen ants never get there. Food income is effectively zero except for ants that happen to spawn within 3 tiles of home.

### Recommended Probability Targets

| Transition | Current | Target | Rationale |
|------------|---------|--------|-----------|
| Idle -> Wandering (single source) | 3.9% + 11.7% (two systems) | 30-40% (one system) | Consolidate to one system. Idle should last ~3-10 ticks max. |
| Wandering -> Digging | 70.3% | 15-25% | Ants need time to wander and discover food. Reduce digging impulse. |
| Idle -> Digging (direct, at dig site) | N/A | Keep implicit through Wandering | Ants should not jump straight to Digging from Idle. |
| Idle -> Wandering (dig_ai) | 11.7% | Remove or reduce | Let movement.rs own the Idle->Wandering transition exclusively. |

### Tuning Strategy

1. **Consolidate Idle->Wandering to ONE system:** Either movement.rs or dig_ai_system, not both. Recommendation: movement.rs owns it, because movement runs every tick and should be the authority on "should this idle ant start moving?"

2. **Reduce Wandering->Digging:** From `180/256` (70.3%) to approximately `50/256` (19.5%). This gives ants ~5 ticks of Wandering before they start digging, during which they can encounter food, follow pheromones, etc.

3. **Validate with state distribution:** After tuning, run for 500 ticks and count ant state distribution. Target: <20% Idle, >30% Wandering, remainder in active states (Digging, Returning, Carrying, etc.).

## Spatial Grid Sizing Analysis

**Map size:** 200 x 100 tiles (from app.rs:55: `Terrain::generate(200, 100, seed)`)

**Ant count:** 3 colonies x 10 initial workers = 30 initial, growing over time via lifecycle. Target: 500+ sustained.

**Cell size options:**

| Cell Size | Grid Dimensions | Cells | Entities/Cell (500 ants) | Pros | Cons |
|-----------|----------------|-------|--------------------------|------|------|
| 4 | 50 x 25 = 1250 | 0.4 avg | Fine-grained, very few entities per cell | More cells to iterate in 9-cell neighborhood |
| 8 | 25 x 13 = 325 | 1.5 avg | Good balance | -- |
| 16 | 13 x 7 = 91 | 5.5 avg | Fewest cells | More entities per cell to filter |

**Recommendation:** Cell size 8. It balances cell count vs. entities-per-cell and aligns well with future sense radius needs (utility AI typically uses 5-8 tile sense radius).

## Integration Sequence

The three plans must execute in order due to dependencies:

```
Plan 01-01: Wire orphaned movement functions
  Modifies: movement.rs (add match arms + signature change)
  Modifies: app.rs (update call site with new args)
  Depends on: Nothing (pure bug fix)
  Validates: Carrying ants move home, Fighting/Fleeing ants move purposefully

Plan 01-02: Tune activity probabilities
  Modifies: movement.rs (Idle probability), dig.rs (Wandering->Digging probability)
  Depends on: 01-01 (no point tuning activity if active states freeze)
  Validates: >60% of ants visibly active at any moment

Plan 01-03: Implement spatial hash grid
  Modifies: app.rs (add SpatialGrid, rebuild per tick)
  Modifies: combat.rs (use SpatialGrid instead of nested loop)
  Adds: spatial.rs (new module)
  Depends on: 01-01 and 01-02 (bugs should be fixed before optimizing)
  Validates: 30 FPS at 500+ ants
```

## Open Questions

1. **What should AntState::Following do?**
   - What we know: The state exists in the enum but is never assigned by any AI system and has no movement logic.
   - What is unclear: Whether Following should track food pheromone trails, follow another ant, or follow recruitment signals.
   - Recommendation: For Phase 1, wire it to follow food pheromones (same as foraging_movement Wandering case). Revisit the semantics in a later phase when utility AI gives it a more specific meaning.

2. **Should dig_ai_system be the authority on Idle->Wandering, or movement_system?**
   - What we know: Both systems currently try to transition Idle ants. This creates unpredictable combined probability.
   - What is unclear: The original developer's intent for separating these.
   - Recommendation: Let `movement_system` own the Idle->Wandering transition (it runs every tick and is the natural home for "should this ant start moving?"). Remove or greatly reduce the Idle case in `dig_ai_system`.

3. **Is the `#![allow(dead_code)]` on food.rs and combat.rs hiding other orphaned code?**
   - What we know: These attributes suppress warnings about `foraging_movement`, `fighting_movement`, and `fleeing_movement` being unused.
   - What is unclear: Whether there are other dead functions in these files beyond the three we are wiring.
   - Recommendation: After wiring is complete, remove the `#![allow(dead_code)]` attributes and fix any remaining warnings.

## Sources

### Primary (HIGH confidence)
- Direct codebase analysis of all source files in `E:/VS Code Projects/AntTrails/src/` -- verified exact line numbers, function signatures, and data flow
- `movement.rs:41` -- confirmed `_ => (0,0)` wildcard as freeze source
- `food.rs:147-215` -- confirmed `foraging_movement()` exists with correct Carrying logic, never called
- `combat.rs:190-235` -- confirmed `fighting_movement()` and `fleeing_movement()` exist, never called
- `app.rs:146-225` -- confirmed system execution order and `movement_system` call site
- `dig.rs:132` -- confirmed 70.3% Wandering->Digging transition probability
- `components.rs:47-56` -- confirmed all 8 AntState variants

### Secondary (MEDIUM confidence)
- Prior research: `E:/VS Code Projects/AntTrails/.planning/research/ARCHITECTURE.md` -- build order and integration analysis
- Prior research: `E:/VS Code Projects/AntTrails/.planning/research/PITFALLS.md` -- Pitfall 8 (wildcard freeze), Pitfall 9 (O(N^2) combat)
- Prior research: `E:/VS Code Projects/AntTrails/.planning/research/STACK.md` -- spatial indexing analysis (grid vs. kd-tree)

### Tertiary (LOW confidence)
- Activity probability targets (30-40% idle-to-wandering, 15-25% wandering-to-digging) -- these are educated estimates based on desired visual behavior. Must be validated empirically in-sim.
- Cell size 8 for spatial grid -- reasonable default but may need adjustment based on actual entity density patterns.

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH -- no new dependencies, purely internal code changes verified against codebase
- Architecture Patterns: HIGH -- wiring pattern verified by reading both producer and consumer code; spatial grid pattern is well-established
- Bug Fixes: HIGH -- root cause identified at exact line numbers with existing fix code located
- Activity Tuning: MEDIUM -- targets are estimates that need empirical validation
- Pitfalls: HIGH -- derived from direct code analysis of interaction points

**Research date:** 2026-02-06
**Valid until:** 2026-03-06 (stable -- all findings are against existing codebase, not external libraries)
