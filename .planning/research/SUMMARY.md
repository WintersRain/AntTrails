# Project Research Summary

**Project:** AntTrails - Emergent AI Behavior for Ant Colony Simulation
**Domain:** Emergent agent-based simulation (terminal-based, ECS architecture)
**Researched:** 2026-02-06
**Confidence:** HIGH

## Executive Summary

AntTrails is a terminal-based ant colony simulator built on hecs ECS, ratatui, and existing pheromone/terrain systems. The current implementation has all the infrastructure for emergent behavior but suffers from broken integration: the movement system freezes ants in critical states (Carrying, Fighting, Fleeing) and orphaned AI functions never execute. Research shows that transforming this into genuinely emergent colony intelligence requires four interlocking layers: (1) hand-rolled Utility AI decision scoring that replaces dice-roll state transitions, (2) stigmergy-based pheromone communication via an expanded grid system, (3) response threshold specialization that produces emergent division of labor, and (4) colony-level aggregation that influences individual decisions without central control.

The recommended approach is NOT to add heavy external AI libraries (no big-brain, no neural nets, no GOAP) but instead to implement lightweight scoring algorithms directly in the codebase. The biological reality is that ant colony intelligence emerges from simple individual rules combined with environmental communication (pheromones), not from complex individual cognition. The core mechanism is the response threshold model: each ant has individual thresholds per task type, colony needs produce stimulus levels, and task engagement probability follows `P = S^2 / (S^2 + T^2)` where S=stimulus and T=threshold. This one formula, applied across 500+ ants independently evaluating their situations, produces emergent specialization, adaptive colony strategy, and visibly intelligent behavior from context-aware decisions.

The critical risk is mistaking randomness for emergence. The current system already shows this pattern: extensive random probability checks produce activity without causality. The fix requires wiring existing purposeful movement code before adding utility scoring, implementing pheromone gradient following that ants actually use, and ensuring every ant state has valid transitions and movement strategies. Performance is a secondary risk: 1000 ants evaluating 6-10 actions with 5+ considerations can exceed frame budgets, requiring staggered evaluation, spatial hashing for neighbor queries, and budget-aware AI systems from the start.

## Key Findings

### Recommended Stack

AntTrails already has a solid foundation (hecs 0.10.x, ratatui 0.29, crossterm, Perlin noise, fastrand). Stay on hecs 0.10.x for this milestone as 0.11.0 introduced breaking query iterator changes orthogonal to AI work. The research strongly recommends hand-rolling the AI systems rather than adopting external libraries for three reasons: (1) ant colony AI is domain-specific and generic libraries add abstraction without value, (2) the scoring logic is simple enough that library overhead is waste, and (3) most Rust AI crates are Bevy-coupled and incompatible with hecs.

**Core technologies to hand-roll:**
- **Utility AI scoring system**: Score-based decisions where each ant evaluates possible actions using response curves (linear, quadratic, sigmoid, inverse) and selects highest-scoring action with weighted randomization for variety. Produces context-sensitive behavior that naturally varies based on ant perception without explicit if/else trees.
- **Stigmergy/pheromone grid system**: 2D grid overlay storing multiple pheromone channels (Food, Home, Danger, Recruit, Scout, Territory) that diffuse and decay. This IS the mechanism producing emergent colony intelligence. Individual decisions aggregate into colony-level strategy through shared chemical signals.
- **Finite State Machine via ECS components**: Ant behavioral state (Idle, Foraging, Returning, Nursing, Guarding) as enum component determines which utility scorers are active. States are coarse-grained mode; utility AI is fine-grained decision within each state.

**Only one new dependency recommended:**
- **ordered-float 5.1.0**: Total ordering for f32 utility scores. Utility scores need Ord for max selection. Tiny dependency, widely used.

### Expected Features

Research identified a clear hierarchy from biological reality and existing simulation patterns (NetLogo Ants, SimAnt, krABMaga literature).

