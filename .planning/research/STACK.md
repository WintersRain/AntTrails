# Technology Stack: Emergent AI for AntTrails

**Project:** AntTrails - Terminal-based ant colony simulator
**Dimension:** Emergent AI behavior systems
**Researched:** 2026-02-06
**Overall Confidence:** MEDIUM-HIGH

## Executive Summary

AntTrails already has a working tech foundation (hecs ECS, ratatui, crossterm, Perlin noise). This stack research focuses exclusively on what's needed to replace the broken dice-roll AI with contextual decision-making that produces emergent colony intelligence. The core recommendation is: **hand-roll a lightweight Utility AI scoring system directly, implement stigmergy-based pheromone communication on a 2D grid, and use ECS component composition for ant role/state management.** No heavy external AI libraries needed.

---

## Current Stack (Already in Place -- Do Not Change)

| Technology | Version | Purpose |
|------------|---------|---------|
| hecs | 0.10.x | ECS framework |
| ratatui | 0.29 | Terminal rendering |
| crossterm | 0.28 | Terminal input/events |
| noise | 0.9 | Perlin noise terrain |
| fastrand | 2.0 | Random number generation |
| anyhow | 1.0 | Error handling |

**Note on hecs:** Version 0.11.0 was released Jan 10, 2026 with breaking changes (query iterators no longer yield `(Entity, Q::Item)` by default; `Entity` now implements `Query`). Stay on 0.10.x for this milestone -- upgrade is orthogonal to AI work and would touch every system query. [Confidence: HIGH, verified via GitHub CHANGELOG]

---

## Recommended Stack Additions

### Tier 1: Core AI Architecture (Build from Scratch)

These are algorithms and patterns to implement directly in the codebase, not external crates. This is the correct approach because:
- Ant colony AI is domain-specific; generic AI libraries add abstraction without value
- The scoring logic is simple enough that a library dependency is overhead
- Full control over tuning parameters is essential for emergent behavior
- The project uses hecs (not Bevy), and most Rust AI crates are Bevy-coupled

#### 1. Utility AI Scoring System

| Aspect | Detail |
|--------|--------|
| **What** | Score-based decision system where each ant evaluates possible actions and picks the highest-scoring one |
| **Why** | Produces naturally varied, context-sensitive behavior without explicit if/else trees. Ants in different situations will organically choose different actions based on their perception of the world. This is the mechanism that generates emergent behavior. |
| **Why not Behavior Trees** | BTs encode designer intent ("do X then Y"); utility AI lets the *situation* drive decisions. For emergent colony behavior, you want ants to surprise you, not follow a script. BTs are better for boss AI in RPGs where you want predictable phases. |
| **Why not GOAP** | GOAP requires goal decomposition and action planning -- computationally expensive per-agent and overkill for ant-scale decisions. Ants make simple moment-to-moment choices, not multi-step plans. GOAP shines for complex NPCs with inventory, not 500+ simple agents. |
| **Why not big-brain crate** | big-brain 0.22.0 is Bevy-only (requires Bevy 0.15). Project uses hecs. Porting would mean pulling in the entire Bevy ecosystem. |
| **Why not neural nets / ML** | Massive overkill. Training infrastructure, non-deterministic debugging, impossible to tune by hand. Utility curves give you the same emergent feel with full designer control. |
| **Confidence** | HIGH -- Utility AI is the established standard for this class of problem (Game AI Pro, GDC talks, The Sims, Zoo Tycoon 2) |

**Implementation pattern:**

```
Consideration (0.0..1.0) = ResponseCurve(normalized_input)
Action Score = consideration_1 * consideration_2 * ... * consideration_n
Best Action = max(all action scores) [with optional randomization threshold]
```

**Response curve types to implement:**
- **Linear:** `y = mx + b` -- distance to food, basic proximity
- **Quadratic:** `y = x^2` or `y = 1-x^2` -- diminishing/increasing returns
- **Logistic/Sigmoid:** `y = 1 / (1 + e^(-k*(x-x0)))` -- sharp thresholds (hunger critical, danger close)
- **Inverse:** `y = 1/x` -- urgency that spikes at low values (low energy)

These are just `f32 -> f32` functions. No crate needed.

#### 2. Stigmergy / Pheromone Grid System

