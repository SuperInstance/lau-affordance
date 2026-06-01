# lau-affordance

> Environment-as-teacher layer — affordances that guide agent behavior through walls, not instructions

## What This Does

Environment-as-teacher layer — affordances that guide agent behavior through walls, not instructions. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-affordance
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_affordance::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub enum ActionOrigin 
pub struct ActionRequest 
    pub fn new(agent_id: &str, action: &str) -> Self 
    pub fn with_param(&mut self, key: &str, value: &str) 
    pub fn calculate_cost(&mut self, registry: &ActionRegistry) -> f64 
pub enum WallType 
pub struct AffordanceWall 
    pub fn blocks(&self) -> bool 
    pub fn message(&self) -> &str 
    pub fn suggestion(&self) -> Option<&str> 
pub struct ActionDef 
pub struct ActionRegistry 
    pub fn new() -> Self 
    pub fn register(&mut self, def: ActionDef) 
    pub fn get(&self, name: &str) -> Option<&ActionDef> 
    pub fn cost(&self, name: &str) -> f64 
pub struct Lesson 
pub struct GrowthTracker 
    pub fn new(agent_id: &str) -> Self 
    pub fn learn(&mut self, lesson: &str, tick: u64) 
    pub fn xp_for_action(&self, action: &str) -> f64 
    pub fn add_xp(&mut self, amount: f64) 
    pub fn level(&self) -> u32 
    pub fn top_lessons(&self, n: usize) -> Vec<&Lesson> 
    pub fn prune_lessons(&mut self, threshold: u32) 
    pub fn growth_summary(&self) -> String 
pub struct ExecutionResult 
    pub fn summary(&self) -> String 
pub struct EngineSummary 
pub struct AffordanceEngine 
    pub fn new(budget: f64) -> Self 
    pub fn evaluate(&self, request: &ActionRequest) -> Vec<AffordanceWall> 
    pub fn execute(&mut self, request: &ActionRequest) -> ExecutionResult 
    pub fn is_conserved(&self) -> bool 
    pub fn set_override(&mut self, active: bool) 
    pub fn suggest_pathway(&self, action: &str) -> Option<String> 
    pub fn reinforce_pathway(&mut self, pathway: &str) 
    pub fn prune_unused(&mut self, threshold: f64) 
    pub fn pathway_report(&self) -> String 
    pub fn engine_summary(&self) -> EngineSummary 
pub struct PathwayNode 
    pub fn use_pathway(&mut self, tick: u64) 
    pub fn decay(&mut self, rate: f64) 
    pub fn is_active(&self) -> bool 
    pub fn strongest_child(&self) -> Option<&PathwayNode> 
pub struct DNASnapshot 
pub struct SelfAssemblingDNA 
    pub fn new() -> Self 
    pub fn use_pathway(&mut self, path: &[&str]) 
    pub fn suggest_pathway(&self, prefix: &[&str]) -> Option<String> 
    pub fn prune(&mut self) 
    pub fn snapshot(&self) -> DNASnapshot 
    pub fn growth_report(&self) -> String 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**63 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
