# Phase 2: Pheromone Communication - Research

**Researched:** 2026-02-07
**Domain:** Stigmergic pheromone systems, adaptive deposit/decay, gradient-based navigation, terminal trail visualization (Rust / ratatui)
**Confidence:** HIGH

## Summary

Phase 2 fixes the mathematically broken pheromone system so that visible, meaningful trails form between food sources and colony nests. The current system has a catastrophic deposit-to-decay imbalance: ants deposit 0.05 pheromone per tick while decay removes only 0.1% per event (once every 10 ticks). A single ant saturates any cell to maximum in under 20 ticks, and an abandoned trail takes ~69,000 ticks (~38 minutes at 30 FPS) to fully fade. Every active cell floods to `MAX_PHEROMONE = 1.0`, destroying all gradients. The `get_gradient()` function returns `None` when all neighbors are at 1.0, so pheromone-following degenerates to random movement even when trails exist.

The fix requires four coordinated changes: (1) dramatically increase the decay rate and make it per-tick instead of every-10-ticks, (2) make deposit amounts adaptive based on current cell concentration, (3) add pheromone diffusion so gradients spread spatially, and (4) implement trail visualization so the user can see pheromone paths forming. Different pheromone types (Food, Home, Danger) need different decay rates appropriate to their purpose.

**Primary recommendation:** Use multiplicative exponential decay at 2-5% per tick (not 0.1% per 10 ticks), adaptive deposit rates that decrease as cell concentration rises, per-tick diffusion to 8 neighbors at ~5% rate, and background-color-based trail rendering in ratatui using `Color::Rgb` for intensity gradients.

## Standard Stack

### Core

No new dependencies required. All fixes are internal changes to existing Rust code.

| Library | Version | Purpose | Relevance to Phase 2 |
|---------|---------|---------|----------------------|
| ratatui | 0.29 | Terminal rendering | Background color on cells for pheromone visualization. Already supports `Color::Rgb` for 24-bit true color and `Style::default().bg(color)` for background coloring. |
| fastrand | 2.0 | RNG | Used for stochastic deposit decisions and noise in gradient following to prevent lock-step movement. |
| hecs | 0.10 | ECS framework | Ant queries for deposit logic. No changes to hecs usage. |

### Supporting

None required. The pheromone grid, diffusion, and adaptive deposit are all simple numerical operations on the existing `PheromoneGrid` data structure.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled adaptive deposit | ACO library (e.g., `aco-rs`) | ACO libraries solve optimization problems (TSP, routing). We need real-time simulation of continuous pheromone fields on a grid. Different domain -- ACO does batch pheromone updates after complete tours, we need per-tick incremental updates. |
| Linear diffusion | Gaussian blur kernel | Overkill. A simple 8-neighbor spread at fixed rate is sufficient for tile-based grids and runs in O(W*H) per tick. Gaussian convolution adds complexity with no visual benefit at this resolution. |
| Per-tick decay | Batched decay every N ticks | Current batched approach (every 10 ticks) causes pheromone to accumulate between decay events, contributing to saturation. Per-tick decay is simpler to reason about and produces smoother gradients. |

## Architecture Patterns

### Recommended Changes to Project Structure

```
src/
  systems/
    pheromone.rs    # MAJOR REWRITE: New constants, adaptive deposit, per-tick decay,
                    #   diffusion system, per-type decay rates
  render.rs         # MODIFY: Add pheromone trail visualization layer
  app.rs            # MODIFY: Update system call order (remove tick%10 gate on decay,
                    #   add diffusion system call)
```

### Pattern 1: Multiplicative Exponential Decay (Per-Tick)

**What:** Replace the current `v *= (1.0 - 0.001)` every-10-ticks approach with per-tick decay using type-specific rates. Each tick, every cell's pheromone is multiplied by `(1.0 - decay_rate)`.

**When to use:** Every tick, for all pheromone data, before deposits are applied.

**Why this works:** Multiplicative decay creates natural exponential falloff. A cell at value `v` after `t` ticks of no new deposits has value `v * (1.0 - rate)^t`. With rate=0.02, a cell at 1.0 drops to 0.5 in 34 ticks (~1.1 seconds), to 0.1 in 114 ticks (~3.8 seconds), and to 0.001 (snap-to-zero threshold) in 345 ticks (~11.5 seconds). This creates visible trail aging: active trails stay bright, recently-abandoned trails visibly fade, and old trails vanish.

**Recommended decay rates by type:**

