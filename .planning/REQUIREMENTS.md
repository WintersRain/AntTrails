# Requirements

## v1: Make It Work, Make It Watchable

### Foundation

- [ ] **FIX-01**: Wire orphaned movement functions (foraging_movement, fighting_movement, fleeing_movement) so ants in Carrying, Fighting, Fleeing states actually move instead of freezing at (0,0)
- [ ] **FIX-02**: Tune activity probabilities from 3-12% to 60-80% per tick so ants feel alive and responsive, not frozen
- [ ] **FIX-03**: Fix pheromone saturation — implement adaptive deposit rates and proper decay so gradient-following produces visible ant trails instead of flooding to max
- [ ] **FIX-04**: Add spatial hash grid for O(1) neighbor lookups, replacing O(N^2) pair checking in combat and enabling 1000+ ants at 30 FPS

### Intelligence

- [ ] **AI-01**: Replace dice-roll state transitions with Utility AI scoring — ants evaluate context (nearby food, danger, colony needs, pheromone signals) and pick the highest-scoring action
- [ ] **AI-02**: Implement response threshold model (P = S^2 / (S^2 + T^2)) — individual ants develop different thresholds for different tasks, producing emergent specialization without explicit role assignment
- [ ] **AI-03**: Add colony-level adaptation — colony tracks aggregate state (food stores, population, threats) and individual ants factor colony needs into their utility scores, producing emergent strategy shifts

### Polish

- [ ] **POL-01**: Centralize all magic constants (20+ scattered across 6 files) into a single tunable config, enabling rapid iteration on behavior parameters
- [ ] **POL-02**: Add debug overlay showing ant decision reasoning, pheromone heatmap, and colony aggregate stats — required for tuning emergent behavior

## v2 (After v1 Ships)

- Expanded pheromone types (Scout, Alarm, Territory, Recruitment)
- Recruitment behavior (tandem running)
- Trophallaxis (ant-to-ant food sharing)
- Brood sorting (emergent nest organization)
- Age polyethism (young nurse, old forage)
- Performance profiling with criterion benchmarks
- Pheromone visualization in terminal (color-coded heatmaps)

## Out of Scope

- Save/load — deferred from original plan
- GUI/graphical rendering — terminal sim
- Player direct control of ants — emergence is the point
- Networking/multiplayer — single-player
- Neural networks / ML — overkill, hand-rolled Utility AI is sufficient
- A* pathfinding — stigmergy replaces explicit pathfinding
- Per-ant cognitive maps — pheromones ARE the colony's memory

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FIX-01 | Phase 1: Unfreeze & Activate | Complete |
| FIX-02 | Phase 1: Unfreeze & Activate | Complete |
| FIX-03 | Phase 2: Pheromone Communication | Pending |
| FIX-04 | Phase 1: Unfreeze & Activate | Complete |
| AI-01 | Phase 4: Utility AI Core | Pending |
| AI-02 | Phase 5: Emergent Specialization | Pending |
| AI-03 | Phase 6: Colony Intelligence | Pending |
| POL-01 | Phase 3: Config Centralization | Pending |
| POL-02 | Phase 7: Debug & Tuning Dashboard | Pending |