**Must have (table stakes):**
- **TS-1: Coherent foraging loop** - Workers find food, carry it home, deposit it, return. Visible trail formation. This is THE canonical ant behavior. Code exists (`foraging_movement()`) but is orphaned, never called.
- **TS-2: Pheromone-guided movement** - Ants probabilistically follow gradients ("sniff ahead in 3 directions, turn toward strongest"). Without this, ants are random walkers. Code exists (`follow_pheromone()`) but movement system calls `random_movement()` instead.
- **TS-3: Activity level tuning** - Ants visibly active 60-80% of time, not current 3-12%. An ant sim where ants mostly sit idle is broken.
- **TS-4: State machine coherence** - Every state has valid transitions and movement behavior. No stuck ants. Current `_ => (0,0)` catch-all freezes Carrying/Fighting/Following/Fleeing states.
- **TS-5: Colony needs drive behavior** - When food is low, more ants forage. When enemies near, soldiers mobilize. Response threshold model: colony-wide stimulus values influence individual ant task selection.
- **TS-6: Basic digging intelligence** - Purposeful nest expansion with dig-site pheromones, depth-based chamber probability, tunnel constraints. Not random soil removal.

**Should have (competitive differentiators):**
- **D-1: Response threshold task allocation** - Individual ants develop task preferences through experience. Thresholds decrease with task performance, increase with inactivity. THE core mechanism for emergent specialization without top-down role assignment.
- **D-2: Contextual decision scoring (Utility AI)** - Ants evaluate situation and score actions: "I smell food AND I'm hungry AND colony food is low = high foraging score." Multiple factors combine. Replaces random probability rolls.
- **D-7: Age polyethism** - Young workers stay inside (nursing), middle-aged do digging, old workers forage (most dangerous). Natural role progression as ants age. Combined with D-1 creates rich labor system.
- **D-4: Colony-level adaptive strategy** - Colony shifts strategy based on circumstances. Emerges from D-1 threshold model if stimulus levels correctly tied to colony state. No explicit "strategy" code needed.

**Defer (v2+):**
- **D-3: Trophallaxis** - Food sharing mouth-to-mouth. Rich social interaction but not required for initial emergence.
- **D-6: Emergent nest architecture** - Recognizable chamber/corridor structure from digging rules. High complexity, high visual payoff, but polish emergence after core AI works.
- **D-8: Brood sorting** - Spatial organization by item type through density-based pick-up/put-down. Beautiful self-organization but requires nurse behavior fully working.
- **D-5: Recruitment signals** - Active recruitment beyond pheromones (excitement behavior, alarm response). Amplifies existing behaviors, add after base behaviors work.

### Architecture Approach

The emergent AI slots into existing ECS as four new layers built on top of current systems: (1) Stimulus Layer - per-ant SenseData component rebuilt each tick from terrain/pheromone/neighbor scans, (2) Decision Layer - UtilityAI system scores candidate actions and writes ActionIntent component, (3) Action Layer - existing systems (movement, dig, food, combat) execute based on AntState set by decision, (4) Feedback Layer - OutcomeObserver updates AntMemory, ColonyAggregator computes colony-level urgencies, pheromone deposit happens.

**Major components:**
1. **SenseData (NEW)** - Per-ant snapshot of nearby world state (food proximity, enemy count, pheromone concentrations, colony urgencies). Pre-computed to avoid borrow-checker issues and repeated queries. Makes scoring functions pure.
2. **AntMemory (NEW)** - Individual experience counters (food collected, combats won, distance explored, ticks digging/nursing). Cheap counters that accumulate over time, no complex structures. Produces divergent utility scores through Specialization.
3. **Specialization (NEW)** - Derived aptitudes calculated periodically from AntMemory (forager_affinity, soldier_affinity, scout_affinity, digger_affinity, nurse_affinity). Acts as multiplier on utility scores (0.8x to 1.2x), soft bias not hard gate.
4. **UtilityAI system (NEW)** - Core decision engine. Scores all available actions using consideration functions (response curves), selects highest with weighted random from top bucket. Maps selected action to AntState for backward compatibility with existing systems.
5. **ColonyStrategy resource (NEW)** - Colony-level aggregated needs (food_urgency, defense_urgency, expansion_urgency, nurse_urgency) computed from colony state. NOT a "colony brain" issuing orders but aggregation individual ants read as one input to utility scoring. Colony intelligence emerges from shared chemical-like signals biasing individual behavior.
6. **Expanded PheromoneGrid (MODIFY)** - From 3 types (Food, Home, Danger) to 6 types (add Recruit, Scout, Territory). Grid size changes from `width * height * colonies * 3` to `* 6`. Concentration-based signaling where thresholds trigger specific responses.

### Critical Pitfalls

1. **"Emergent" behavior that is dressed-up randomness** - Fancier dice rolls that still look aimless because nothing causal. Current code shows this: `foraging_movement()` exists but never called, movement system sends many states to `_ => (0,0)` freeze. Prevention: Wire up existing purposeful movement code FIRST before adding utility scoring. Define behavioral tests ("ant 3 tiles from food should move toward it >70% of time"). Add randomness on top of purposeful behavior, not as base layer.