| Pheromone Type | Decay Rate/Tick | Half-Life (ticks) | Half-Life (seconds @30fps) | Rationale |
|----------------|-----------------|-------------------|---------------------------|-----------|
| Food | 0.02 | ~34 | ~1.1s | Food trails should persist while foraging is active but fade within seconds when abandoned. Ants need to quickly redirect when food is depleted. |
| Home | 0.005 | ~138 | ~4.6s | Home trails should persist longer since nest location is stable. Provides reliable navigation backbone. |
| Danger | 0.05 | ~14 | ~0.5s | Danger should fade fast. Combat is transient -- lingering danger signals cause false alarms and permanent flee behavior. |

**Code structure:**
```rust
// In PheromoneGrid:
const DECAY_FOOD: f32 = 0.02;
const DECAY_HOME: f32 = 0.005;
const DECAY_DANGER: f32 = 0.05;
const SNAP_TO_ZERO: f32 = 0.001;

pub fn decay_all(&mut self) {
    // Iterate in strides of 3 (food, home, danger per colony per cell)
    for chunk in self.data.chunks_exact_mut(3) {
        chunk[0] *= 1.0 - DECAY_FOOD;
        if chunk[0] < SNAP_TO_ZERO { chunk[0] = 0.0; }
        chunk[1] *= 1.0 - DECAY_HOME;
        if chunk[1] < SNAP_TO_ZERO { chunk[1] = 0.0; }
        chunk[2] *= 1.0 - DECAY_DANGER;
        if chunk[2] < SNAP_TO_ZERO { chunk[2] = 0.0; }
    }
}
```

### Pattern 2: Adaptive Deposit Rate

**What:** Deposit amount decreases as current cell concentration increases. Uses the formula: `effective_deposit = base_deposit * (1.0 - current_value / MAX_PHEROMONE)`. When a cell is near empty, full deposit rate applies. When near max, almost nothing is deposited.

**When to use:** In `pheromone_deposit_system()`, replacing the flat `DEPOSIT_AMOUNT`.

**Why this works:** This prevents saturation naturally. A high-traffic cell stabilizes at a concentration where deposit and decay are balanced, rather than slamming into `MAX_PHEROMONE`. Cells with different traffic levels reach different equilibria, creating the gradient that `get_gradient()` needs to function.

**Equilibrium analysis with adaptive deposit:**
- Let `d` = base deposit rate per tick, `r` = decay rate, `v` = current value
- Per-tick change: `delta = d * (1 - v) - r * v`
- Equilibrium (`delta = 0`): `v_eq = d / (d + r)`
- With `d = 0.03` (base deposit) and `r = 0.02` (food decay): `v_eq = 0.03 / 0.05 = 0.6`
- With 3 ants on same cell: effectively `d = 0.09`: `v_eq = 0.09 / 0.11 = 0.82`
- Single-ant cell: 0.6 vs multi-ant cell: 0.82 -- meaningful gradient exists!

**Recommended base deposit rates:**

| Context | Base Deposit | Rationale |
|---------|-------------|-----------|
| Wandering/Digging (Home pheromone) | 0.03 | Lower rate since home pheromone also decays slower |
| Carrying (Food pheromone) | 0.05 | Slightly higher to establish strong food trails |
| Combat (Danger pheromone) | 0.10 | High deposit since danger decays very fast |

**Code structure:**
```rust
pub fn deposit_adaptive(
    &mut self, x: i32, y: i32, colony: u8,
    ptype: PheromoneType, base_amount: f32
) {
    if let Some(i) = self.index(x, y, colony, ptype) {
        let current = self.data[i];
        let effective = base_amount * (1.0 - current / MAX_PHEROMONE);
        self.data[i] = (current + effective).min(MAX_PHEROMONE);
    }
}
```

### Pattern 3: Pheromone Diffusion (Spatial Spreading)

**What:** Each tick, a small fraction of each cell's pheromone spreads to its 8 neighbors. This creates smooth gradients radiating outward from trails rather than sharp on/off boundaries.

**When to use:** After decay, before deposits. Run every tick.

**Why this is critical:** Without diffusion, pheromone exists only on the exact cells where ants have walked. An ant one tile away from a trail sees zero pheromone and `get_gradient()` returns `None`. Diffusion creates a "scent cloud" around trails that ants can detect from several tiles away, making gradient following actually work.

**Diffusion parameters:**

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Diffusion rate | 0.05 (5%) | Each cell spreads 5% of its value to neighbors per tick. Low enough to preserve trail sharpness, high enough to create detectable gradients 3-5 tiles wide. |
| Cardinal weight | 1.0 | Direct neighbors (N/S/E/W) receive full share |
| Diagonal weight | 0.707 (~1/sqrt(2)) | Diagonal neighbors receive reduced share proportional to distance |
| Total spread per tick | ~5% * (4 * 1.0 + 4 * 0.707) / normalization | Conservation: cell loses what it spreads, neighbors gain proportionally |

**Implementation approach:** Use a double-buffer (swap buffers each tick) to avoid read-write conflicts. The PheromoneGrid already has a flat `Vec<f32>` data array. Allocate a second buffer of the same size and swap after diffusion pass.