| Aspect | Detail |
|--------|--------|
| **What** | A 2D grid overlay storing pheromone concentrations that diffuse and decay over time |
| **Why** | Stigmergy (indirect communication through environment modification) is THE mechanism that produces emergent colony intelligence in real ants. Without it, you just have independent agents. With it, individual decisions aggregate into colony-level strategy. |
| **Algorithm** | Each tick: (1) ants deposit pheromone at current position, (2) pheromone diffuses to neighboring cells (3x3 kernel averaging), (3) pheromone decays by constant factor (e.g., `concentration *= 0.995`). |
| **Multiple channels** | Use separate grids for different pheromone types: FOOD_TRAIL, HOME_TRAIL, DANGER, RECRUITMENT. This mirrors real ant chemistry where different pheromones trigger different behaviors. |
| **Data structure** | `Vec<f32>` with `width * height` elements, indexed as `y * width + x`. One vec per pheromone type. No ndarray needed -- flat array with manual indexing is faster for this access pattern and avoids a dependency. |
| **Confidence** | HIGH -- Stigmergy is not optional for emergent ant behavior. This is the foundational mechanism backed by decades of entomology and ABM research. |

**Key tuning parameters:**
- Deposition rate (how much pheromone an ant drops)
- Diffusion rate (how fast pheromone spreads to neighbors)
- Evaporation/decay rate (how fast pheromone fades)
- Sensitivity threshold (minimum concentration an ant can detect)

#### 3. Finite State Machine via ECS Components

| Aspect | Detail |
|--------|--------|
| **What** | Ant behavioral state (Idle, Foraging, Returning, Nursing, Guarding) represented as an enum component in hecs |
| **Why** | State determines which utility scorers are active. A foraging ant evaluates "follow food trail" and "explore randomly"; a returning ant evaluates "follow home trail" and "drop food at nest". States are the coarse-grained behavioral mode; utility AI is the fine-grained decision within each state. |
| **Why not statig crate** | statig 0.4.1 is a hierarchical state machine with derive macros -- powerful but over-engineered for this. Ant states are a flat enum with simple transitions. A `match` statement on an enum component is clearer, more debuggable, and idiomatic hecs. |
| **Confidence** | HIGH -- FSM + Utility AI hybrid is a well-documented pattern in game AI literature |

