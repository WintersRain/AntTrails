# Feature Landscape: Emergent Ant Colony AI

**Domain:** Ant colony / emergent agent simulation
**Researched:** 2026-02-06
**Overall confidence:** MEDIUM-HIGH (well-studied domain; biology is documented, simulation patterns are proven)

---

## Table Stakes

Features the simulation must have or the colony will feel dead/broken. These are the behaviors any observer expects when they watch "ants doing ant things."

### TS-1: Coherent Foraging Loop

| Aspect | Detail |
|--------|--------|
| **What** | Workers find food, pick it up, carry it home, deposit it, go back out. Visible trail formation between food and nest. |
| **Why expected** | This is THE canonical ant behavior. A sim without visible foraging trails is not an ant sim. Every reference implementation (NetLogo Ants, SimAnt, Empires of the Undergrowth) centers on this. |
| **How real ants do it** | Forager discovers food, fills crop (social stomach), returns to nest depositing pheromone. Trail is reinforced by subsequent ants. Shorter paths get more reinforcement (ants return faster = more pheromone before decay). Evaporation removes trails to depleted sources. |
| **How sims implement it** | Each ant senses pheromone in 3 directions (ahead, ahead-left, ahead-right), probabilistically follows strongest gradient. Carrying ants deposit trail pheromone. Pheromone decays globally each tick. NetLogo uses diffusion-rate + evaporation-rate parameters. |
| **Current state in AntTrails** | BROKEN. `foraging_movement()` in food.rs is orphaned (never called). Movement system returns `(0,0)` for `Carrying` state. Ants pick up food then freeze forever. |
| **Complexity** | **Low** -- the code exists, it just needs to be wired in. Fix the movement system to call foraging_movement for Wandering/Carrying states. |
| **Dependencies** | Requires working pheromone system (exists), movement system (exists but needs Carrying fix) |

### TS-2: Pheromone-Guided Movement (Gradient Following)

| Aspect | Detail |
|--------|--------|
| **What** | Ants probabilistically follow pheromone gradients rather than moving randomly. The "sniff ahead in 3 directions, turn toward strongest" pattern. |
| **Why expected** | Without pheromone following, ants look like random walkers. The trail-formation feedback loop that creates emergent shortest-path finding depends on this. |
| **How real ants do it** | Antennae detect chemical concentration differences. Decision-making follows psychophysical theory: the relationship between stimulus intensity and response probability is sigmoidal, not linear. Ants have individual sensitivity variation. |
| **How sims implement it** | NetLogo: sniff 3 cells ahead, turn toward highest concentration. Add stochastic noise so ants don't perfectly lock onto trails (prevents ant-highway collapse). Key parameters: sensitivity threshold (ignore below 0.05), noise factor, exploration probability. |
| **Current state in AntTrails** | `PheromoneGrid::get_gradient()` exists and works. `follow_pheromone()` exists. But movement_system doesn't call them for Wandering ants -- it just calls `random_movement()`. |
| **Complexity** | **Low** -- integrate existing gradient-following into the movement system with probability weighting |
| **Dependencies** | Pheromone deposit system (exists), pheromone decay (exists) |

### TS-3: Activity Level Tuning (Ants Actually Do Things)

| Aspect | Detail |
|--------|--------|
| **What** | Ants are visibly active most of the time. Workers forage, dig, tend brood. The colony looks alive. |
| **Why expected** | An ant sim where ants mostly sit idle is a broken ant sim. Real ant colonies are bustling. |
| **How real ants do it** | Workers are active 60-80% of the time in young colonies. Even "resting" ants respond to stimuli quickly. Activity correlates with colony needs -- more foraging when food is low, more brood care when eggs are present. |
| **How sims implement it** | High base activity rates (80%+ for workers), with state determining what activity. Idle is a brief transition state, not a default. Task selection each tick, not "maybe do something" rolls. |
| **Current state in AntTrails** | Probabilities are 3-12% per tick (fastrand::u8(..) < 10 is ~4%). This makes ants appear lifeless. |
| **Complexity** | **Low** -- tune constants, restructure tick logic from "maybe act" to "always act, choose what" |
| **Dependencies** | None -- pure parameter tuning |