**Performance consideration:** Diffusion iterates the full grid (200 * 100 * 3 colonies * 3 types = 180,000 values). This is a tight loop of multiply-add operations -- trivially fast on modern CPUs. The double buffer adds ~700KB memory (180,000 * 4 bytes). Acceptable.

**Code structure:**
```rust
pub fn diffuse(&mut self) {
    // Use temp buffer to avoid read-write conflicts
    let mut next = vec![0.0f32; self.data.len()];
    let diffusion_rate = 0.05;
    let cardinal = 1.0;
    let diagonal = 0.707;
    let total_weight = 4.0 * cardinal + 4.0 * diagonal;  // ~6.828

    for y in 0..self.height as i32 {
        for x in 0..self.width as i32 {
            for colony in 0..self.max_colonies as u8 {
                for ptype in [PheromoneType::Food, PheromoneType::Home, PheromoneType::Danger] {
                    if let Some(i) = self.index(x, y, colony, ptype) {
                        let val = self.data[i];
                        if val < SNAP_TO_ZERO { continue; }

                        let spread = val * diffusion_rate;
                        next[i] += val - spread;  // Cell keeps most of its value

                        // Spread to neighbors
                        for (dx, dy) in DIRECTIONS_8 {
                            if let Some(ni) = self.index(x + dx, y + dy, colony, ptype) {
                                let weight = if dx.abs() + dy.abs() == 1 {
                                    cardinal
                                } else {
                                    diagonal
                                };
                                next[ni] += spread * weight / total_weight;
                            }
                        }
                    }
                }
            }
        }
    }
    self.data = next;
}
```

**Optimization:** Skip cells with value < SNAP_TO_ZERO (most cells will be zero). This makes diffusion cost proportional to the number of non-zero cells rather than total grid size.

### Pattern 4: Improved Gradient Following with Stochastic Selection

**What:** Replace the current greedy "pick strongest neighbor" gradient following with a weighted-probability selection that biases toward stronger signals but allows some randomness.

**When to use:** In `get_gradient()` and `follow_pheromone()`.

**Why:** The current `get_gradient()` always picks the single strongest neighbor. This causes all ants to follow exactly the same path (lock-step), which looks unnatural and creates congestion. Adding stochastic selection based on pheromone weights creates more natural-looking trails where ants generally follow the gradient but spread out slightly.

**Also fixes a gradient bug:** The current `get_gradient()` initializes `best_strength` to the ant's current cell value (line 100). This means the function only returns a direction if a neighbor is STRONGER than the current position. When cells are saturated to max, no neighbor is stronger, and gradient following fails silently. The fix: initialize `best_strength` to 0.0 or use weighted random selection across all neighbors with non-zero pheromone.

**Code structure:**
```rust
pub fn get_gradient_weighted(
    &self, x: i32, y: i32, colony: u8, ptype: PheromoneType,
) -> Option<(i32, i32)> {
    let directions = [
        (0, -1), (0, 1), (-1, 0), (1, 0),
        (-1, -1), (1, -1), (-1, 1), (1, 1),
    ];

    // Collect all neighbor strengths
    let mut candidates: Vec<((i32, i32), f32)> = Vec::new();
    let current = self.get(x, y, colony, ptype);

    for (dx, dy) in directions {
        let strength = self.get(x + dx, y + dy, colony, ptype);
        if strength > 0.01 {  // Ignore negligible pheromone
            candidates.push(((dx, dy), strength));
        }
    }

    if candidates.is_empty() {
        return None;
    }

    // Weighted random selection: probability proportional to strength^2
    // (squaring emphasizes stronger trails)
    let total: f32 = candidates.iter().map(|(_, s)| s * s).sum();
    let mut roll = fastrand::f32() * total;
    for ((dx, dy), s) in &candidates {
        roll -= s * s;
        if roll <= 0.0 {
            return Some((*dx, *dy));
        }
    }

    // Fallback to last candidate
    candidates.last().map(|((dx, dy), _)| (*dx, *dy))
}
```

### Pattern 5: Pheromone Trail Visualization (Terminal Rendering)

**What:** Render pheromone trails as background colors on terrain cells in ratatui. Trail intensity maps to color brightness.

**When to use:** In `render_terrain()` in render.rs, as a layer between terrain rendering and entity rendering.

**Why this is a success criterion:** "Visible trails form between food sources and colony nests." Without visualization, the user cannot see trails at all.

**Ratatui capabilities:**
- `Style::default().bg(Color::Rgb(r, g, b))` sets background color on any cell
- `frame.buffer_mut().set_string(x, y, ch, style)` already used in render.rs for terrain
- The existing render loop checks entity positions first, then terrain. Pheromone visualization should modify the terrain cell's background color when pheromone is present.