2. **Performance death spiral from per-agent utility evaluation** - 1000 ants * 8 considerations * 8 tile checks = 64,000 operations per tick, 30x/sec = 2.5M operations/sec. Frame time blows budget. Prevention: Budget-aware AI (evaluate max 100 ants per frame, round-robin), hierarchical evaluation (fast rejection first), spatial hashing for neighbor lookups (not O(N^2) entity iteration), cache pheromone gradients for active regions.

3. **Pheromone system floods or evaporates into uselessness** - Current deposit 0.05/tick with decay 0.001/tick means trails saturate to 1.0 and stay bright forever (no gradient), OR increased decay makes trails disappear before ants can follow. Prevention: Bounded pheromone with adaptive deposit (decreases as tile level rises), separate decay rates per type (danger fast, food medium, home slow), visualization/debug overlay, tau_min/tau_max clamping from ACO literature.

4. **Specialization locks ants into permanent roles** - Experience only increases, ants lock into first activity within 200 ticks. Current `decide_worker_state()` has 70% dig probability so most ants become diggers. Colony cannot adapt. Prevention: Decay specialization when not practicing role, colony need signals override individual preference, use probability weighting not thresholds, cap max specialization difference (no more than 3x preference).

5. **The Carrying state freeze pattern (incomplete state machine coverage)** - New states added but downstream systems don't handle them. Ant enters state and freezes. Already happening: `movement_system()` line 41 has `_ => (0,0)` silently freezing Carrying/Fighting/Following/Fleeing. `foraging_movement()`, `fighting_movement()`, `fleeing_movement()` exist but orphaned. Prevention: Eliminate all wildcard matches on AntState, add exhaustive match enforcement, state duration watchdog (warn if same state >200 ticks), wire orphaned functions before writing new ones.

## Implications for Roadmap

Based on research, suggested phase structure organized around dependency chains and risk mitigation:

### Phase 1: Foundation Fix & Infrastructure
**Rationale:** Cannot build utility AI on top of broken wiring. The codebase has purposeful movement code that never executes and states that freeze ants. Fix integration and add spatial infrastructure before adding complexity. This phase delivers immediate visible improvement (ants become un-stuck) and validates patterns that all future phases will extend.

**Delivers:**
- Ants in Carrying state actually move home with food (wire `foraging_movement()`)
- Ants in Fighting/Fleeing states move purposefully (wire `fighting_movement()/fleeing_movement()`)
- Activity probability tuned from 3-12% to 60-80% (ants look alive)
- Spatial hash grid for O(1) neighbor queries (prerequisite for SenseData)
- All AntState wildcard matches removed (exhaustive handling)

**Addresses:** TS-1 (foraging loop broken), TS-3 (activity tuning), TS-4 (state machine coherence)

**Avoids:** Pitfall 1 (randomness masquerading as emergence), Pitfall 5 (Carrying freeze pattern), Pitfall 9 (N-squared entity interactions)

**Research flags:** Standard bug fixes and well-documented spatial hashing patterns. Skip research-phase.

---

### Phase 2: Pheromone Communication Overhaul
**Rationale:** Pheromone gradients are THE mechanism for emergent colony intelligence. Current system has saturation/evaporation balance wrong. Must be working before any AI that depends on pheromone following. This phase makes existing infrastructure functional through tuning, not wholesale replacement.

**Delivers:**
- Adaptive pheromone deposit (decreases as tile level rises, prevents saturation)
- Separate decay rates per pheromone type (danger fast, food medium, home slow)
- Pheromone debug visualization (background colors or unicode density in terminal)
- Tau_min/tau_max clamping (floor prevents complete disappearance, ceiling prevents saturation)
- Ants actually follow pheromone gradients visibly (wire `follow_pheromone()` into movement system)

**Addresses:** TS-2 (pheromone-guided movement), foundation for TS-5 (colony needs)

**Avoids:** Pitfall 3 (pheromone floods or evaporates), establishes correct signal quality before decision systems read them

**Research flags:** ACO (ant colony optimization) literature has well-documented evaporation/deposit balancing formulas. Possible quick research on tau_min/tau_max parameter ranges, but likely skip research-phase as patterns are standard.

---

