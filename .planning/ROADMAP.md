# Roadmap: AntTrails

## Overview

Transform AntTrails from a compiling-but-frozen ant colony simulator into a living ecosystem where hundreds of ants make contextual decisions, develop individual specializations, and produce colony-level intelligence that surprises its creator. The roadmap front-loads critical integration fixes (ants currently freeze in key states and sit idle 88-97% of ticks), then layers on pheromone communication, utility-based AI, emergent specialization, and colony-level adaptation -- each phase delivering observable behavioral improvement over the last.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Unfreeze & Activate** - Wire orphaned movement code, tune activity rates, add spatial hashing so ants visibly move and interact
- [ ] **Phase 2: Pheromone Communication** - Fix pheromone saturation/decay balance and wire gradient following so visible ant trails form
- [ ] **Phase 3: Config Centralization** - Gather 20+ scattered magic constants into a single tunable config before adding more systems
- [ ] **Phase 4: Utility AI Core** - Replace dice-roll state transitions with context-aware scoring so ants make situational decisions
- [ ] **Phase 5: Emergent Specialization** - Add memory, response thresholds, and age polyethism so individual ants develop unique behavioral profiles
- [ ] **Phase 6: Colony Intelligence** - Wire colony-level aggregation into individual scoring so colony strategy emerges without central control
- [ ] **Phase 7: Debug & Tuning Dashboard** - Add debug overlay with decision reasoning, pheromone heatmap, and aggregate stats for tuning emergent behavior

## Phase Details

### Phase 1: Unfreeze & Activate
**Goal**: Ants in all behavioral states move purposefully and the simulation feels alive -- no frozen ants, no idle wasteland
**Depends on**: Nothing (first phase)
**Requirements**: FIX-01, FIX-02, FIX-04
**Success Criteria** (what must be TRUE):
  1. Ants carrying food move back toward their colony nest instead of freezing at their pickup location
  2. Ants in Fighting and Fleeing states move toward enemies or away from danger respectively, not stuck at (0,0)
  3. At any given moment, the majority of worker ants are visibly doing something (moving, digging, foraging) rather than sitting idle
  4. The simulation runs at 30 FPS with 500+ ants on screen without frame drops from neighbor lookups
  5. No AntState variant produces a (0,0) freeze -- every state has explicit movement handling
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md -- Wire orphaned movement functions and eliminate wildcard state matches
- [x] 01-02-PLAN.md -- Tune activity probabilities and validate ant liveliness
- [x] 01-03-PLAN.md -- Implement spatial hash grid for O(1) neighbor lookups

### Phase 2: Pheromone Communication
**Goal**: Ants lay and follow pheromone trails that form visible paths between food sources and colonies -- the foundation of stigmergic intelligence
**Depends on**: Phase 1
**Requirements**: FIX-03
**Success Criteria** (what must be TRUE):
  1. Visible trails form between food sources and colony nests as ants forage, with trail intensity reflecting traffic volume
  2. Ants arriving at a pheromone gradient visibly turn toward the stronger signal rather than walking randomly
  3. Pheromone trails decay over time -- abandoned trails fade while active trails persist, creating meaningful gradients
  4. Different pheromone types (food, home, danger) decay at different rates appropriate to their purpose
**Plans**: TBD

Plans:
- [ ] 02-01: Implement adaptive pheromone deposit rates and tau_min/tau_max clamping
- [ ] 02-02: Wire pheromone gradient following into the movement system
- [ ] 02-03: Tune decay rates per pheromone type and validate trail formation

### Phase 3: Config Centralization
**Goal**: All behavioral constants live in one place so tuning ant behavior is a config edit, not a codebase scavenger hunt
**Depends on**: Phase 2
**Requirements**: POL-01
**Success Criteria** (what must be TRUE):
  1. A single SimConfig struct (or equivalent) contains all tunable behavior parameters from across the codebase
  2. Changing a pheromone decay rate, activity probability, or combat parameter requires editing exactly one location
  3. The config is organized by system (movement, pheromone, combat, lifecycle) so parameters are findable
**Plans**: TBD

Plans:
- [ ] 03-01: Audit all magic constants across source files and create centralized config
- [ ] 03-02: Replace scattered constants with config references and validate behavior unchanged