**Color mapping:**

| Pheromone Type | Color | Low Intensity | High Intensity |
|----------------|-------|---------------|----------------|
| Food | Green | Rgb(0, 30, 0) | Rgb(0, 120, 0) |
| Home | Blue | Rgb(0, 0, 30) | Rgb(0, 0, 120) |
| Danger | Red | Rgb(30, 0, 0) | Rgb(120, 0, 0) |

**Blending multiple pheromone types:** When food and home pheromone overlap on the same cell, combine RGB channels: `(food_r + home_r, food_g + home_g, food_b + home_b)` clamped to 255. Food is green-channel, home is blue-channel, danger is red-channel, so they map cleanly to RGB without conflicts.

**Colony-specific vs. combined display:** The simplest approach is to render the MAXIMUM pheromone across all colonies for each type. This means the user sees all trails regardless of which colony laid them. Colony-specific visualization could be a future enhancement toggled by keypress.

**Minimum visibility threshold:** Only render pheromone background when value > 0.05. Below this, the trail is too faint to be meaningful and would add visual noise.

**Code structure in render.rs:**
```rust
// After terrain rendering, before entity rendering:
// Check for pheromone at this position
let mut bg_r: u8 = 0;
let mut bg_g: u8 = 0;
let mut bg_b: u8 = 0;

for colony in 0..num_colonies {
    let food = pheromones.get(world_x, world_y, colony as u8, PheromoneType::Food);
    let home = pheromones.get(world_x, world_y, colony as u8, PheromoneType::Home);
    let danger = pheromones.get(world_x, world_y, colony as u8, PheromoneType::Danger);

    // Map intensity (0.0-1.0) to color (0-120)
    bg_g = bg_g.max((food.clamp(0.0, 1.0) * 120.0) as u8);
    bg_b = bg_b.max((home.clamp(0.0, 1.0) * 120.0) as u8);
    bg_r = bg_r.max((danger.clamp(0.0, 1.0) * 120.0) as u8);
}

if bg_r > 5 || bg_g > 5 || bg_b > 5 {
    let style = Style::default()
        .fg(terrain_color)
        .bg(Color::Rgb(bg_r, bg_g, bg_b));
    frame.buffer_mut().set_string(x, y, ch.to_string(), style);
} else {
    // Normal terrain rendering (existing code)
}
```

### Anti-Patterns to Avoid

- **Flat deposit rate with clamping to MAX:** This is the current broken approach. Any flat deposit rate with multiplicative decay will saturate given enough ant traffic. Use adaptive deposit.
- **Batched decay (every N ticks):** Creates sawtooth pheromone levels and makes the deposit/decay ratio harder to balance. Decay every tick for smooth curves.
- **Greedy gradient following (always pick strongest):** Causes lock-step ant movement and fails under saturation. Use weighted random selection.
- **Grid-wide diffusion without skip-zero optimization:** Iterating 180K cells when 95% are zero wastes cycles. Skip cells below threshold.
- **Pheromone rendering that overwrites entity display:** Entities (ants, food) must always render on top of pheromone backgrounds. The render order must be: terrain base -> pheromone background -> water -> entities.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pheromone decay | Fixed-interval batch decay | Per-tick multiplicative decay (simple multiply loop) | Already one line of code per element. The existing `decay_all()` just needs rate adjustment and per-type rates. |
| Trail pathfinding | A* or Dijkstra to follow trails | Gradient ascent via `get_gradient()` | Ant-like behavior IS greedy local gradient following. Real ants don't compute global paths -- they follow the strongest local signal. |
| Color interpolation for visualization | Custom color lerp library | Direct Rgb math with `.clamp()` | Three multiplications and a clamp. No library needed. |
| Pheromone field data structure | Sparse map / hashmap | Flat `Vec<f32>` (existing) | The grid is only 200x100x3x3 = 180K floats = 720KB. Dense array is cache-friendly and trivially indexable. Sparse would add overhead for no benefit. |

**Key insight:** The pheromone system does not need new data structures or external libraries. It needs correct PARAMETERS (deposit rate, decay rate) and two new OPERATIONS (adaptive deposit, diffusion) on the existing flat array.

## Common Pitfalls

### Pitfall 1: Deposit/Decay Ratio Still Wrong After "Fixing"

**What goes wrong:** You increase the decay rate but leave the deposit rate unchanged, or vice versa. The system still saturates, just more slowly. Or the decay is too aggressive and trails vanish before ants can follow them.

**Why it happens:** Deposit and decay form a coupled system. Changing one without the other shifts the equilibrium but does not fix the fundamental imbalance.