### TS-4: State Machine Coherence (No Stuck Ants)

| Aspect | Detail |
|--------|--------|
| **What** | Every ant state (Idle, Wandering, Digging, Carrying, Fighting, Following, Fleeing, Returning) has valid transitions and movement behavior. No state is a dead end. |
| **Why expected** | Stuck/frozen ants break immersion instantly. |
| **How real ants do it** | Real ants don't get stuck -- they have fallback behaviors (wander randomly if lost, follow nest-mate if confused). |
| **How sims implement it** | Every state has: (1) a movement strategy, (2) transition conditions out, (3) a timeout/fallback. No state should be reachable without an exit. |
| **Current state in AntTrails** | Movement system has `_ => (0, 0)` catch-all that freezes ants in Fighting, Following, Fleeing, and Carrying states. |
| **Complexity** | **Low-Medium** -- implement movement strategies for each state, add timeout fallbacks |
| **Dependencies** | None |

### TS-5: Colony Needs Drive Behavior (Not Just Random Walks)

| Aspect | Detail |
|--------|--------|
| **What** | When food is low, more ants forage. When enemies are near, more soldiers mobilize. When the queen has eggs, nurses attend them. The colony visibly responds to its situation. |
| **Why expected** | This is what separates a "simulation" from a "screensaver." Without it, colonies don't feel like organisms. |
| **How real ants do it** | Response threshold model (Bonabeau et al. 1996): each task has a colony-wide stimulus level (e.g., hunger stimulus rises as food drops). Each ant has individual thresholds per task. When stimulus exceeds threshold, ant switches to that task. Low-threshold ants are "specialists" who respond first; high-threshold ants are generalists who only respond in emergencies. |
| **How sims implement it** | Per-colony task stimulus values updated each tick based on colony state. Per-ant threshold values (can be fixed at spawn or drift with experience). Task selection: probability of engaging = stimulus^2 / (stimulus^2 + threshold^2). This is the standard sigmoidal response function from the literature. |
| **Current state in AntTrails** | Non-existent. Lifecycle system has fixed 80/20 worker/soldier ratio. No colony-needs feedback. |
| **Complexity** | **Medium** -- requires new stimulus tracking per colony, threshold values per ant, and integration into task selection |
| **Dependencies** | TS-3 (activity tuning), TS-4 (state machine) |

### TS-6: Basic Digging Intelligence

| Aspect | Detail |
|--------|--------|
| **What** | Workers dig purposefully -- expanding the nest downward, creating chambers, connecting tunnels -- not just randomly removing soil. |
| **Why expected** | Random digging produces ugly scattered holes. Purposeful digging produces recognizable nest structures that are satisfying to watch. |
| **How real ants do it** | Workers follow chemical gradients to dig sites. Nest architecture follows species-specific patterns but adapts to soil conditions. Chambers form at regular intervals. Digging direction influenced by gravity, moisture, and pheromones from other diggers. |
| **How sims implement it** | Dig-site pheromone type: queen/nest deposits "dig here" signal. Workers follow it to dig frontier. Depth-based chamber probability. Tunnel width constraints (1-2 tiles). Avoid digging into water. |
| **Current state in AntTrails** | dig_system exists, dig_movement prefers downward. But no purposeful nest architecture -- just downward bias. |
| **Complexity** | **Medium** -- add dig-site pheromone, chamber placement logic, tunnel width constraints |
| **Dependencies** | Pheromone system, terrain system (both exist) |

---

## Differentiators

Features that would make AntTrails special compared to other ant sims. Not expected, but create the "whoa, I didn't program that" moments the user wants.

### D-1: Response Threshold Task Allocation (Emergent Specialization)