### Phase 3: Decision Layer - Utility AI Core
**Rationale:** With movement wiring fixed (Phase 1) and pheromone signals working (Phase 2), now build the decision engine that chooses what to do. This is the biggest architectural addition but dependencies are satisfied. Focus on Workers first (more actions, more interesting), Soldiers second (fewer actions, simpler scoring).

**Delivers:**
- SenseData component + SenseDataBuilder system (perception layer)
- AntMemory component (zeroed at spawn, updated by feedback)
- Specialization component (zeroed at spawn, calculated periodically)
- UtilityAI system for Workers (score: wander, forage, return food, dig, flee, scout)
- UtilityAI system for Soldiers (score: patrol, attack, defend, flee)
- Response curves (linear, quadratic, sigmoid, inverse) as f32 -> f32 functions
- ActionIntent component maps to AntState for backward compatibility
- RETIRE dig_ai_system, soldier_ai_system, flee_system (subsumed by UtilityAI)

**Addresses:** D-2 (contextual decision scoring), foundation for D-1 (response thresholds)

**Avoids:** Pitfall 2 (performance death spiral via budget-aware evaluation from start), Pitfall 6 (robotic determinism via weighted random selection)

**Research flags:** Utility AI scoring is well-documented in Game AI Pro references. Implementation patterns are clear. Skip research-phase, but flag for potential tuning research if initial scoring curves produce unexpected behavior.

---

### Phase 4: Emergent Specialization & Memory
**Rationale:** Utility AI (Phase 3) works but every ant behaves identically in identical situations. Add memory and specialization to produce individual variation and emergent role differentiation. Response threshold model is the linchpin for all colony-level intelligence.

**Delivers:**
- OutcomeObserver system (updates AntMemory counters based on action outcomes)
- SpecializationCalculator system (periodic, derives affinities from memory)
- Specialization integrated into UtilityAI scoring as multipliers (0.8x-1.2x)
- Response threshold framework (per-ant thresholds initialized from age, drift with experience)
- Age polyethism (young -> nursing, middle -> digging, old -> foraging)

**Addresses:** D-1 (response threshold task allocation), D-7 (age polyethism), foundation for D-4 (colony strategy)

**Avoids:** Pitfall 4 (specialization locks via decaying thresholds and colony override), Pitfall 6 (robotic predictability via per-ant personality offsets)

**Research flags:** Response threshold model formula `P = S^2 / (S^2 + T^2)` is from academic literature (Bonabeau 1996). Parameters well-documented. Skip research-phase, but monitor for tuning needs during implementation.

---

### Phase 5: Colony-Level Intelligence
**Rationale:** With individual decision-making and specialization working (Phases 3-4), now add the colony-level aggregation that influences individual behavior without central control. This makes colony strategy emergent from shared state rather than scripted.

**Delivers:**
- ColonyStrategy resource (food_urgency, defense_urgency, expansion_urgency, nurse_urgency)
- ColonyAggregator system (periodic, computes urgencies from colony state and ant memories)
- Colony urgencies integrated into UtilityAI scoring as multipliers
- Queen egg-laying ratio responsive to colony state (not fixed 80/20 worker/soldier)
- Stimulus functions tied to observable colony conditions (food stores, casualties, population density)

**Addresses:** TS-5 (colony needs drive behavior), D-4 (colony-level adaptive strategy)

**Avoids:** Pitfall 5 (scripted or chaotic colony strategy via emergent inputs, structured weight outputs)

**Research flags:** Colony aggregation patterns are straightforward applications of response threshold model. Skip research-phase.

---

### Phase 6: Rich Communication & Refinement
**Rationale:** Core emergent behavior is working (Phases 3-5). Expand pheromone vocabulary and add refinement actions (nurse, guard, recruit) that leverage existing decision infrastructure.

**Delivers:**
- Expand PheromoneGrid from 3 to 6 types (add Scout, Territory, make Recruit functional)
- Concentration-based threshold behaviors (recruit > 0.5 pulls soldiers, danger > 0.7 forces flee)
- New WorkerActions: Nurse, Guard, Recruit (with scoring functions)
- New SoldierActions: Escort, RespondRecruit (with scoring functions)
- Parameter centralization (move scattered constants to config.rs or SimConfig struct)
- Debug dashboard (real-time values: pheromone levels, ant state distribution, utility scores)

**Addresses:** TS-6 (digging intelligence via dig-site pheromone), D-5 (recruitment signals)

**Avoids:** Pitfall 7 (tuning hell via centralized constants and debug dashboard), Pitfall 12 (pheromone types without behaviors via simultaneous deposit/read implementation)