### Phase 4: Utility AI Core
**Goal**: Ants evaluate their situation and choose the best action based on context -- nearby food, danger, colony needs, pheromone signals -- instead of rolling dice
**Depends on**: Phase 3 (config provides tunable parameters), Phase 1 (spatial hash enables SenseData), Phase 2 (pheromone signals meaningful)
**Requirements**: AI-01
**Success Criteria** (what must be TRUE):
  1. An ant near a food source visibly prioritizes foraging over wandering -- proximity to food increases foraging likelihood
  2. An ant near an enemy visibly reacts (soldiers engage, workers flee) rather than ignoring the threat
  3. Worker ants in different situations make different choices -- an ant near food behaves differently from one near an unexplored tunnel
  4. Ant behavior varies slightly between individuals in identical situations (weighted randomization prevents robotic uniformity)
  5. The simulation maintains 30 FPS with utility evaluation active for 500+ ants (budget-aware evaluation)
**Plans**: TBD

Plans:
- [ ] 04-01: Build SenseData perception layer (per-ant world snapshot)
- [ ] 04-02: Implement response curve functions and action scoring framework
- [ ] 04-03: Build Worker UtilityAI (score: wander, forage, return, dig, flee, scout)
- [ ] 04-04: Build Soldier UtilityAI (score: patrol, attack, defend, flee)
- [ ] 04-05: Retire legacy AI systems and validate contextual behavior

### Phase 5: Emergent Specialization
**Goal**: Individual ants develop unique behavioral profiles through experience -- some become dedicated foragers, others lean toward digging or nursing -- without any explicit role assignment
**Depends on**: Phase 4 (Utility AI must exist for specialization to modify)
**Requirements**: AI-02
**Success Criteria** (what must be TRUE):
  1. After 500+ ticks, ants that have been foraging show measurably higher foraging preference than ants that have been digging
  2. Young ants stay closer to the nest (nursing bias) while older ants venture further out (foraging bias)
  3. No ant becomes permanently locked into one role -- specialization shifts when colony conditions change
  4. Watching two ants of similar age in the same area, they make noticeably different decisions based on their accumulated experience
**Plans**: TBD

Plans:
- [ ] 05-01: Implement AntMemory component and OutcomeObserver system
- [ ] 05-02: Build Specialization calculator and integrate as utility score multipliers
- [ ] 05-03: Implement response threshold framework with experience-based drift
- [ ] 05-04: Add age polyethism (young nurse, middle dig, old forage)

### Phase 6: Colony Intelligence
**Goal**: Colony strategy emerges from shared state -- when food runs low, more ants forage; when enemies press, soldiers mobilize -- without any central "colony brain" issuing orders
**Depends on**: Phase 5 (individual specialization provides the mechanism colony signals modulate)
**Requirements**: AI-03
**Success Criteria** (what must be TRUE):
  1. When a colony's food stores drop below a threshold, visibly more ants shift from idle/digging to foraging within 100 ticks
  2. When enemies appear near a colony entrance, soldier activity near that entrance increases within 50 ticks
  3. Queen egg-laying ratios shift based on colony state (more soldiers when threatened, more workers when food-rich)
  4. Colony strategy differences emerge between the three colonies based on their different circumstances (terrain, food proximity, neighbor pressure)
**Plans**: TBD

Plans:
- [ ] 06-01: Build ColonyStrategy resource and ColonyAggregator system
- [ ] 06-02: Integrate colony urgencies into individual utility scoring
- [ ] 06-03: Wire queen egg-laying ratios to colony state
- [ ] 06-04: Tune stimulus functions and validate emergent strategy shifts

### Phase 7: Debug & Tuning Dashboard
**Goal**: Developers can observe the invisible -- ant decision reasoning, pheromone concentrations, colony aggregate stats -- enabling informed tuning of emergent behavior
**Depends on**: Phase 6 (all behavior systems exist and need observation)
**Requirements**: POL-02
**Success Criteria** (what must be TRUE):
  1. Pressing a key toggles a pheromone heatmap overlay showing trail concentrations in the terminal
  2. Selecting an ant shows its current utility scores, top action candidates, and specialization values
  3. A colony stats panel shows aggregate data (ant count by state, food stores, urgency levels) updating in real time
  4. The debug overlay does not degrade performance below 30 FPS when active
**Plans**: TBD

Plans:
- [ ] 07-01: Implement pheromone heatmap overlay rendering
- [ ] 07-02: Build per-ant decision inspector (utility scores, specialization, memory)
- [ ] 07-03: Add colony aggregate stats panel and real-time urgency display

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Unfreeze & Activate | 3/3 | Complete | 2026-02-07 |
| 2. Pheromone Communication | 0/3 | Not started | - |
| 3. Config Centralization | 0/2 | Not started | - |
| 4. Utility AI Core | 0/5 | Not started | - |
| 5. Emergent Specialization | 0/4 | Not started | - |
| 6. Colony Intelligence | 0/4 | Not started | - |
| 7. Debug & Tuning Dashboard | 0/3 | Not started | - |