**Pattern:**
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum AntState {
    Idle,
    Foraging,
    Returning,
    Nursing,
    Guarding,
    Recruiting,
}
```
Each state has its own set of utility considerations. State transitions are themselves scored actions (e.g., "should I switch from Idle to Foraging?" is a utility evaluation).

---

### Tier 2: Supporting Infrastructure (Crates to Add)

| Library | Version | Purpose | Why This One | Confidence |
|---------|---------|---------|-------------|------------|
| `ordered-float` | 5.1.0 | Sortable/comparable f32 for scoring | Utility scores need `Ord` for max selection. `ordered-float` wraps f32 with total ordering (NaN handling). Tiny dependency, widely used (Sep 2025 release). Alternative: manual `f32::total_cmp` -- but `OrderedFloat` is cleaner in collections. | HIGH -- verified Sep 2025 release on lib.rs |
| `fastrand` | 2.3.0 (upgrade from 2.0) | Weighted random selection, exploration noise | Already a dependency. Latest is 2.3.0. Upgrade is non-breaking (same major). Provides `f32()` for random floats, sufficient for exploration randomization in utility AI. No need for `rand` crate's heavier distribution machinery. | HIGH -- verified on lib.rs |

**Why no spatial indexing crate:**

The simulation runs in a terminal grid. Ant positions are grid cells (integer coordinates). Proximity queries ("what's near me?") are just checking adjacent cells in the pheromone grid arrays -- O(1) lookups. There is no continuous 2D space requiring kd-trees or spatial hashing. The pheromone grid IS the spatial index.

- `kiddo` 5.2.4 -- Overkill. kd-trees are for continuous-space nearest-neighbor queries with floating-point coordinates. Grid cells don't need this.
- `flat_spatial` 0.6.1 -- Same reasoning. Designed for continuous 2D space, not grid cells.
- `rstar` -- R*-trees for geometric queries. Not applicable.

**Why no ndarray:**

`ndarray` 0.17.2 is a powerful n-dimensional array library, but the pheromone grid is just a `Vec<f32>` with width*height elements. Adding ndarray for 2D indexing pulls in a significant dependency for something that's a one-liner: `grid[y * width + x]`. The diffusion kernel is a simple 3x3 neighbor average -- no matrix operations needed.

**Why no krABMaga:**

krABMaga 0.5.3 is a full ABM framework with its own simulation loop, visualization (Bevy-based), and agent model. AntTrails already has all of this via hecs + ratatui. Adopting krABMaga would mean rewriting the entire architecture to fit its framework conventions. It solves a problem AntTrails doesn't have.

---

### Tier 3: Development/Tuning Tools (Dev Dependencies)

| Library | Version | Purpose | When to Add | Confidence |
|---------|---------|---------|-------------|------------|
| `criterion` | 0.5.1 | Benchmark AI tick performance | When optimizing. Not needed initially. Ensures 500+ ants at 30+ FPS in terminal. | MEDIUM -- version from lib.rs, may have newer |
| `tracing` | 0.1.x | Structured logging for AI decisions | When debugging emergent behavior. Add `tracing::debug!` to utility scoring to understand why ants make specific choices. | HIGH -- standard Rust ecosystem tool |

---

## Algorithms Reference

### Algorithm 1: Utility AI Decision Loop

**Per-ant, per-tick:**
1. Gather sensor inputs (nearby pheromone concentrations, distance to nest, energy level, carrying state, nearby ants)
2. For each possible action in current state:
   a. Evaluate each consideration using response curves
   b. Multiply considerations together for final action score
   c. If any consideration scores 0, skip remaining (early-out optimization)
3. Select action: either highest score (greedy) or weighted random among top N (adds behavioral variety)
4. Execute selected action (move, pick up, drop, deposit pheromone, change state)

**Complexity:** O(ants * actions * considerations) per tick. With ~500 ants, ~6 actions, ~4 considerations = 12,000 float multiplications per tick. Trivial.

### Algorithm 2: Pheromone Diffusion + Decay

**Per-tick, per-pheromone-type:**
1. For each cell in grid:
   a. Average the 8 neighbors + self (3x3 kernel), weighted: self gets higher weight to prevent instant dissipation
   b. Multiply result by decay factor (e.g., 0.995)
2. Write results to swap buffer (double-buffer to avoid read-during-write artifacts)
3. Swap buffers

**Optimization:** Run diffusion every N ticks instead of every tick if performance is tight. Pheromone dynamics don't need per-frame precision.

**Complexity:** O(width * height) per pheromone type per tick. For a 200x60 terminal grid = 12,000 cells * 4 pheromone types = 48,000 operations. Trivial.

### Algorithm 3: Ant Specialization via Age + Experience

**Emergent role differentiation without explicit assignment:**
1. Each ant has an `age` counter (incremented each tick) and `experience` counters per activity type
2. Young ants start Idle/Nursing (low exploration drive in utility scoring)
3. As age increases, Foraging considerations score higher (mimics real ant temporal polyethism)
4. Ants that successfully find food get `foraging_experience` incremented, which boosts their Foraging utility scores further (positive feedback = specialization)
5. If colony is under threat (danger pheromone detected), Guard considerations spike for experienced ants near the nest

**Result:** Specialization emerges from the scoring system without any role-assignment logic. The colony develops nurses, foragers, and guards organically.

### Algorithm 4: Recruitment via Pheromone Intensity

**Colony-level strategy from individual behavior:**
1. When an ant finds a rich food source, it deposits extra RECRUITMENT pheromone on return
2. Idle ants near RECRUITMENT pheromone get a boosted "Follow Recruitment" consideration
3. Multiple ants converge on the rich source, depositing more trail pheromone
4. Positive feedback loop creates an "ant highway" to valuable resources
5. When source depletes, pheromone decays naturally, highway dissolves

**This is the core emergent intelligence mechanism.** Individual ants have no concept of "colony strategy" -- they just follow scored utilities -- but the colony collectively allocates foraging effort proportional to resource value.

---

## What NOT to Use (and Why)

| Technology | Why Not |
|------------|---------|
| **Bevy** | Project uses hecs + ratatui. Bevy is a full game engine. Migrating would rewrite the entire project for zero benefit in a terminal app. |
| **big-brain** (0.22.0) | Bevy-only. Cannot use with hecs. |
| **bevy_observed_utility** | Bevy-only. Same problem. |
| **bonsai-bt** (0.10.0) | Behavior trees encode rigid sequences. Ant behavior should be fluid and context-driven, not scripted. BTs fight against emergence. |
| **behavior-tree-lite** (0.3.2) | Same BT limitation. Also has deprecated serde_yaml dependency. |
| **krABMaga** (0.5.3) | Full ABM framework with own simulation loop. Would require rewriting AntTrails' architecture. We already have ECS + rendering. |
| **Neural networks / ML** | Cannot hand-tune, cannot debug, cannot explain "why did that ant do that?". Training infrastructure is massive overhead. Utility curves give the same emergent feel with full transparency. |
| **GOAP** | Computational cost per agent is high (action graph search). With 500+ ants, this blows the frame budget. GOAP is for complex NPCs with multi-step plans, not simple agents. |
| **ndarray** (0.17.2) | Pheromone grids are simple flat arrays. ndarray adds complexity and a dependency for `grid[y * width + x]`. |
| **kiddo** (5.2.4) / spatial trees | Grid-based simulation doesn't need continuous-space spatial indexing. The grid IS the spatial index. |

---

## Installation

Only one new dependency is recommended. The rest is hand-rolled code.

```toml
# Cargo.toml additions for emergent AI milestone
[dependencies]
ordered-float = "5.1"    # Total ordering for f32 utility scores
# fastrand already present -- consider bumping to "2.3" (non-breaking)