| Aspect | Detail |
|--------|--------|
| **What** | Individual ants develop task preferences through experience. An ant that forages a lot becomes a better forager (lower threshold = responds to foraging stimulus sooner). Over time, the colony self-organizes into specialists without any top-down assignment. |
| **Value proposition** | This IS the emergent intelligence the user wants. Ants that "do things I didn't explicitly code." The colony develops its own division of labor based on its environment and history. Different colonies facing different challenges will develop different specialist distributions. |
| **How real ants do it** | The fixed response threshold model (Bonabeau 1996) describes it mathematically. Thresholds decrease with task performance (positive reinforcement) and increase with inactivity on a task (forgetting). Two mechanisms coexist: age polyethism (young ants tend brood, old ants forage -- following a natural progression) AND experience-based specialization (ants that succeed at a task lower their threshold for it). Research from Nature (2020) shows both caste polyethism and age-dependent switching operate simultaneously. |
| **How sims implement it** | Each ant has a threshold vector (one value per task type). Performing a task decreases that threshold by delta. Not performing it increases threshold by smaller delta. Task selection uses sigmoid: P(task) = S^n / (S^n + T^n) where S=stimulus, T=threshold, n=2. MIT Press (2022) found deterministic threshold models more robust than probabilistic ones in artificial ants. |
| **Complexity** | **Medium** -- add per-ant threshold vector, experience tracking, integrate into decision loop. Core formula is simple. |
| **Dependencies** | TS-5 (colony needs/stimulus), TS-4 (state machine) |

### D-2: Contextual Decision Scoring (Utility AI for Ants)

| Aspect | Detail |
|--------|--------|
| **What** | Instead of random probability rolls, each ant evaluates its situation and scores possible actions. "I smell food pheromone AND I'm hungry AND colony food is low = high foraging score." Multiple factors combine into a weighted decision. |
| **Value proposition** | Produces visibly intelligent behavior. Ants near food go forage. Ants near enemies flee (workers) or fight (soldiers). Ants near the queen tend brood. Behavior looks purposeful, not random. Combined with D-1, creates ants that both respond to context AND have individual personalities. |
| **How real ants do it** | Ants integrate multiple sensory inputs: pheromone concentrations (multiple types), visual cues, tactile information from nestmate encounters, internal state (hunger, age), and memory of recent experiences. Decision is not binary -- it's a weighted evaluation. |
| **How sims implement it** | Utility AI pattern from game development: define considerations (proximity to food, pheromone strength, colony food level, personal hunger, danger level, distance from home). Each consideration produces a 0.0-1.0 score. Multiply scores together (or weighted sum). Highest-scoring action wins, with noise for variety. This is the approach used in The Sims, Guild Wars 2, and other games with large agent counts. |
| **Complexity** | **Medium** -- define ~5-8 considerations, scoring curves, action evaluation loop. Replaces current probability rolls. |
| **Dependencies** | TS-5 (colony needs data), sensor data (pheromone grid exists, colony state exists) |

### D-3: Trophallaxis (Food Sharing Network)

| Aspect | Detail |
|--------|--------|
| **What** | Ants share food mouth-to-mouth when they encounter nestmates. Food distributes through the colony via social contacts, not just depot deposit/withdraw. Creates a visible food-sharing network. |
| **Value proposition** | Produces emergent nutrition distribution patterns. Ants near the food source are well-fed; ants deep in the nest get food through chains of sharing. Starvation starts at the periphery. Creates visible ant-to-ant interactions that look social. Also enables information transfer (ants that share food with a forager "learn" where food is). |
| **How real ants do it** | Foragers fill their crop (social stomach) and regurgitate to nestmates. The fluid contains not just food but hormones, pheromones, and RNA fragments. Food flows through the colony like a fluid network. Research (Nature, Scientific Reports) shows trophallactic networks have specific topology -- not random, but structured. |
| **How sims implement it** | When two same-colony ants are adjacent and one has food, probability of transfer. Food amount splits. Recipient satisfaction level affects transfer probability. Track a "fullness" value per ant (0.0-1.0). Hungry ants solicit from full ants. |
| **Complexity** | **Medium** -- add hunger/fullness component, adjacency check during movement, food transfer logic |
| **Dependencies** | Working foraging (TS-1), adjacency detection (movement system) |

