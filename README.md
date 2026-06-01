# lau-affordance

> **Environment-as-teacher** — affordances that guide agent behavior through *walls*, not instructions.

Part of the [PLATO/LAU](https://github.com/SuperInstance) ecosystem: a mathematically rigorous framework for building agents that learn, teach, and evolve under strict conservation laws.

---

## What This Does

`lau-affordance` is the *behavioral shaping layer* of the PLATO architecture. Instead of telling an agent what to do (instructions), you build an environment that *naturally guides* correct behavior through **affordance walls** — constraints the agent bumps into during execution.

Every action an agent attempts is evaluated against a battery of walls:

| Wall | Effect |
|---|---|
| **ConservationBudget** | Blocks if the action would exceed the energy budget |
| **ApprovalNeeded** | Blocks if the action type requires explicit approval |
| **CrewRequired** | Blocks if the action needs a crew assignment |
| **OverrideActive** | Non-blocking warning that captain override is in effect |
| **DecompositionSuggested** | Non-blocking hint to decompose into sub-intentions |
| **FieldReading** | Non-blocking warning about stale/incomplete field data |
| **GrowthOpportunity** | Non-blocking signal that this action earns growth XP |

The agent *learns* from every wall it hits — not because you told it a rule, but because the environment enforced it. Over time, a **GrowthTracker** records lessons and a **SelfAssemblingDNA** tree evolves the agent's preferred behavioral pathways.

---

## The Key Idea

> **Teach through constraints, not corrections.**

Traditional agent systems correct wrong behavior after the fact. `lau-affordance` prevents wrong behavior from executing at all, then records *what the agent learned* from the wall it hit. This creates a natural curriculum:

1. Agent tries `motor_control` → hits **ApprovalNeeded** wall → learns *"approval is required for this action"*
2. Agent tries `sensor_read` (within budget) → passes → earns growth XP
3. Agent keeps hitting budget wall → learns *"conservation is enforced"*
4. Pathways used more often get reinforced; unused pathways decay

The system conserves a finite energy budget (like `conservation-law-v2` conserves energy in physics), tracks growth through XP and levels, and self-assembles a behavioral DNA tree that reflects the agent's learned habits.

---

## Install

```bash
cargo add lau-affordance
```

Or add to `Cargo.toml`:

```toml
[dependencies]
lau-affordance = "0.1"
```

Requires **Rust 2021 edition**. Dependencies: `serde` + `serde_json`.

---

## Quick Start

```rust
use lau_affordance::*;

// 1. Create an engine with an energy budget
let mut engine = AffordanceEngine::new(100.0);

// 2. Build an action request
let mut req = ActionRequest::new("hermes", "sensor_read");
req.with_param("sensor_id", "temp-1");

// 3. Evaluate walls without executing (preview)
let walls = engine.evaluate(&req);
for wall in &walls {
    println!("Wall: {} (blocking: {})", wall.message(), wall.blocks());
}

// 4. Execute — returns success/failure + everything learned
let result = engine.execute(&req);
println!("{}", result.summary());
// → Success: true | Energy: 0.50 used, 99.50 remaining | Growth: +2.0 XP

// 5. Check engine state
let summary = engine.engine_summary();
println!("{}", summary);
// → Budget: 100.00 | Used: 0.50 | Remaining: 99.50 | Override: false | Pathways: 0 | Conserved: true

// 6. Track growth
println!("{}", engine.growth_tracker.growth_summary());
```

### Registering Custom Actions

```rust
let mut engine = AffordanceEngine::new(100.0);

engine.action_registry.register(ActionDef {
    name: "warp_drive".into(),
    base_cost: 50.0,
    requires_crew: true,
    requires_decomposition: false,
    requires_approval: true,
    triggers_growth: true,
});

let req = ActionRequest::new("helm", "warp_drive");
let result = engine.execute(&req);
assert!(!result.success); // blocked by ApprovalNeeded + CrewRequired
```

### Self-Assembling DNA Pathways

```rust
let mut dna = SelfAssemblingDNA::new();

// Build behavioral pathways through use
dna.use_pathway(&["hardware", "motor", "spin"]);
dna.use_pathway(&["hardware", "sensor", "read"]);
dna.use_pathway(&["hardware", "sensor", "read"]); // reinforce

// Ask the DNA what comes next
let next = dna.suggest_pathway(&["hardware"]);
assert_eq!(next, Some("sensor".to_string())); // most used

// View the full tree
println!("{}", dna.growth_report());
// plato (w=1.00, used=4×)
//   hardware (w=1.00, used=4×)
//     sensor (w=1.00, used=2×)
//     motor (w=0.60, used=1×)

// Prune dead pathways
dna.prune();

// Snapshot for persistence
let snapshot = dna.snapshot();
let json = serde_json::to_string(&snapshot).unwrap();
```

---

## API Reference

### `ActionOrigin`
Where an action came from: `Direct`, `Intention(String)`, `Crew(String)`, `Autonomous`.

### `ActionRequest`
What an agent wants to do.

| Method | Description |
|---|---|
| `new(agent_id, action)` | Create a request with defaults |
| `with_param(key, value)` | Add a parameter |
| `calculate_cost(&mut self, registry)` | Look up energy cost from registry |

Fields: `agent_id`, `action`, `params`, `energy_cost`, `timestamp`, `origin`.

### `WallType`
Enum of wall kinds: `ConservationBudget`, `OverrideActive`, `CrewRequired`, `DecompositionSuggested`, `FieldReading`, `ApprovalNeeded`, `GrowthOpportunity`.

### `AffordanceWall`
A wall the action encounters.

| Method | Returns |
|---|---|
| `blocks()` | Whether this wall blocks execution |
| `message()` | Human-readable explanation |
| `suggestion()` | Optional hint on what to do |

### `ActionDef`
Defines an action type: `name`, `base_cost`, `requires_crew`, `requires_decomposition`, `requires_approval`, `triggers_growth`.

### `ActionRegistry`
Maps action names to definitions and costs. Pre-loaded with 8 actions: `motor_control` (5.0), `sensor_read` (0.5), `intention_submit` (1.0), `crew_activate` (2.0), `field_read` (0.1), `bridge_send` (0.5), `estop` (0.0), `report` (0.0).

| Method | Description |
|---|---|
| `new()` | Create with pre-registered actions |
| `register(def)` | Add a custom action definition |
| `get(name)` | Look up a definition |
| `cost(name)` | Get cost (defaults to 1.0 for unknown) |

### `Lesson`
A learned lesson: `id`, `description`, `learned_at` (tick), `times_reinforced`.

### `GrowthTracker`
Tracks XP, levels, and lessons for a single agent.

| Method | Description |
|---|---|
| `new(agent_id)` | Create a new tracker |
| `learn(lesson, tick)` | Record or reinforce a lesson |
| `xp_for_action(action)` | XP earned per action type |
| `add_xp(amount)` | Add XP (auto-updates level) |
| `level()` | Current level (1 + floor(xp/100)) |
| `top_lessons(n)` | Top-N lessons by reinforcement count |
| `prune_lessons(threshold)` | Remove lessons below reinforcement threshold |
| `growth_summary()` | Human-readable summary string |

### `ExecutionResult`
Result of executing an action through the engine.

| Method | Description |
|---|---|
| `summary()` | Human-readable summary |

Fields: `success`, `walls_hit`, `energy_consumed`, `energy_remaining`, `growth_earned`, `learned`.

### `EngineSummary`
Snapshot of engine state: `budget`, `used`, `remaining`, `override_active`, `pathway_count`, `conserved`. Implements `Display`.

### `AffordanceEngine`
The core engine. Evaluates every action against affordance walls and executes if unblocked.

| Method | Description |
|---|---|
| `new(budget)` | Create engine with energy budget |
| `evaluate(request)` | Preview walls without executing |
| `execute(request)` | Evaluate + execute; returns `ExecutionResult` |
| `is_conserved()` | True if `used ≤ budget` |
| `set_override(active)` | Toggle captain override |
| `suggest_pathway(action)` | Get highest-weighted pathway suggestion |
| `reinforce_pathway(pathway)` | Strengthen a pathway (clamped to 1.0) |
| `prune_unused(threshold)` | Remove pathways below weight threshold |
| `pathway_report()` | String report of all pathways |
| `engine_summary()` | `EngineSummary` snapshot |

### `PathwayNode`
A node in the behavioral DNA tree. Tracks `pathway`, `weight`, `times_used`, `last_used`, `children`.

| Method | Description |
|---|---|
| `use_pathway(tick)` | Increment usage, increase weight (max 1.0) |
| `decay(rate)` | Decrease weight (floor 0.0), recursively |
| `is_active()` | Weight > 0.1 |
| `strongest_child()` | Highest-weight active child |

### `SelfAssemblingDNA`
The pathway evolution system. A tree rooted at `"plato"` that grows as the agent uses pathways.

| Method | Description |
|---|---|
| `new()` | Create with root node |
| `use_pathway(path)` | Walk/create tree path, reinforce each node |
| `suggest_pathway(prefix)` | Predict next segment from strongest child |
| `prune()` | Remove inactive nodes recursively |
| `snapshot()` | Capture `DNASnapshot` for serialization |
| `growth_report()` | Indented tree view of all pathways |

### `DNASnapshot`
Serializable snapshot: `root` (`PathwayNode`) + `tick`.

---

## How It Works

### Evaluation Pipeline

```
ActionRequest → AffordanceEngine.evaluate()
                     ↓
         ┌──── ConservationBudget wall (budget check)
         ├──── OverrideActive wall (if override set)
         ├──── ApprovalNeeded wall (if action requires)
         ├──── CrewRequired wall (if action requires)
         ├──── DecompositionSuggested wall (if action benefits)
         ├──── GrowthOpportunity wall (if action triggers growth)
         └──── FieldReading wall (if action is field_read)
                     ↓
              Vec<AffordanceWall>
```

### Execution

If **any** wall is blocking → action is rejected, energy is preserved, lessons are still recorded from the walls encountered.

If **no** walls are blocking → action executes:
1. Energy cost is deducted from the budget
2. XP is awarded via `GrowthTracker`
3. Lessons are extracted from walls and recorded
4. `ExecutionResult` is returned with full telemetry

### Growth System

- **XP per action**: `motor_control` → 10.0, `sensor_read` → 2.0, `field_read` → 1.0, others → 0.5
- **Levels**: `1 + floor(xp / 100)`
- **Lessons**: Each wall type maps to a canonical lesson string (e.g., `"conservation is enforced"`). Lessons are reinforced when encountered again, building a reinforcement-weighted knowledge base.

### DNA Tree

The `SelfAssemblingDNA` is a self-organizing tree where:
- Each node has a `weight` (0.0–1.0) that increases with use (+0.1 per use, max 1.0)
- `decay(rate)` reduces all weights (floor 0.0) — inactive pathways die
- `prune()` removes nodes with weight ≤ 0.1
- `suggest_pathway` predicts the next action from the strongest child

This creates an evolving behavioral profile: the agent's most-used pathways grow stronger while unused ones atrophy.

---

## The Math

### Conservation Law

The engine enforces a strict conservation invariant:

```
∀ t: used(t) ≤ budget
```

Actions that would violate this invariant are blocked by the `ConservationBudget` wall. This mirrors energy conservation in physical systems — energy cannot be created, only transferred or consumed within budget.

### Affordance as Constraint

Each action type defines a set of preconditions encoded as walls. The affordance model treats these as *environmental constraints* rather than *instructions*:

```
Action × Environment → Wall[] → {Allow | Block}
```

This is analogous to Gibson's theory of affordances in ecological psychology: the environment *offers* possibilities for action, and constraints *shape* behavior without explicit rules.

### Growth as Reinforcement

Learning follows a reinforcement model:

```
lesson_weight(l, t) = Σᵢ₌₁ⁿ 0.5   (where n = times_reinforced)
level(agent) = 1 + ⌊xp / 100⌋
```

### Pathway Evolution

The DNA tree implements a use-dependent plasticity model:

```
weight(n, t+1) = min(weight(n, t) + 0.1, 1.0)    on use
weight(n, t+1) = max(weight(n, t) - decay_rate, 0.0)  on decay
active(n) ⟺ weight(n) > 0.1
```

This is structurally identical to Hebbian learning with decay — pathways used together strengthen together, while unused connections atrophy.

---

## Testing

**63 tests** covering:
- Construction and defaults for all types
- Wall evaluation (blocking vs non-blocking, all 7 wall types)
- Budget conservation (exhaustion, boundary cases, zero-budget actions like `estop`)
- Growth tracking (XP accrual, leveling, lesson reinforcement, pruning)
- DNA tree (pathway creation, reinforcement, suggestion, decay, pruning, snapshots)
- Full serde round-trips for all serializable types
- Integration tests (full workflow, repeated conservation blocking, growth through repetition)

Run: `cargo test`

---

## License

MIT