**How to avoid:** Use the equilibrium formula: `v_eq = d / (d + r)` where `d` = effective deposit rate and `r` = decay rate. Target equilibrium of 0.5-0.7 for single-ant trails. Verify by running the simulation for 200 ticks and checking that cells near active trails are NOT at 1.0 and cells 5+ tiles from trails are at or near 0.

**Warning signs:** All pheromone cells at 1.0 (saturation persists) or all at 0.0 (over-decay). Run `pheromones.data.iter().filter(|v| **v > 0.9).count()` as a debug diagnostic.

### Pitfall 2: Diffusion Creates Uniform Background Instead of Trails

**What goes wrong:** Diffusion rate too high (>10%) causes pheromone to spread uniformly across the map, erasing the trail structure. Everything becomes a faint haze rather than defined paths.

**Why it happens:** High diffusion rate means pheromone spreads faster than ants walk, so the "trail" widens to fill available space. Combined with slow decay, the entire reachable area fills to a low uniform level.

**How to avoid:** Keep diffusion rate at 3-5%. The trail should be visible 3-5 tiles wide around the actual ant path, not 20+ tiles. Verify by checking that cells far from any ant (e.g., top corners of the map with no activity) remain at exactly 0.0 after 500 ticks.

**Warning signs:** Pheromone visible everywhere on the map rather than as distinct paths.

### Pitfall 3: Decay Every Tick Breaks Danger Pheromone for Combat

**What goes wrong:** Switching from decay-every-10-ticks to decay-every-tick means danger pheromone now decays 10x faster than before. The combat system deposits danger at 0.5 per combat event (combat.rs:90), but with 5% per-tick decay, that 0.5 drops to 0.25 in just 14 ticks. The `soldier_ai_system` threshold for entering Fighting state is `danger > 0.1` (combat.rs:155). Soldiers may not respond to combat fast enough.

**Why it happens:** Danger pheromone is designed to be high-deposit, fast-decay. But the combat system only deposits during actual combat events (every 5 ticks via `COMBAT_INTERVAL`). Between deposits, rapid decay erodes the signal.

**How to avoid:** This is actually the DESIRED behavior for danger -- fast decay means soldiers stop fighting when combat ends. But verify that the deposit amount (0.5) is high enough to stay above the 0.1 threshold for the ~5 ticks between combat events. With 5% decay: `0.5 * (1-0.05)^5 = 0.39`. Still above 0.1. Safe.

**Warning signs:** Soldiers ignoring nearby combat, flickering between Fighting and Wandering states.

### Pitfall 4: Pheromone Visualization Overwhelms Entity Display

**What goes wrong:** Bright pheromone background colors make ant characters hard to see. Green food trails on green surface terrain become invisible. Blue home trails on dark backgrounds wash out text characters.

**Why it happens:** Background colors compete with foreground character visibility. Terminal cells have limited contrast range.

**How to avoid:** Cap pheromone background RGB at 120 (not 255). This keeps backgrounds dim enough that white/colored foreground characters remain visible. Also: do NOT apply pheromone backgrounds to cells that have entities on them -- entity cells should use the colony color system unchanged.

**Warning signs:** Ants becoming invisible against bright pheromone backgrounds.

### Pitfall 5: Double-Buffer Diffusion Memory Allocation Every Tick

**What goes wrong:** Allocating a new `Vec<f32>` of 180K elements every tick for the diffusion buffer. This hits the allocator 30 times per second for a ~700KB allocation, causing GC pressure and potential frame hitches.

**Why it happens:** Naive implementation creates `let mut next = vec![0.0; self.data.len()]` inside `diffuse()`.

**How to avoid:** Store the diffusion buffer as a permanent second field in `PheromoneGrid`. Swap `self.data` and `self.buffer` after each diffusion pass instead of allocating. Zero the buffer at the start of diffusion, not by reallocating.

**Warning signs:** Frame time spikes during pheromone-heavy scenarios. Memory usage climbing over time (if buffers leak).

### Pitfall 6: Wandering Ants Deposit Home Pheromone Everywhere

**What goes wrong:** Current code deposits home pheromone for ALL Wandering and Digging ants (pheromone.rs:123). If 200 ants are wandering randomly across the surface, home pheromone saturates the entire surface area. There is no useful gradient pointing toward the actual nest.

**Why it happens:** The deposit logic does not consider distance from home. Every wandering ant claims its position is "home-ish" by depositing home pheromone.

**How to avoid:** Only deposit home pheromone when the ant is NEAR the nest (within a configurable radius, e.g., 15 tiles), or scale deposit amount inversely with distance from nest. An ant at the nest deposits full home pheromone; an ant 30 tiles away deposits almost none. This creates a gradient that actually points toward the nest.

**Alternative:** Deposit home pheromone in a "breadcrumb" fashion only while the ant is in Carrying state (returning home with food). This creates a home trail ONLY along successful return paths, which is biologically accurate.