### D-4: Colony-Level Adaptive Strategy

| Aspect | Detail |
|--------|--------|
| **What** | The colony as a whole shifts strategy based on circumstances. Low food -> more foragers, high casualties -> more soldiers, successful raiding -> expand territory, flooding -> relocate. These shifts emerge from individual threshold adjustments, not top-down commands. |
| **Value proposition** | The "macro" version of emergence. Individual ants adjusting thresholds produces visible colony personality. One colony becomes aggressive (lots of soldiers, territory expansion). Another becomes agrarian (lots of foragers, aphid farming focus). Each playthrough produces different colony "personalities." |
| **How real ants do it** | Colony size affects strategy: small colonies use solitary foraging, medium colonies use tandem running, large colonies use mass recruitment trails (PLOS ONE, 2010). Recruitment strategy shifts based on colony size. Worker/soldier ratios shift based on conflict frequency. Queens adjust egg-laying rate based on food availability and season. |
| **How sims implement it** | This emerges naturally from D-1 (threshold model) if stimulus levels are correctly tied to colony state. No explicit "strategy" code needed -- just correct stimulus signals. Food stimulus = max_food - current_food. Defense stimulus = recent_casualties * decay. Expansion stimulus = population - territory_capacity. The colony's emergent ratio of specialists IS its strategy. |
| **Complexity** | **Low** (if D-1 is implemented) -- mostly about defining the right stimulus functions. The emergence does the rest. |
| **Dependencies** | D-1 (threshold model), TS-5 (colony needs) |

### D-5: Recruitment Signals (Beyond Basic Pheromones)

| Aspect | Detail |
|--------|--------|
| **What** | Ants can actively recruit nestmates to tasks. A forager that finds a rich food source deposits extra-strong trail AND performs "excitement" behavior near the nest that triggers nearby idle ants to follow the trail. A soldier in combat deposits alarm pheromone that pulls nearby soldiers toward the fight. |
| **Value proposition** | Creates visible swarm responses. A food discovery triggers a rush of ants. A battle triggers soldier mobilization. These cascading responses look dramatic and intelligent. |
| **How real ants do it** | Multiple recruitment methods exist: (1) Tandem running -- scout leads one follower to food, proceeding slower than a single ant. (2) Group recruitment -- scout excites a group and leads them. (3) Mass recruitment -- strong pheromone trail triggers colony-wide response. Method scales with colony size. Alarm pheromone triggers immediate aggressive response in nearby ants. |
| **How sims implement it** | Recruitment pheromone type (already exists in component enum as `Recruit`). Deposit recruitment signal on finding food > threshold. Recruitment signal boosts activity of nearby idle ants (lowers their response threshold temporarily). For combat: alarm signal with fast diffusion but fast decay (urgent, short-lived). |
| **Complexity** | **Medium** -- new pheromone interaction logic, recruitment response in decision system |
| **Dependencies** | Pheromone system (exists), D-2 (contextual decisions) |

### D-6: Emergent Nest Architecture

| Aspect | Detail |
|--------|--------|
| **What** | Nests develop recognizable structure through emergent rules: entrance tunnels, branching corridors, specialized chambers (food storage, nursery, royal chamber, trash heap). Not pre-planned -- emerges from digging rules + pheromone signals. |
| **Value proposition** | Visually stunning emergent outcome. The nest "grows" organically and each colony's nest looks different based on terrain and history. Observers can see the nest develop structure over time. This is the spatial equivalent of emergent specialization. |
| **How real ants do it** | Nest architecture emerges from simple local rules: ants deposit soil pellets based on pheromone concentration, creating pillars that become walls. Chamber size regulated by ant body length. Brood sorting creates distinct zones (eggs near queen, older larvae further out) in annular rings. Different activities happen at different depths. |
| **How sims implement it** | Depth-based chamber probability. "Chamber seed" pheromone: when a digger reaches target depth, deposit signal that attracts more diggers laterally (widening = chamber). Connect chambers with tunnels (dig toward home pheromone gradient). Brood placement preferences by type create functional zones. Trash/midden placement at periphery or dedicated chamber. |
| **Complexity** | **High** -- requires multiple interacting rules for chamber formation, tunnel routing, zone designation. Hard to tune for aesthetic results. |
| **Dependencies** | TS-6 (digging intelligence), pheromone system, brood mechanics (lifecycle exists) |