**Research flags:** Standard refinement work. Skip research-phase.

---

### Phase Ordering Rationale

- **Phase 1 before 2-6:** Cannot build on broken wiring. Integration fixes are prerequisite for everything.
- **Phase 2 before 3:** Utility AI reads pheromone signals; signals must be meaningful before decision system uses them.
- **Phase 3 before 4-5:** Decision engine must exist before adding memory/specialization/colony aggregation as modifiers.
- **Phases 4 and 5 are semi-independent:** Memory (Phase 4) and colony strategy (Phase 5) both modify utility scores but can be interleaved or parallelized if needed.
- **Phase 6 is polish:** Expands existing systems rather than adding new foundational layers. Safest to defer if timeline pressure.

**Dependency chain:** Phase 1 (spatial hash grid) blocks Phase 3 (SenseData needs spatial lookups) blocks Phase 4 (UtilityAI must exist before specialization modifies it) blocks Phase 5 (ColonyStrategy influences utility scoring).

**Risk frontloading:** Phases 1-2 address the critical pitfalls (broken integration, pheromone balance) before adding complexity. Performance infrastructure (spatial hashing, budget-aware eval) built into Phases 1 and 3 from the start, not bolted on later.

**Grouping logic:** Each phase delivers observable improvement (Phase 1: ants un-freeze, Phase 2: trails form and are followed, Phase 3: context-aware decisions, Phase 4: individual personalities emerge, Phase 5: colony responds to threats/food, Phase 6: richer behaviors).

### Research Flags

**Phases with standard patterns (skip research-phase):**
- **Phase 1:** Bug fixes and spatial hashing are well-documented.
- **Phase 3:** Utility AI patterns extensively documented in Game AI Pro. Implementation is straightforward application.
- **Phase 4:** Response threshold model has clear formula from academic sources.
- **Phase 5:** Straightforward extension of Phase 4 patterns.
- **Phase 6:** Refinement work building on established systems.

**Phases potentially needing targeted research:**
- **Phase 2:** If initial pheromone tuning fails, may need quick ACO literature review for tau_min/tau_max parameter ranges. Most likely can skip research-phase and tune empirically with debug visualization.
- **Phase 3:** If initial utility scoring curves produce unexpected behavior (ants too passive, too aggressive, oscillating), may need brief research on consideration curve shapes for specific scenarios. Not likely given extensive Game AI Pro documentation.

**Overall:** This is a well-researched domain with clear patterns. No phases require deep research during planning. All research-phase skips justified. Empirical tuning with debug tools will handle parameter adjustments.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | hecs/ratatui foundation verified, external AI libraries correctly rejected, hand-rolled approach matches domain requirements. ordered-float is only new dependency and is widely used. |
| Features | MEDIUM-HIGH | Table stakes grounded in biological reality and existing sims (NetLogo, SimAnt), response threshold model academically validated. Feature dependencies traced through codebase analysis. Differentiation level (D-1 through D-8) may need runtime validation to confirm emergent quality. |
| Architecture | HIGH | Integration patterns verified against codebase (19 source files analyzed), ECS component boundaries follow hecs idioms, Sense-Decide-Act-Feedback loop is established game AI pattern, backward compatibility strategy (ActionIntent -> AntState) preserves existing systems. |
| Pitfalls | HIGH | All critical pitfalls verified through codebase inspection (orphaned functions, wildcard matches, current constants producing saturation). Performance concerns grounded in O(N^2) patterns visible in combat.rs and foraging.rs. Prevention strategies match standard game AI solutions. |

**Overall confidence:** HIGH

Research is grounded in three sources: (1) direct codebase analysis of all 19 source files showing current broken patterns, (2) academic literature on ant colony behavior and response threshold models (Bonabeau 1996, Nature 2020, MIT Press 2022), and (3) game AI implementation patterns (Game AI Pro, utility AI in The Sims/Zoo Tycoon). The convergence of biological reality, simulation best practices, and existing code analysis produces high confidence.

### Gaps to Address

**Parameter tuning ranges:** Research identifies the mechanisms (adaptive deposit, decay rates, response curves) but specific numeric values will require empirical tuning. The debug dashboard (Phase 6) is essential infrastructure for this. Recommendation: Start with NetLogo Ants reference values (diffusion-rate, evaporation-rate parameters) as baseline, tune from there.