**Warning signs:** Home pheromone visible uniformly across the entire map rather than concentrated near nests.

## Code Examples

### Example 1: Complete Revised PheromoneGrid Constants

```rust
// In pheromone.rs - replace existing constants
const MAX_PHEROMONE: f32 = 1.0;
const SNAP_TO_ZERO: f32 = 0.001;

// Per-type decay rates (per tick)
const DECAY_FOOD: f32 = 0.02;    // Half-life ~34 ticks (~1.1s)
const DECAY_HOME: f32 = 0.005;   // Half-life ~138 ticks (~4.6s)
const DECAY_DANGER: f32 = 0.05;  // Half-life ~14 ticks (~0.5s)

// Base deposit amounts (before adaptive scaling)
const DEPOSIT_HOME_BASE: f32 = 0.03;
const DEPOSIT_FOOD_BASE: f32 = 0.05;
const DEPOSIT_DANGER_BASE: f32 = 0.10;

// Diffusion
const DIFFUSION_RATE: f32 = 0.05;
```

### Example 2: Revised Deposit Logic with Context-Awareness

```rust
pub fn pheromone_deposit_system(world: &World, pheromones: &mut PheromoneGrid, colonies: &[ColonyState]) {
    for (_entity, (pos, ant, member)) in world.query::<(&Position, &Ant, &ColonyMember)>().iter() {
        let colony_id = member.colony_id;

        match ant.state {
            // Carrying ants lay FOOD pheromone (they found food, others should follow)
            AntState::Carrying => {
                pheromones.deposit_adaptive(
                    pos.x, pos.y, colony_id,
                    PheromoneType::Food, DEPOSIT_FOOD_BASE,
                );
            }
            // Ants near home lay HOME pheromone (breadcrumb back to nest)
            AntState::Wandering | AntState::Returning => {
                // Scale home deposit by proximity to nest
                let home_x = colonies.get(colony_id as usize).map(|c| c.home_x).unwrap_or(0);
                let home_y = colonies.get(colony_id as usize).map(|c| c.home_y).unwrap_or(0);
                let dist = ((pos.x - home_x).abs() + (pos.y - home_y).abs()) as f32;
                let proximity_factor = (1.0 - dist / 30.0).max(0.0);  // Fades to 0 at 30 tiles

                if proximity_factor > 0.0 {
                    pheromones.deposit_adaptive(
                        pos.x, pos.y, colony_id,
                        PheromoneType::Home, DEPOSIT_HOME_BASE * proximity_factor,
                    );
                }
            }
            // Digging ants also leave faint home pheromone near nest
            AntState::Digging => {
                let home_x = colonies.get(colony_id as usize).map(|c| c.home_x).unwrap_or(0);
                let home_y = colonies.get(colony_id as usize).map(|c| c.home_y).unwrap_or(0);
                let dist = ((pos.x - home_x).abs() + (pos.y - home_y).abs()) as f32;
                let proximity_factor = (1.0 - dist / 20.0).max(0.0);

                if proximity_factor > 0.0 {
                    pheromones.deposit_adaptive(
                        pos.x, pos.y, colony_id,
                        PheromoneType::Home, DEPOSIT_HOME_BASE * 0.5 * proximity_factor,
                    );
                }
            }
            // Other states don't deposit (Fighting/Fleeing handled by combat system)
            _ => {}
        }
    }
}
```

### Example 3: System Call Order in app.rs Update Loop

```rust
// === Phase 4: Pheromones === (revised)
// 1. Decay first (reduces all values)
systems::pheromone::pheromone_decay_system(&mut self.pheromones);

// 2. Diffuse (spread gradients spatially)
self.pheromones.diffuse();

// 3. Then deposit new pheromone from ant positions
systems::pheromone::pheromone_deposit_system(
    &self.world, &mut self.pheromones, &self.colonies,
);
```

Note: The current `tick % 10 == 0` gate on decay must be REMOVED. Decay now runs every tick.

### Example 4: Rendering Integration (render.rs)

```rust
// In render_terrain(), after computing terrain (ch, color) but before writing to buffer:
// Add pheromone background layer

let mut bg_r: u8 = 0;
let mut bg_g: u8 = 0;
let mut bg_b: u8 = 0;

for c in 0..num_colonies as u8 {
    let food_val = pheromones.get(world_x, world_y, c, PheromoneType::Food);
    let home_val = pheromones.get(world_x, world_y, c, PheromoneType::Home);
    let danger_val = pheromones.get(world_x, world_y, c, PheromoneType::Danger);

    if food_val > 0.05 {
        bg_g = bg_g.max((food_val * 120.0) as u8);
    }
    if home_val > 0.05 {
        bg_b = bg_b.max((home_val * 120.0) as u8);
    }
    if danger_val > 0.05 {
        bg_r = bg_r.max((danger_val * 120.0) as u8);
    }
}

let style = if bg_r > 0 || bg_g > 0 || bg_b > 0 {
    Style::default().fg(color).bg(Color::Rgb(bg_r, bg_g, bg_b))
} else {
    Style::default().fg(color)
};

frame.buffer_mut().set_string(x, y, ch.to_string(), style);
```