### D-7: Age Polyethism (Temporal Division of Labor)

| Aspect | Detail |
|--------|--------|
| **What** | Young workers stay inside the nest tending brood. Middle-aged workers do nest maintenance and digging. Old workers forage outside (the most dangerous job). Ants naturally progress through roles as they age. |
| **Value proposition** | Creates visible life progression for individual ants. Combined with D-1, produces a rich labor system where both age AND experience matter. Also naturally protective: young (replaceable) ants do safe work, old ants take risks. Colony age demographics become strategically meaningful. |
| **How real ants do it** | Widely documented across species (Formicidae). Younger workers remain in the colony to care for eggs and larvae. Older workers forage outside. The "foraging for work" hypothesis suggests ants switch to tasks that need doing, and proximity to brood (young ants are near brood because they were recently brood) drives initial assignment. Age polyethism is more advantageous when mortality difference between tasks is large. |
| **How sims implement it** | Age influences initial response thresholds: young ants start with low brood-care threshold, high foraging threshold. Thresholds drift with age (foraging threshold naturally decreases). Combined with D-1's experience reinforcement, this creates a two-factor system. |
| **Complexity** | **Low** (if D-1 is implemented) -- just initialize thresholds based on age and add age-based drift |
| **Dependencies** | D-1 (threshold model), lifecycle system (exists with age tracking) |

### D-8: Brood Sorting and Spatial Organization

| Aspect | Detail |
|--------|--------|
| **What** | Nurse ants organize brood items by type, creating visible clusters: eggs together, small larvae together, large larvae together, pupae together. Items are picked up and moved based on local density of similar items. |
| **Value proposition** | Produces beautiful emergent spatial patterns inside the nest with zero top-down planning. An observer can watch disorder become order. Classic demonstration of self-organization. |
| **How real ants do it** | Ants move randomly through brood area. Pick up probability is inversely related to local density of same-type items (isolated items get picked up). Put down probability is proportional to local density of same-type items (items near similar items get placed). This simple rule produces sorted clusters and annular rings. |
| **How sims implement it** | For each nurse ant near brood: calculate local same-type density. Pick-up probability = (k1 / (k1 + same_density))^2. Put-down probability = (same_density / (k2 + same_density))^2. Two parameters (k1, k2) control sorting speed and cluster tightness. |
| **Complexity** | **Medium** -- requires brood as distinct entities (eggs/larvae already exist), nurse behavior, pick-up/put-down logic with density sensing |
| **Dependencies** | Lifecycle system (exists), carrying system (exists), spatial queries |

---

## Anti-Features

Things to deliberately NOT build. Common traps in ant simulation design.

### AF-1: Neural Networks / Machine Learning for Individual Ants

| Aspect | Detail |
|--------|--------|
| **Why avoid** | Massively overcomplicates the system for no emergent benefit. The entire point of ant colony emergence is that SIMPLE individual rules produce COMPLEX collective behavior. Adding NN to individuals makes debugging impossible, removes the elegant simplicity, and actually produces LESS interesting emergence (behavior becomes opaque rather than understandable). The original PLAN.md explicitly excludes this. |
| **What to do instead** | Utility AI scoring with response thresholds. Simple, tunable, debuggable, and produces better emergence because the rules are transparent and composable. |

### AF-2: Global Colony Planner / Commander AI

