# Architecture Patterns: Emergent Agent AI for AntTrails

**Domain:** Emergent agent-based simulation / ECS game AI
**Researched:** 2026-02-06
**Overall confidence:** HIGH (verified against codebase analysis + published game AI patterns)

## Executive Summary

Adding emergent AI to AntTrails requires four interlocking layers built on top of the existing ECS: (1) a Utility AI decision system that replaces the current `match`-on-`AntState` logic, (2) per-ant memory and specialization components, (3) colony-level aggregation that influences individual scoring curves, and (4) an expanded pheromone vocabulary. All four layers integrate through **new ECS components and new systems** while keeping existing systems largely intact. The key insight is that **colony intelligence must emerge from individual utility scoring influenced by shared state** -- not from an explicit "colony brain" that issues orders.

## Recommended Architecture

```
                    STIMULUS LAYER
        +---------------------------------+
        | SenseRadius scan (terrain,       |
        | pheromones, neighbors, water,    |
        | food, enemies)                   |
        +---------------------------------+
                        |
                        v
                 DECISION LAYER (new)
        +---------------------------------+
        | UtilityAI System                 |
        | For each ant:                    |
        |   Score all candidate Actions    |
        |   Select highest (or weighted    |
        |     random from top bucket)      |
        |   Write ActionIntent component   |
        +---------------------------------+
                        |
                        v
                  ACTION LAYER (modified)
        +---------------------------------+
        | Existing systems execute based   |
        | on AntState set by decision:     |
        |   movement, dig, food, combat    |
        | New systems for new actions:     |
        |   recruit, guard, nurse, scout   |
        +---------------------------------+
                        |
                        v
                 FEEDBACK LAYER (new)
        +---------------------------------+
        | Outcome observation:             |
        |   Did foraging succeed?          |
        |   Did combat result in death?    |
        |   Was food deposited?            |
        | Update Memory component          |
        | Update ColonyStrategy resource   |
        | Deposit pheromones               |
        +---------------------------------+
                        |
                        v
                 (loops back to Stimulus)
```

### Component Boundaries

| Component | Responsibility | New/Modify | Communicates With |
|-----------|---------------|------------|-------------------|
| `SenseData` | Per-ant snapshot of nearby world state, rebuilt each tick | NEW component | Read by UtilityAI system |
| `AntMemory` | Individual experience counters (food found, combats survived, areas explored) | NEW component | Read/written by decision + feedback systems |
| `Specialization` | Derived aptitudes from memory (forager affinity, soldier affinity, scout affinity) | NEW component | Read by UtilityAI scoring |
| `ActionIntent` | The chosen action + target for this tick | NEW component | Read by action execution systems |
| `Ant` | Role + state (kept as-is, state set by ActionIntent resolution) | KEEP existing | Written by decision system |
| `ColonyStrategy` | Colony-level aggregated needs (food urgency, defense urgency, expansion urgency) | NEW resource (not per-entity) | Read by UtilityAI scoring, written by aggregation system |
| `PheromoneGrid` | Expanded with new pheromone types | MODIFY existing | Read by SenseData builder, written by feedback |
| `ColonyState` | Add population ratios, threat history, food trend | MODIFY existing | Feeds ColonyStrategy computation |

### Data Flow

```
Per tick, in order:

1. SENSE:    SenseDataBuilder system
                reads: Position, Terrain, PheromoneGrid, WaterGrid, nearby entities
                writes: SenseData component on each ant

2. DECIDE:   UtilityAI system
                reads: SenseData, AntMemory, Specialization, ColonyStrategy
                writes: ActionIntent component, updates AntState on Ant component

3. EXECUTE:  Existing systems (movement, dig, food, combat, pheromone) + new systems
                reads: ActionIntent, Position, Ant, Carrying, Fighter, etc.
                writes: Position, Terrain, Carrying, Fighter, PheromoneGrid, etc.

4. FEEDBACK: OutcomeObserver system
                reads: Position, Ant, Carrying (did state change?), Fighter (took damage?)
                writes: AntMemory (increment counters)
             ColonyAggregator system
                reads: all AntMemory, ColonyMember, Ant, ColonyState
                writes: ColonyStrategy resource, updated ColonyState
             PheromoneDeposit system (already exists, expanded)
                reads: Position, Ant, AntMemory
                writes: PheromoneGrid

5. CLEANUP:  Existing cleanup_dead + specialization recalculation (periodic)
```