### Example 5: Double-Buffered Diffusion in PheromoneGrid

```rust
pub struct PheromoneGrid {
    pub width: usize,
    pub height: usize,
    data: Vec<f32>,
    buffer: Vec<f32>,  // NEW: diffusion scratch buffer
    pub max_colonies: usize,
}

impl PheromoneGrid {
    pub fn new(width: usize, height: usize, max_colonies: usize) -> Self {
        let size = width * height * max_colonies * 3;
        Self {
            width,
            height,
            data: vec![0.0; size],
            buffer: vec![0.0; size],
            max_colonies,
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Flat deposit 0.05/tick | Adaptive deposit: `base * (1.0 - current/max)` | This phase | Prevents saturation; cells reach traffic-proportional equilibrium |
| Decay 0.001 every 10 ticks | Per-tick decay at 0.02/0.005/0.05 by type | This phase | Trails fade in seconds not hours; type-appropriate lifetimes |
| No diffusion | 5% per-tick diffusion to 8 neighbors | This phase | Ants detect trails 3-5 tiles away; gradients become followable |
| Greedy gradient (pick strongest) | Weighted random selection (probability ~ strength^2) | This phase | Natural-looking trail following; reduces lock-step movement |
| No visualization | Background RGB coloring in ratatui | This phase | Users see trail formation -- the core visual feedback |
| Uniform home deposit everywhere | Proximity-scaled home deposit near nest | This phase | Home gradient actually points toward nest |

**Deprecated/outdated:**
- `DECAY_RATE: f32 = 0.001` -- inadequate by ~20-50x for meaningful gradients
- `DEPOSIT_AMOUNT: f32 = 0.05` (flat) -- must be replaced with adaptive deposit
- `tick % 10 == 0` gate on decay in app.rs -- decay must run every tick
- `get_gradient()` initializing `best_strength` to current cell value -- causes failure under saturation

## Mathematical Proof of Current Bug (FIX-03)

### Why the current system is broken

**Given:**
- Deposit per tick: `d = 0.05`
- Decay per event: `r = 0.001` (multiplicative)
- Decay frequency: every 10 ticks
- Max: 1.0

**Scenario: Single ant on a cell for 10 ticks, then leaves**

After 10 ticks of deposit (before first decay event):
- Value = `min(0.05 * 10, 1.0) = 0.50`

After first decay event:
- Value = `0.50 * (1 - 0.001) = 0.4995`

After 10 more ticks of deposit + second decay:
- Value = `min(0.4995 + 0.50, 1.0) = 1.0` (capped)
- After decay: `1.0 * 0.999 = 0.999`

**Time to decay from 1.0 to 0.001 (after ant leaves):**
- `1.0 * (1 - 0.001)^n = 0.001`
- `n = log(0.001) / log(0.999) = -6.908 / -0.001001 = 6,907 decay events`
- At one event per 10 ticks: `69,070 ticks`
- At 30 FPS: **38.4 minutes**

**Conclusion:** A trail takes 38 minutes to vanish. Every cell an ant has ever visited will be at or near maximum for the duration of a typical simulation run. `get_gradient()` sees uniform 1.0 everywhere and returns `None`. Pheromone-guided movement is completely non-functional.

### Proposed system verification

With new parameters (food pheromone):
- Adaptive deposit base: `d = 0.05`
- Decay per tick: `r = 0.02`

**Single ant, steady state:**
- `v_eq = d / (d + r) = 0.05 / 0.07 = 0.71`

**After ant leaves (from v=0.71):**
- Half-life: `ln(2) / 0.02 = 34.7 ticks = 1.16 seconds`
- Time to 0.001: `ln(0.71/0.001) / 0.02 = 329 ticks = 11 seconds`

**Trail of 5 ants vs 1 ant:**
- 5 ants, effective `d = 0.25`: `v_eq = 0.25 / 0.27 = 0.93`
- 1 ant: `v_eq = 0.71`
- Gradient: 0.93 vs 0.71 = 0.22 difference -- meaningful for `get_gradient()` to detect

This is correct behavior: heavy traffic creates stronger trails, gradient following works.

## Open Questions

1. **Should pheromone visualization be toggleable?**
   - What we know: Pheromone backgrounds will change the visual appearance significantly. Some users may prefer to see the terrain clearly without pheromone overlay.
   - What's unclear: Whether this should be a Phase 2 feature or deferred.
   - Recommendation: Add a toggle keypress (e.g., 'P' for pheromone visibility) as a low-effort addition during the visualization task. Implement it, but don't gate Phase 2 completion on it.

2. **What is the correct home pheromone deposit strategy?**
   - What we know: Current approach (all Wandering/Digging ants deposit home pheromone) floods the map. Proximity-based or carrying-only deposit are both viable alternatives.
   - What's unclear: Which produces better emergent behavior. Proximity-based creates a static "homing beacon" around the nest. Carrying-only creates dynamic trails that map actual successful foraging routes.
   - Recommendation: Start with proximity-based (simpler, guaranteed to create a visible gradient around the nest). If emergent behavior is unsatisfying, switch to carrying-only in a later iteration.

3. **Should diffusion respect terrain passability?**
   - What we know: Real pheromones do not diffuse through solid rock. Currently the grid has no concept of terrain blocking.
   - What's unclear: Whether blocking diffusion through impassable terrain meaningfully improves simulation behavior or just adds complexity.
   - Recommendation: Skip terrain-aware diffusion for Phase 2. The performance cost of checking terrain for each of 8 neighbors on every non-zero cell is non-trivial. Additionally, ants only deposit pheromone in passable cells, so pheromone naturally concentrates in tunnels and open areas. Diffusion through a few tiles of rock is a minor visual artifact that does not affect behavior.

4. **Performance impact of per-tick decay + diffusion on full grid?**
   - What we know: 180K float multiply-adds for decay + ~180K for diffusion = ~360K float ops per tick. At 30 FPS = ~10.8M float ops/second.
   - What's unclear: Whether this causes measurable frame time impact on the target hardware.
   - Recommendation: Implement with skip-zero optimization. Profile after implementation. If too slow, run decay+diffusion every 2-3 ticks instead of every tick (still much better than every 10 ticks). On modern CPUs, 10M float ops/second is trivial (< 1ms).

## Sources

### Primary (HIGH confidence)
- Direct codebase analysis of `pheromone.rs` (deposit logic, decay logic, gradient following), `food.rs` (foraging_movement pheromone usage), `movement.rs` (movement dispatch), `combat.rs` (danger pheromone deposit), `app.rs` (system call order), `render.rs` (current rendering approach)
- Mathematical analysis of deposit/decay equilibrium equations (verified analytically)
- ratatui `Color::Rgb` and `Style::bg()` support verified in render.rs existing usage of `Color::Rgb` for terrain and water rendering

### Secondary (MEDIUM confidence)
- [Ant Colony Optimization - Wikipedia](https://en.wikipedia.org/wiki/Ant_colony_optimization_algorithms) -- MMAS tau_min/tau_max clamping, exponential decay formula
- [Ant Colony Optimization overview - ScienceDirect](https://www.sciencedirect.com/topics/engineering/ant-colony-optimization) -- Pheromone deposit/evaporation dynamics
- [Pheromone-Focused ACO (2025) - arXiv](https://arxiv.org/html/2601.07597v1) -- Adaptive pheromone concentration strategies
- [Ratatui Colors documentation](https://ratatui.rs/examples/style/colors/) -- RGB color support verification
- [Ratatui Cell documentation](https://docs.rs/ratatui/latest/ratatui/buffer/struct.Cell.html) -- Cell-level rendering capabilities

### Tertiary (LOW confidence)
- Specific decay rate values (0.02 for food, 0.005 for home, 0.05 for danger) -- these are analytically derived from desired half-lives but need in-simulation tuning. The equilibrium math is correct, but the "right" visual result is subjective.
- Diffusion rate of 5% -- standard in literature for grid-based diffusion but optimal value for this specific simulation is unknown until tested.
- Color intensity cap of 120 for pheromone backgrounds -- chosen to preserve entity visibility, but terminal-specific rendering may need adjustment based on actual terminal color rendering.

## Metadata

**Confidence breakdown:**
- Architecture Patterns: HIGH -- decay/deposit math is analytically provable; double-buffer diffusion is standard; ratatui rendering capabilities verified in existing code
- Bug Analysis (FIX-03): HIGH -- mathematical proof shows current system is broken by 3-4 orders of magnitude
- Adaptive Deposit Formula: HIGH -- equilibrium equation is textbook; verified algebraically
- Specific Parameter Values: MEDIUM -- analytically sound targets but require in-sim validation
- Visualization Design: MEDIUM -- ratatui supports needed features (verified), but exact color values and thresholds need visual tuning
- Performance Impact: MEDIUM -- order-of-magnitude estimate (trivial), but no profiling data yet

**Research date:** 2026-02-07
**Valid until:** 2026-03-07 (stable -- all findings are against existing codebase internals, not external libraries)