| Aspect | Detail |
|--------|--------|
| **Why avoid** | A top-down AI that decides "colony should expand east" or "produce more soldiers" defeats the purpose of emergence. If the colony's strategy is pre-programmed, there's nothing emergent about it. This is the approach Empires of the Undergrowth takes (player gives pheromone commands) -- fine for an RTS, wrong for an emergence simulator. |
| **What to do instead** | Colony-level behavior should ONLY emerge from individual ant decisions aggregated through response thresholds and pheromone feedback. Colony "strategy" is an observable emergent property, not a designed one. |

### AF-3: Pixel-Perfect Pathfinding (A*, Dijkstra)

| Aspect | Detail |
|--------|--------|
| **Why avoid** | Real ants don't have maps. Giving simulated ants perfect pathfinding removes the core mechanic that makes ant sims interesting: imperfect local information leading to collectively optimal solutions via pheromone reinforcement. A* to food source would eliminate trail formation entirely. Also expensive at 1000+ ants. |
| **What to do instead** | Pheromone gradient following with stochastic exploration. Ants sometimes wander "wrong" -- this is a feature, not a bug. Wrong turns occasionally find shortcuts, which then get reinforced. The imperfection IS the mechanism of emergence. |

### AF-4: Complex Genetics / Evolution System

| Aspect | Detail |
|--------|--------|
| **Why avoid** | Evolutionary timescales don't match simulation timescales. A single run shows one colony generation (maybe two if a new queen buds off). Building a genetics system for something that won't manifest observably within a typical session is wasted complexity. |
| **What to do instead** | Give each colony a fixed "personality" vector at spawn (aggression, foraging preference, expansion rate) that creates inter-colony variation without needing evolution. If desired later, colony budding could mix parent personality with mutation. |

### AF-5: Realistic Chemical Diffusion Physics

| Aspect | Detail |
|--------|--------|
| **Why avoid** | Full PDE-based pheromone diffusion is computationally expensive and visually indistinguishable from simple decay + neighbor averaging at the tile scale. AntTrails already has good enough pheromone mechanics. Over-engineering diffusion will eat performance budget needed for ant count. |
| **What to do instead** | Current decay model is fine. At most, add simple 4-neighbor diffusion (average with neighbors each tick at low rate). NetLogo's diffusion-rate parameter approach is sufficient and proven. |

### AF-6: Player Direct Control of Individual Ants

| Aspect | Detail |
|--------|--------|
| **Why avoid** | Explicitly out of scope per PROJECT.md. SimAnt did this (control one yellow ant). It's a game mechanic, not an emergence mechanic. Direct control undermines the observation experience that makes emergence interesting. |
| **What to do instead** | The player observes and maybe adjusts simulation parameters (speed, view). The ants run themselves. The fun is watching, not directing. |

### AF-7: Detailed Individual Ant Memory / Cognitive Maps

| Aspect | Detail |
|--------|--------|
| **Why avoid** | Giving each ant a memory of visited locations, a cognitive map, or learned routes adds O(n) memory per ant (multiplied by 1000+ ants) and doesn't produce better emergence than pheromone stigmergy. Real ants primarily use stigmergy precisely because it's external memory that scales. Internal memory is minimal. |
| **What to do instead** | Pheromones ARE the colony's memory. Individual ants need only: current state, role, thresholds, hunger level, age. Everything else is encoded in the environment via pheromones. This is the fundamental insight of stigmergy. |

---

## Feature Dependencies

```
TS-3 (Activity Tuning)          -- no dependencies, do first
TS-4 (State Machine Fix)        -- no dependencies, do first
    |
    v
TS-1 (Foraging Loop)            -- needs TS-3, TS-4
TS-2 (Pheromone Movement)       -- needs TS-4
    |
    v
TS-5 (Colony Needs)             -- needs TS-3, TS-4
TS-6 (Dig Intelligence)         -- needs TS-2
    |
    v
D-1 (Response Thresholds)       -- needs TS-5
D-2 (Utility AI Scoring)        -- needs TS-5
    |
    v
D-7 (Age Polyethism)            -- needs D-1
D-4 (Colony Strategy)           -- needs D-1
D-5 (Recruitment)               -- needs D-2
    |
    v
D-3 (Trophallaxis)              -- needs TS-1 (foraging working)
D-8 (Brood Sorting)             -- needs D-1, lifecycle
D-6 (Nest Architecture)         -- needs TS-6, D-1 (hardest, do last)
```