**Emergent behavior validation:** While the mechanisms (utility AI, response thresholds, stigmergy) are proven to produce emergence, the specific quality of emergence in AntTrails cannot be validated until runtime. The "ants do things I didn't explicitly code" success criterion is subjective. Recommendation: Define concrete behavioral tests ("colony under food pressure shifts >30% of ants to foraging within 100 ticks") to make emergence measurable.

**Performance scaling:** Research identifies spatial hashing and budget-aware evaluation as solutions but actual frame time at 1000 ants depends on terminal rendering overhead (ratatui) in addition to AI cost. Recommendation: Add frame time instrumentation in Phase 1, benchmark after Phase 3 when UtilityAI is active, adjust budget parameters before Phase 4.

**Pheromone visualization technique:** Research recommends debug visualization but specific terminal rendering approach (background colors vs. unicode density characters vs. separate overlay mode) depends on ratatui capabilities and terminal color support. Recommendation: Prototype in Phase 2, simplest approach is background color intensity mapping pheromone 0.0-1.0 to grayscale.

## Sources

### Primary (HIGH confidence)
- **Codebase analysis**: Direct inspection of all 19 source files in `E:/VS Code Projects/AntTrails/src/` - current broken patterns (orphaned functions, wildcard matches, constant values)
- **hecs crate**: https://lib.rs/crates/hecs v0.10.5 and CHANGELOG for v0.11.0 breaking changes
- **ordered-float**: https://lib.rs/crates/ordered-float v5.1.0 (Sep 2025)
- **Game AI Pro Chapter 9**: http://www.gameaipro.com/GameAIPro/GameAIPro_Chapter09_An_Introduction_to_Utility_Theory.pdf - Utility AI foundational reference
- **Game AI Pro 3 Chapter 13**: http://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter13_Choosing_Effective_Utility-Based_Considerations.pdf - Scoring design patterns
- **Bonabeau et al. 1996**: https://link.springer.com/article/10.1006/bulm.1998.0041 - Fixed response threshold model
- **MIT Press 2022**: https://direct.mit.edu/artl/article/28/2/264/111794 - Deterministic threshold models more robust
- **Nature 2020**: https://www.nature.com/articles/s41598-020-59920-5 - Age polyethism and caste specialization coexist

### Secondary (MEDIUM confidence)
- **Utility AI Architecture**: https://shaggydev.com/2023/04/19/utility-ai/ - Practical implementation guide, scoring patterns, response curves
- **NetLogo Ants Model**: https://ccl.northwestern.edu/netlogo/models/Ants - Canonical minimal ant simulation with parameters
- **Practicing Ruby - Ant Colony Simulation**: https://practicingruby.com/articles/ant-colony-simulation - FSM + pheromone architecture walkthrough
- **Active Inferants Framework**: https://www.frontiersin.org/journals/behavioral-neuroscience/articles/10.3389/fnbeh.2021.647732/full - Stigmergy at 3 scales
- **ECS and AI Integration**: https://pixelmatic.github.io/articles/2020/05/13/ecs-and-ai.html - ECS-specific integration patterns
- **PLOS ONE - Recruitment Strategies**: https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0011664 - Recruitment scales with colony size
- **Spatial Hashing vs ECS**: https://leetless.de/posts/spatial-hashing-vs-ecs/ - Performance comparison for Rust/Bevy (patterns apply to hecs)
- **Ant Colony Optimization - Wikipedia**: https://en.wikipedia.org/wiki/Ant_colony_optimization_algorithms - Tau_min/tau_max clamping, stagnation prevention
- **Stigmergy - Wikipedia**: https://en.wikipedia.org/wiki/Stigmergy - Indirect coordination mechanism definition

### Tertiary (LOW confidence - background/context)
- **krABMaga**: https://krabmaga.github.io/ - ABM framework patterns (not used but informative for agent architecture)
- **Are We Game Yet AI**: https://arewegameyet.rs/ecosystem/ai/ - Rust AI ecosystem overview
- **Collective Stigmergic Optimization**: https://medium.com/@jsmith0475/collective-stigmergic-optimization-leveraging-ant-colony-emergent-properties-for-multi-agent-ai-55fa5e80456a - Design principles (needs validation)
- **SimAnt Wikipedia**: https://en.wikipedia.org/wiki/SimAnt - Classic ant game mechanics (historical reference)
- **Empires of the Undergrowth**: https://store.steampowered.com/app/463530/Empires_of_the_Undergrowth/ - Modern ant RTS (design comparison)

---
*Research completed: 2026-02-06*
*Ready for roadmap: yes*