[dev-dependencies]
# criterion = "0.5"      # Add later when benchmarking AI performance
# tracing = "0.1"        # Add later when debugging emergent behavior
```

**Total new runtime dependencies: 1** (`ordered-float`)

This minimal footprint is intentional. Emergent AI in ant simulations comes from well-tuned algorithms, not from library features. The value is in the scoring curves, pheromone dynamics, and feedback loops -- all of which are 50-200 lines of domain-specific Rust code that would be harder to write against a generic library API.

---

## Architecture Implications

The AI system slots into the existing ECS architecture as new components and systems:

**New Components:**
- `AntState` (enum: Idle, Foraging, Returning, Nursing, Guarding, Recruiting)
- `AntNeeds` (struct: hunger, energy, age, experience counters)
- `Carrying` (optional: what the ant is holding)
- `SensorRange` (how far ant can detect pheromones)

**New Resources (world-level, not per-entity):**
- `PheromoneGrid` (struct containing multiple `Vec<f32>` layers)
- `ColonyStats` (aggregate data: total food, ant count, threat level)

**New Systems (functions that query the hecs World):**
1. `system_pheromone_diffusion` -- update pheromone grids (runs first)
2. `system_ant_perception` -- read pheromones and neighbors into per-ant sensor data
3. `system_ant_decision` -- run utility scoring, select actions
4. `system_ant_action` -- execute chosen actions (move, pick up, drop, deposit pheromone)
5. `system_ant_state_transition` -- evaluate state change utilities
6. `system_pheromone_deposit` -- ants mark pheromones based on actions taken

System ordering matters: perception before decision, decision before action, action before pheromone deposit.

---

## Sources

### Verified (HIGH confidence)
- hecs crate: https://lib.rs/crates/hecs -- v0.11.0 (Jan 2026), v0.10.5 (May 2024)
- hecs CHANGELOG: https://github.com/Ralith/hecs/blob/master/CHANGELOG.md -- 0.11 breaking changes
- bonsai-bt: https://lib.rs/crates/bonsai-bt -- v0.10.0 (Mar 2025), standalone, no engine dependency
- ordered-float: https://lib.rs/crates/ordered-float -- v5.1.0 (Sep 2025)
- fastrand: https://docs.rs/crate/fastrand/latest -- v2.3.0
- kiddo: https://lib.rs/crates/kiddo -- v5.2.4 (Jan 2026)
- big-brain: https://lib.rs/crates/big-brain -- v0.22.0 (Nov 2024), Bevy 0.15 required
- statig: https://lib.rs/crates/statig -- v0.4.1 (Jul 2025), standalone with optional bevy_ecs
- krABMaga: https://lib.rs/crates/krabmaga -- v0.5.3 (Dec 2025)
- ndarray: https://lib.rs/crates/ndarray -- v0.17.2 (Jan 2026)
- flat_spatial: https://lib.rs/crates/flat_spatial -- v0.6.1 (Jul 2024)

### Verified (MEDIUM confidence)
- Utility AI architecture: https://shaggydev.com/2023/04/19/utility-ai/ -- scoring patterns, response curves, bucketing
- Game AI comparison: https://www.davideaversa.it/blog/choosing-behavior-tree-goap-planning/ -- BT vs GOAP tradeoffs
- Ant colony simulation patterns: https://practicingruby.com/articles/ant-colony-simulation -- FSM + pheromone architecture
- Stigmergy research: https://www.frontiersin.org/journals/behavioral-neuroscience/articles/10.3389/fnbeh.2021.647732/full -- Active Inferants framework
- Game AI Pro Chapter 9: http://www.gameaipro.com/GameAIPro/GameAIPro_Chapter09_An_Introduction_to_Utility_Theory.pdf -- Utility theory reference

### Background (LOW confidence -- informed thinking but not prescriptive)
- krABMaga ABM patterns: https://krabmaga.github.io/
- Are We Game Yet AI: https://arewegameyet.rs/ecosystem/ai/
- Collective stigmergic optimization: https://medium.com/@jsmith0475/collective-stigmergic-optimization-leveraging-ant-colony-emergent-properties-for-multi-agent-ai-55fa5e80456a