Dependency summary:
- **Phase 1 (Foundation Fix):** TS-3, TS-4, TS-1, TS-2 -- fix what's broken
- **Phase 2 (Decision Engine):** TS-5, D-2 -- build the contextual decision system
- **Phase 3 (Emergent Specialization):** D-1, D-7 -- response thresholds + age polyethism
- **Phase 4 (Colony Intelligence):** D-4, D-5 -- emergent strategy + recruitment
- **Phase 5 (Social Behaviors):** D-3, D-8 -- trophallaxis + brood sorting
- **Phase 6 (Spatial Emergence):** TS-6, D-6 -- digging intelligence + nest architecture

---

## MVP Recommendation

For the milestone "emergent AI behavior," prioritize:

1. **TS-3 + TS-4** (Fix activity + state machine) -- ants look alive immediately
2. **TS-1 + TS-2** (Wire up foraging + pheromone following) -- trails form, food flows
3. **TS-5 + D-2** (Colony needs + utility scoring) -- ants make contextual decisions
4. **D-1 + D-7** (Response thresholds + age polyethism) -- emergent specialization appears

These 8 features together produce the "ants do things I didn't explicitly code" outcome. A colony under food pressure will visibly shift to more foragers. An older colony will have experienced specialists. Two colonies facing each other will develop different strategies based on their unique histories.

Defer to post-MVP:
- **D-3** (Trophallaxis): Rich but not required for initial emergence. Can add social food sharing after core intelligence works.
- **D-6** (Nest Architecture): High complexity, high visual payoff, but the AI intelligence features should be solid first. Nest architecture is the "polish" emergence.
- **D-8** (Brood Sorting): Beautiful emergence but requires nurse behavior to be fully working. Natural second-pass feature.
- **D-5** (Recruitment): Amplifies existing behaviors. Add after base behaviors work well.

---

## Complexity Summary

| Feature | Complexity | New Code | Modifies Existing |
|---------|-----------|----------|-------------------|
| TS-1 Foraging Loop | Low | Minimal | movement.rs, food.rs |
| TS-2 Pheromone Movement | Low | Minimal | movement.rs |
| TS-3 Activity Tuning | Low | None | movement.rs constants |
| TS-4 State Machine Fix | Low-Med | Movement handlers per state | movement.rs |
| TS-5 Colony Needs | Medium | Stimulus tracking, threshold framework | colony.rs, new decision module |
| TS-6 Dig Intelligence | Medium | Dig-site pheromone, chamber logic | dig.rs, pheromone.rs |
| D-1 Response Thresholds | Medium | Per-ant threshold vector, experience tracking | components.rs, new decision module |
| D-2 Utility AI Scoring | Medium | Consideration system, scoring curves | New decision module |
| D-3 Trophallaxis | Medium | Hunger component, food transfer | components.rs, new system |
| D-4 Colony Strategy | Low* | Stimulus functions | colony.rs (*low if D-1 done) |
| D-5 Recruitment | Medium | Recruitment behavior, signal response | pheromone.rs, decision module |
| D-6 Nest Architecture | High | Chamber formation, tunnel routing, zone rules | dig.rs, pheromone.rs, terrain.rs |
| D-7 Age Polyethism | Low* | Age-based threshold init | components.rs (*low if D-1 done) |
| D-8 Brood Sorting | Medium | Density-based pick-up/put-down | New system, components.rs |

---

## Key Insight: The Response Threshold Model is the Linchpin

The single most impactful architectural decision for this milestone is implementing the response threshold model (D-1). Nearly every other differentiator either depends on it or is amplified by it:

- **D-4** (Colony Strategy) emerges FOR FREE from correct stimulus/threshold dynamics
- **D-7** (Age Polyethism) is just threshold initialization based on age
- **D-2** (Utility Scoring) provides the per-tick evaluation, thresholds provide the per-ant personality
- **D-5** (Recruitment) works by temporarily lowering thresholds of nearby ants