## Component Design Details

### 1. SenseData -- The Ant's "Eyes" (NEW Component)

```rust
pub struct SenseData {
    // Nearby food (distance, direction, amount)
    pub nearest_food: Option<(f32, i32, i32)>,
    // Nearby enemies (count within radius, nearest distance)
    pub enemy_count: u8,
    pub nearest_enemy_dist: f32,
    // Nearby allies (count within radius)
    pub ally_count: u8,
    // Pheromone readings at current + adjacent tiles
    pub food_pheromone: f32,
    pub home_pheromone: f32,
    pub danger_pheromone: f32,
    pub recruit_pheromone: f32,
    // NEW pheromone types
    pub scout_pheromone: f32,
    pub territory_pheromone: f32,
    // Environment
    pub water_depth: u8,
    pub on_surface: bool,
    pub can_dig: bool,
    pub distance_from_home: f32,
    // Colony needs (copied from ColonyStrategy for local access)
    pub colony_food_urgency: f32,
    pub colony_defense_urgency: f32,
    pub colony_expansion_urgency: f32,
}
```

**Rationale:** Rather than having the UtilityAI system query the world directly (which would create complex borrow-checker issues with hecs), a dedicated SenseDataBuilder system pre-computes a flat struct per ant. This is cache-friendly, avoids repeated spatial queries, and makes the scoring functions pure: `fn score(sense: &SenseData, memory: &AntMemory, spec: &Specialization) -> f32`.

**Integration point:** The SenseDataBuilder system replaces no existing systems. It runs at the START of the AI phase (before `dig_ai_system`, `soldier_ai_system`, etc.), and those existing AI systems are eventually retired as the UtilityAI system subsumes them.

### 2. AntMemory -- Individual Experience (NEW Component)

```rust
pub struct AntMemory {
    // Lifetime counters
    pub food_collected: u16,
    pub food_deposited: u16,
    pub combats_fought: u16,
    pub combats_won: u16,
    pub distance_explored: u32,
    pub times_fled: u16,
    pub ticks_digging: u16,
    pub ticks_nursing: u16,  // time near eggs/larvae

    // Recent history (sliding window, last N ticks)
    pub recent_food_found: u8,      // food found in last 200 ticks
    pub recent_danger_seen: u8,     // danger encountered in last 200 ticks
    pub last_food_direction: Option<(i32, i32)>,  // general direction of last food
    pub last_home_direction: Option<(i32, i32)>,  // general direction toward home
}
```

**Rationale:** True emergent specialization requires ants to have differentiated histories. Two ants with identical genes but different experiences should behave differently. This is the minimum viable memory: cheap counters that accumulate over time. No complex data structures, no pathfinding caches, no neural network weights. Just counters that produce divergent utility scores through the Specialization component.

**Integration point:** Added to ants at spawn (all zeroed). Updated by the OutcomeObserver system each tick. Never modified by any existing system -- only by the new feedback layer.

### 3. Specialization -- Derived Aptitudes (NEW Component)

```rust
pub struct Specialization {
    pub forager_affinity: f32,    // 0.0 - 1.0
    pub soldier_affinity: f32,    // 0.0 - 1.0
    pub scout_affinity: f32,      // 0.0 - 1.0
    pub digger_affinity: f32,     // 0.0 - 1.0
    pub nurse_affinity: f32,      // 0.0 - 1.0
}
```

**Derived periodically** (every ~100 ticks, not every frame) from AntMemory:

```
forager_affinity = normalize(food_collected + food_deposited * 2)
soldier_affinity = normalize(combats_fought + combats_won * 3)
scout_affinity   = normalize(distance_explored)
digger_affinity  = normalize(ticks_digging)
nurse_affinity   = normalize(ticks_nursing)
```

**How it affects behavior:** Specialization values act as **multipliers on utility scores** for related actions. A high forager_affinity multiplies the "go forage" action score by e.g. 1.3x, making that ant more likely to choose foraging when conditions are ambiguous. This is NOT role assignment -- it is a soft bias that still yields to extreme situations (a specialist forager will still flee from combat).