The formula is simple: `P(engage_task) = S^2 / (S^2 + T^2)`

Where S = colony stimulus for task, T = individual's threshold for task. This one equation, applied across all ants and all tasks, is what produces emergent division of labor, adaptive colony strategy, and individual specialization. It's the biological reality AND the proven simulation technique AND it's computationally cheap.

---

## Sources

### Academic / Research (HIGH confidence)
- [Bonabeau et al. 1996 - Fixed Response Thresholds](https://link.springer.com/article/10.1006/bulm.1998.0041) -- foundational threshold model
- [MIT Press 2022 - Deterministic vs Probabilistic Threshold Models](https://direct.mit.edu/artl/article/28/2/264/111794) -- deterministic models more robust
- [Nature 2020 - Task Allocation: Specialized Castes or Age-Dependent Switching](https://www.nature.com/articles/s41598-020-59920-5) -- both mechanisms coexist
- [Active Inferants Framework (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8264549/) -- active inference for ant colonies at 3 scales
- [PLOS ONE - Recruitment Strategies and Colony Size](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0011664) -- recruitment scales with colony size
- [PNAS 2022 - Collective Sensory Response Threshold](https://www.pnas.org/doi/10.1073/pnas.2123076119) -- colony-size-dependent thresholds
- [Pheromone Communication Concentration-Dependent Decisions](https://link.springer.com/article/10.1007/s00265-014-1770-3) -- psychophysical decision theory
- [Brood Sorting by Ants (Behavioral Ecology)](https://link.springer.com/article/10.1007/BF00173947) -- pick-up/put-down density rules

### Simulation References (MEDIUM confidence)
- [NetLogo Ants Model](https://ccl.northwestern.edu/netlogo/models/Ants) -- canonical minimal ant simulation
- [NetLogo Ants Simple](https://ccl.northwestern.edu/netlogo/models/AntsSimple) -- simplified version with clear rules
- [Practicing Ruby - Ant Colony Simulation](https://practicingruby.com/articles/ant-colony-simulation) -- implementation walkthrough
- [GitHub: jeffasante/ant-colony-rl](https://github.com/jeffasante/ant-colony-rl) -- Q-learning approach (interesting but not recommended for this project)
- [GitHub: bones-ai/rust-ants-colony-simulation](https://github.com/bones-ai/rust-ants-colony-simulation) -- Rust/Bevy reference implementation

### Game Design References (MEDIUM confidence)
- [SimAnt Wikipedia](https://en.wikipedia.org/wiki/SimAnt) -- classic ant game mechanics
- [Empires of the Undergrowth](https://store.steampowered.com/app/463530/Empires_of_the_Undergrowth/) -- modern ant colony RTS
- [Game AI Pro - Utility Theory](http://www.gameaipro.com/GameAIPro/GameAIPro_Chapter09_An_Introduction_to_Utility_Theory.pdf) -- utility AI system design
- [Game AI Pro 3 - Utility Considerations](http://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter13_Choosing_Effective_Utility-Based_Considerations.pdf) -- effective scoring
- [Spatial Hashing vs ECS](https://leetless.de/posts/spatial-hashing-vs-ecs/) -- performance comparison for Bevy/Rust

### Biology References (HIGH confidence for real ant behavior)
- [Stigmergy - Wikipedia](https://en.wikipedia.org/wiki/Stigmergy) -- indirect coordination mechanism
- [Trophallaxis - AntWiki](https://www.antwiki.org/wiki/Trophallaxis) -- food sharing behavior
- [Scientific American - Ants and the Art of War](https://www.scientificamerican.com/article/ants-and-the-art-of-war/) -- ant warfare strategies
- [Entomology Today - How Ants Do Battle](https://entomologytoday.org/2024/07/19/ant-battles-strategies-tactics-toxic-sprays-armor-overwhelming-numbers-more/) -- combat tactics in ants