**Integration point:** Lives alongside `Ant` component. Does NOT replace `AntRole` -- role is still the hard constraint (soldiers can't carry food), while specialization is the soft preference within a role's available actions.

### 4. UtilityAI Decision System (NEW System)

This is the core architectural addition. It replaces `dig_ai_system`, `soldier_ai_system`, and `flee_system` with a single, unified decision system.

**Architecture:**

```rust
// Actions available to workers
enum WorkerAction {
    Wander,
    Forage,         // move toward food
    ReturnFood,     // carry food home
    Dig,            // expand tunnels
    Scout,          // explore unknown areas
    Flee,           // escape danger
    Nurse,          // tend eggs/larvae
    Recruit,        // deposit recruit pheromone
    Guard,          // protect area (aphids, nest entrance)
}

// Actions available to soldiers
enum SoldierAction {
    Patrol,         // wander territory edges
    Attack,         // move toward enemy
    Defend,         // hold position near threat
    Escort,         // follow workers
    Flee,           // overwhelming odds
    RespondRecruit, // move toward recruit pheromone
}

// Each action has a scoring function:
fn score_forage(sense: &SenseData, memory: &AntMemory, spec: &Specialization) -> f32 {
    let mut score = 0.0;

    // Base: is there food nearby?
    if let Some((dist, _, _)) = sense.nearest_food {
        score += response_curve_inverse(dist / MAX_SENSE_RADIUS);  // closer = higher
    }

    // Consideration: food pheromone trail
    score += sense.food_pheromone * 0.3;

    // Consideration: colony needs food
    score *= 0.5 + sense.colony_food_urgency * 0.5;

    // Consideration: not carrying food already (handled by action availability)

    // Specialization multiplier
    score *= 0.8 + spec.forager_affinity * 0.4;  // range: 0.8x to 1.2x

    // Safety check: danger nearby reduces foraging desire
    if sense.danger_pheromone > 0.3 {
        score *= 0.3;
    }

    score.clamp(0.0, 1.0)
}
```

**Selection strategy:** After scoring all available actions, the system does NOT simply pick the max. Instead, it uses **weighted random from top bucket** -- all actions within 10% of the top score are candidates, selected by their relative scores as weights. This prevents oscillation and produces natural variation.

**Integration with existing AntState:** The UtilityAI system maps its selected action to an `AntState` value and writes it to the existing `Ant` component. This means all existing movement, dig, food, and combat systems continue to work unchanged:

| UtilityAI Action | Maps to AntState |
|-----------------|------------------|
| Wander, Scout, Patrol | `Wandering` |
| Forage | `Following` (toward food pheromone) |
| ReturnFood | `Carrying` / `Returning` |
| Dig | `Digging` |
| Flee | `Fleeing` |
| Attack, Defend, RespondRecruit | `Fighting` |
| Nurse, Guard | `Idle` (at target location) |
| Recruit | `Wandering` (with recruit pheromone deposit) |

**Critical integration detail:** The UtilityAI system writes ActionIntent AND updates AntState. Existing systems read AntState. This provides backward compatibility: the UtilityAI system is a drop-in replacement for the existing `dig_ai_system` / `soldier_ai_system` / `flee_system` functions. During migration, both can coexist (UtilityAI for workers, old system for soldiers, etc.).

### 5. ColonyStrategy -- Emergent Colony Intelligence (NEW Resource)

```rust
pub struct ColonyStrategy {
    pub food_urgency: f32,        // 0.0 (plenty) to 1.0 (starving)
    pub defense_urgency: f32,     // 0.0 (safe) to 1.0 (under attack)
    pub expansion_urgency: f32,   // 0.0 (enough space) to 1.0 (crowded)
    pub nurse_urgency: f32,       // 0.0 (enough nurses) to 1.0 (eggs unattended)
}
```

**This is NOT a "colony brain" issuing orders.** It is an aggregation of colony state that individual ants read as one of many inputs to their utility scoring. The colony does not decide "send 20 ants to forage" -- instead, when food is low, `food_urgency` rises, which increases the forage action score for ALL ants, causing more of them to independently choose foraging. This is how real ant colonies work: no central command, just shared chemical signals that bias individual behavior.

**Computation (ColonyAggregator system, runs once per ~30 ticks):**

```
food_urgency = 1.0 - (food_stored / (population * ticks_until_starvation)).clamp(0, 1)
defense_urgency = (recent_combat_events + danger_pheromone_total) / normalization_factor
expansion_urgency = population / max_comfortable_population
nurse_urgency = (eggs + larvae) / (ants_near_nursery + 1)
```

**Integration point:** Stored as a `Vec<ColonyStrategy>` in App (one per colony), passed by reference to the UtilityAI system. Updated by the ColonyAggregator system at the end of each tick. Not an ECS component -- it is a shared resource like `PheromoneGrid`.

### 6. Expanded Pheromone System (MODIFY Existing)

**Current state:** 3 pheromone types (Food, Home, Danger) in `PheromoneGrid`.

**Expanded to 6 types:**

| Type | Current? | Purpose | Deposited When |
|------|----------|---------|---------------|
| Food | YES | Trail to food source | Carrying food home |
| Home | YES | Trail to nest | Wandering/exploring |
| Danger | YES | Enemy/hazard warning | Combat, seeing enemies |
| Recruit | defined in enum, not in grid | "Help needed here" | Combat, large food find, defending aphids |
| Scout | NEW | "Unexplored/interesting area" | Exploring new territory |
| Territory | NEW | "This is our area" | Persistent presence in an area |

**Grid modification:** The `PheromoneGrid` data layout changes from `colony * 3` to `colony * 6` pheromone values per tile. The `PheromoneType` enum in `pheromone.rs` already has `Recruit` defined, but the grid only allocates 3 slots. Expanding to 6 is a data change:

```rust
// Current: data size = width * height * max_colonies * 3
// New:     data size = width * height * max_colonies * 6
```

The `index()` method and all pheromone read/write functions update accordingly.

**Concentration-based signaling:** Beyond simple gradient following, ants should respond to **pheromone concentration thresholds**:
- Recruit > 0.5: Soldiers prioritize moving toward source
- Danger > 0.7: All ants flee regardless of other scores
- Territory from enemy colony > 0.3: Soldiers become alert
- Scout > 0.3: Explorers drawn to investigate

These thresholds feed into utility scoring as considerations, not as hard overrides (except extreme danger, which acts as an early-out zero-score on non-flee actions).

**Integration point:** `PheromoneGrid::new()` allocation changes. `pheromone_deposit_system` expands to deposit new types based on ActionIntent. All existing pheromone reading code works -- callers just gain access to new types.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Central Colony Controller
**What:** Creating an explicit "colony brain" entity that issues orders to individual ants.
**Why bad:** Destroys emergence. Behavior becomes scripted top-down rather than bottom-up. Debugging becomes "why did the brain decide X" rather than observing natural patterns. Scaling is poor -- the controller must evaluate O(n) ants each tick.
**Instead:** Use ColonyStrategy as a passive influence on individual utility scores. Colony intelligence should be an *observer's interpretation* of aggregated individual behaviors, not a programmed strategy.

### Anti-Pattern 2: Scoring Every Tick
**What:** Running the full UtilityAI scoring for every ant every tick.
**Why bad:** With 1000+ ants and 6-10 actions each with 5+ considerations, this is 30,000-50,000 score evaluations per tick. At 30 FPS, that is up to 1.5M evaluations per second.
**Instead:** Stagger decisions. Ants only re-evaluate every N ticks (10-30), unless interrupted by a state-invalidating event (took damage, reached destination, pheromone spike). Between re-evaluations, ants continue their current action.

### Anti-Pattern 3: Perfect Information
**What:** Giving ants access to global state (all food positions, all enemy positions, colony food stores).
**Why bad:** Produces optimal but unrealistic behavior. Ants move with GPS-like precision. No room for emergent mistakes that make the simulation interesting.
**Instead:** SenseData should be limited to a small radius (5-8 tiles). Colony-level information reaches ants indirectly through pheromones and ColonyStrategy urgency values (which are a legitimate abstraction of chemical signaling within the nest).

### Anti-Pattern 4: Hard Role Locking from Specialization
**What:** Once an ant specializes as a forager, it can only forage.
**Why bad:** Prevents colony adaptation. In real ant colonies, even specialized workers switch tasks when colony needs shift dramatically.
**Instead:** Specialization is a soft multiplier (0.8x to 1.2x), not a gate. Colony urgency values can override specialization through their own multiplier effect. A specialist forager in a colony under attack WILL fight, just slightly less eagerly than a soldier-specialist.

## Performance Considerations

| Concern | At 100 ants | At 500 ants | At 1000+ ants |
|---------|------------|------------|--------------|
| SenseData build | No concern -- N * neighbor scan in radius | Needs spatial hashing for neighbor queries | Spatial hashing mandatory; consider grid-based lookup |
| UtilityAI scoring | Score all each tick fine | Stagger: 50 ants per tick at 10-tick interval | Stagger: 33-100 ants per tick at 10-30 tick interval |
| Memory updates | Trivial counter increments | Trivial | Trivial |
| Colony aggregation | Sum over 100 entities | Sum over 500 entities | Sum over 1000; run every 30 ticks, not every tick |
| Pheromone grid | 2x memory vs current (6 types vs 3) | Same | Same -- grid size depends on terrain, not ant count |

**Critical dependency:** A spatial index (grid-based, not HashMap) is needed before SenseData can scale to 1000+ ants. The SenseDataBuilder must query "all entities within radius R of position P" efficiently. Without spatial indexing, this is O(n^2). With a grid-based spatial index (cell size = sense radius), it is O(n * k) where k is average entities per cell.

## Build Order: What Depends On What

```
Phase A: Foundation (no dependencies, enables everything else)
  1. Fix existing bugs (Carrying movement freeze, orphaned foraging_movement)
  2. Tune activity probabilities (ants too idle)
  3. Add spatial index (grid-based, needed for SenseData)

Phase B: Decision Layer (depends on A.3 for sense data)
  4. Add SenseData component + SenseDataBuilder system
  5. Add AntMemory component (zeroed at spawn)
  6. Add Specialization component (zeroed at spawn)
  7. Implement UtilityAI system for Workers (forage, dig, wander, flee)
     - Wire to existing AntState so existing systems execute actions
     - RETIRE dig_ai_system (subsumed)
  8. Implement UtilityAI system for Soldiers (patrol, attack, defend, flee)
     - RETIRE soldier_ai_system and flee_system (subsumed)

Phase C: Feedback + Memory (depends on B.7)
  9.  Add OutcomeObserver system (updates AntMemory)
  10. Add SpecializationCalculator system (periodic, derives affinities from memory)
  11. Wire specialization into UtilityAI scoring as multipliers

Phase D: Colony-Level Intelligence (depends on B.7, benefits from C)
  12. Add ColonyStrategy resource
  13. Add ColonyAggregator system (periodic, computes urgencies)
  14. Wire colony urgencies into UtilityAI scoring as multipliers

Phase E: Rich Communication (depends on B.4 for SenseData, independent of C/D)
  15. Expand PheromoneGrid from 3 to 6 types
  16. Add Scout and Territory pheromone deposit logic
  17. Add Recruit pheromone deposit + response scoring
  18. Add concentration-based threshold behaviors

Phase F: Refinement (depends on all above)
  19. Add new WorkerActions: Nurse, Guard, Recruit
  20. Add new SoldierActions: Escort, RespondRecruit
  21. Tune response curves and scoring weights through observation
  22. Performance optimization (staggered evaluation, early-out scoring)
```

**Key dependency chain:** A.3 (spatial index) blocks B.4 (SenseData) blocks B.7 (UtilityAI). Everything else can proceed incrementally once B.7 exists.

**Phases C, D, and E are largely independent of each other** and can be interleaved. The UtilityAI system works without memory/specialization (just less interesting) and without colony strategy (just less coordinated) and without expanded pheromones (just less communicative). Each layer makes the behavior richer but none is a hard prerequisite for the others after the decision layer exists.

## Integration Points with Existing Code

### Files Modified

| File | What Changes | Why |
|------|-------------|-----|
| `components.rs` | Add `SenseData`, `AntMemory`, `Specialization`, `ActionIntent` structs | New component definitions |
| `colony.rs` | Add trend/history fields to `ColonyState` | Feed ColonyAggregator |
| `systems/pheromone.rs` | Expand `PheromoneType` enum, change grid size from 3 to 6 | Richer communication |
| `systems/mod.rs` | Add new module declarations | New systems |
| `app.rs` | Add `ColonyStrategy` vec to App, add new systems to update loop, add spatial index | New resources and system ordering |
| `systems/spawn.rs` | Add `AntMemory` and `Specialization` to ant spawn bundles | New components at entity creation |

### Files Added

| File | Purpose |
|------|---------|
| `systems/sense.rs` | SenseDataBuilder system |
| `systems/decision.rs` | UtilityAI system (scoring + selection) |
| `systems/feedback.rs` | OutcomeObserver + ColonyAggregator systems |
| `systems/specialization.rs` | SpecializationCalculator system |
| `spatial.rs` | Grid-based spatial index |
| `scoring.rs` | Response curves, utility math, action definitions |

### Files Retired (systems subsumed)

| Current Function | Replaced By | When |
|-----------------|------------|------|
| `dig::dig_ai_system()` | `decision::utility_ai_system()` | Phase B.7 |
| `combat::soldier_ai_system()` | `decision::utility_ai_system()` | Phase B.8 |
| `combat::flee_system()` | `decision::utility_ai_system()` | Phase B.8 |

Note: `dig::dig_system()` (the actual digging execution) is KEPT. Only the AI decision functions are retired. All action-execution systems remain.

### System Ordering in app.rs update()

```
// Current order (for reference):
//   dig_ai → soldier_ai → flee → movement → dig → forage → combat → aphid → pheromone → lifecycle → food_regrow → hazard → water → cleanup

// New order:
//   sense_build → utility_ai_decide → movement → dig → forage → combat → aphid → pheromone_deposit → outcome_observe → colony_aggregate → spec_recalc(periodic) → lifecycle → food_regrow → hazard → water → cleanup
```

## Patterns to Follow

### Pattern 1: Consideration-Based Scoring
**What:** Each action's score is the product of multiple independent consideration functions, each returning 0.0-1.0.
**When:** All utility scoring.
**Why:** Multiplying considerations means any single zero-score vetoes the action (natural "can't do this" gates). Non-zero scores compose naturally. Adding new considerations does not require rethinking existing ones.

### Pattern 2: Staggered Evaluation
**What:** Ants re-evaluate decisions on a rotating schedule, not every tick.
**When:** UtilityAI system.
**Why:** Amortizes CPU cost across ticks. Also produces more natural behavior -- real ants don't reconsider their plan 30 times per second. An ant committed to foraging should forage for many ticks before re-evaluating, unless interrupted.

### Pattern 3: Soft Influence, Not Hard Control
**What:** Colony needs and specialization modify scores by 0.5x-1.5x, never force specific actions.
**When:** ColonyStrategy and Specialization integration.
**Why:** Preserves emergence. The developer sets the scoring rules; the behavior that arises from 1000 ants independently scoring their situations is what makes the simulation interesting. Hard overrides ("if food < 50, all ants forage") produce mechanical, predictable behavior.

### Pattern 4: Sense-Decide-Act-Feedback Loop
**What:** Strict separation of perception, decision, action, and learning into distinct systems that run in order.
**When:** Every tick.
**Why:** Each system has clear inputs and outputs. No system both reads and writes the same data in the same phase. This prevents order-dependent bugs and makes each system independently testable.

## Sources

- [Utility AI Architecture (The Shaggy Dev)](https://shaggydev.com/2023/04/19/utility-ai/) -- MEDIUM confidence, practical implementation guide
- [ECS and AI Integration (Pixelmatic)](https://pixelmatic.github.io/articles/2020/05/13/ecs-and-ai.html) -- MEDIUM confidence, ECS-specific integration patterns
- [Game AI Pro Chapter 9: Utility Theory (Dave Mark)](http://www.gameaipro.com/GameAIPro/GameAIPro_Chapter09_An_Introduction_to_Utility_Theory.pdf) -- HIGH confidence, authoritative reference on utility AI
- [Game AI Pro 3 Chapter 13: Choosing Effective Utility-Based Considerations](http://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter13_Choosing_Effective_Utility-Based_Considerations.pdf) -- HIGH confidence, scoring design patterns
- [Ant Colony Simulation (Practicing Ruby)](https://practicingruby.com/articles/ant-colony-simulation) -- MEDIUM confidence, emergent behavior from simple rules
- [Active Inference Framework for Ant Colony Behavior (Frontiers)](https://www.frontiersin.org/journals/behavioral-neuroscience/articles/10.3389/fnbeh.2021.647732/full) -- MEDIUM confidence, biological basis for decision architecture
- [Stigmergy (Wikipedia)](https://en.wikipedia.org/wiki/Stigmergy) -- HIGH confidence, definition and types of indirect communication
- [Utility System (Wikipedia)](https://en.wikipedia.org/wiki/Utility_system) -- HIGH confidence, foundational concept definition
- Codebase analysis of `E:/VS Code Projects/AntTrails/src/` -- HIGH confidence, direct inspection of all 19 source files
