//! # lau-affordance
//!
//! The environment-as-teacher layer. What conservation-law-v2 is to energy,
//! lau-affordance is to learning — it makes the system teach correct behavior
//! through affordances, not instructions.
//!
//! Every agent action hits "walls" that naturally guide behavior. The model
//! learns because the environment won't let it do the wrong thing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// ActionOrigin
// ---------------------------------------------------------------------------

/// Where an action originated from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionOrigin {
    Direct,
    Intention(String),
    Crew(String),
    Autonomous,
}

// ---------------------------------------------------------------------------
// ActionRequest
// ---------------------------------------------------------------------------

/// What an agent wants to do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub agent_id: String,
    pub action: String,
    pub params: HashMap<String, String>,
    pub energy_cost: f64,
    pub timestamp: u64,
    pub origin: ActionOrigin,
}

impl ActionRequest {
    pub fn new(agent_id: &str, action: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            action: action.to_string(),
            params: HashMap::new(),
            energy_cost: 0.0,
            timestamp: 0,
            origin: ActionOrigin::Direct,
        }
    }

    pub fn with_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }

    pub fn calculate_cost(&mut self, registry: &ActionRegistry) -> f64 {
        self.energy_cost = registry.cost(&self.action);
        self.energy_cost
    }
}

// ---------------------------------------------------------------------------
// WallType
// ---------------------------------------------------------------------------

/// The kind of wall an action hits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WallType {
    ConservationBudget,
    OverrideActive,
    CrewRequired,
    DecompositionSuggested,
    FieldReading,
    ApprovalNeeded,
    GrowthOpportunity,
}

// ---------------------------------------------------------------------------
// AffordanceWall
// ---------------------------------------------------------------------------

/// A wall the action encounters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffordanceWall {
    pub wall_type: WallType,
    pub message: String,
    pub blocking: bool,
    pub suggestion: Option<String>,
}

impl AffordanceWall {
    pub fn blocks(&self) -> bool {
        self.blocking
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }
}

// ---------------------------------------------------------------------------
// ActionDef & ActionRegistry
// ---------------------------------------------------------------------------

/// Definition of an action type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    pub name: String,
    pub base_cost: f64,
    pub requires_crew: bool,
    pub requires_decomposition: bool,
    pub requires_approval: bool,
    pub triggers_growth: bool,
}

/// Maps action names to costs and wall requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRegistry {
    pub actions: HashMap<String, ActionDef>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            actions: HashMap::new(),
        };
        // Pre-registered actions
        reg.register(ActionDef {
            name: "motor_control".into(),
            base_cost: 5.0,
            requires_crew: false,
            requires_decomposition: false,
            requires_approval: true,
            triggers_growth: true,
        });
        reg.register(ActionDef {
            name: "sensor_read".into(),
            base_cost: 0.5,
            requires_crew: false,
            requires_decomposition: false,
            requires_approval: false,
            triggers_growth: true,
        });
        reg.register(ActionDef {
            name: "intention_submit".into(),
            base_cost: 1.0,
            requires_crew: false,
            requires_decomposition: true,
            requires_approval: false,
            triggers_growth: false,
        });
        reg.register(ActionDef {
            name: "crew_activate".into(),
            base_cost: 2.0,
            requires_crew: false,
            requires_decomposition: false,
            requires_approval: true,
            triggers_growth: false,
        });
        reg.register(ActionDef {
            name: "field_read".into(),
            base_cost: 0.1,
            requires_crew: false,
            requires_decomposition: false,
            requires_approval: false,
            triggers_growth: false,
        });
        reg.register(ActionDef {
            name: "bridge_send".into(),
            base_cost: 0.5,
            requires_crew: false,
            requires_decomposition: false,
            requires_approval: false,
            triggers_growth: false,
        });
        reg.register(ActionDef {
            name: "estop".into(),
            base_cost: 0.0,
            requires_crew: false,
            requires_decomposition: false,
            requires_approval: false,
            triggers_growth: false,
        });
        reg.register(ActionDef {
            name: "report".into(),
            base_cost: 0.0,
            requires_crew: false,
            requires_decomposition: false,
            requires_approval: false,
            triggers_growth: false,
        });
        reg
    }

    pub fn register(&mut self, def: ActionDef) {
        self.actions.insert(def.name.clone(), def);
    }

    pub fn get(&self, name: &str) -> Option<&ActionDef> {
        self.actions.get(name)
    }

    pub fn cost(&self, name: &str) -> f64 {
        self.actions.get(name).map(|d| d.base_cost).unwrap_or(1.0)
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Lesson & GrowthTracker
// ---------------------------------------------------------------------------

/// A lesson the agent has learned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub description: String,
    pub learned_at: u64,
    pub times_reinforced: u32,
}

/// Tracks what the agent has learned over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthTracker {
    pub agent_id: String,
    pub xp: f64,
    pub level: u32,
    pub learned_lessons: Vec<Lesson>,
    pub lesson_weights: HashMap<String, f64>,
}

impl GrowthTracker {
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            xp: 0.0,
            level: 1,
            learned_lessons: Vec::new(),
            lesson_weights: HashMap::new(),
        }
    }

    pub fn learn(&mut self, lesson: &str, tick: u64) {
        let id = lesson.to_lowercase().replace(' ', "_");
        if let Some(existing) = self.learned_lessons.iter_mut().find(|l| l.id == id) {
            existing.times_reinforced += 1;
            existing.learned_at = tick;
            *self.lesson_weights.entry(id.clone()).or_insert(1.0) += 0.5;
        } else {
            self.learned_lessons.push(Lesson {
                id: id.clone(),
                description: lesson.to_string(),
                learned_at: tick,
                times_reinforced: 1,
            });
            self.lesson_weights.insert(id, 1.0);
        }
    }

    pub fn xp_for_action(&self, action: &str) -> f64 {
        match action {
            "motor_control" => 10.0,
            "sensor_read" => 2.0,
            "field_read" => 1.0,
            _ => 0.5,
        }
    }

    pub fn add_xp(&mut self, amount: f64) {
        self.xp += amount;
        self.level = 1 + (self.xp / 100.0).floor() as u32;
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn top_lessons(&self, n: usize) -> Vec<&Lesson> {
        let mut lessons: Vec<_> = self.learned_lessons.iter().collect();
        lessons.sort_by_key(|b| std::cmp::Reverse(b.times_reinforced));
        lessons.into_iter().take(n).collect()
    }

    pub fn prune_lessons(&mut self, threshold: u32) {
        self.learned_lessons.retain(|l| l.times_reinforced >= threshold);
    }

    pub fn growth_summary(&self) -> String {
        let top: Vec<String> = self.top_lessons(5)
            .iter()
            .map(|l| format!("  - {} (×{})", l.description, l.times_reinforced))
            .collect();
        format!(
            "Agent {} | Level {} | XP {:.1}/{}\nTop lessons:\n{}",
            self.agent_id,
            self.level,
            self.xp,
            self.level * 100,
            if top.is_empty() { "  (none yet)".to_string() } else { top.join("\n") }
        )
    }
}

// ---------------------------------------------------------------------------
// ExecutionResult
// ---------------------------------------------------------------------------

/// Result of executing an action through the affordance engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub walls_hit: Vec<AffordanceWall>,
    pub energy_consumed: f64,
    pub energy_remaining: f64,
    pub growth_earned: f64,
    pub learned: Vec<String>,
}

impl ExecutionResult {
    pub fn summary(&self) -> String {
        let walls: Vec<String> = self.walls_hit.iter().map(|w| w.message.clone()).collect();
        format!(
            "Success: {} | Energy: {:.2} used, {:.2} remaining | Growth: +{:.1} XP | Walls: [{}] | Learned: [{}]",
            self.success,
            self.energy_consumed,
            self.energy_remaining,
            self.growth_earned,
            if walls.is_empty() { "none".to_string() } else { walls.join("; ") },
            self.learned.join(", "),
        )
    }
}

// ---------------------------------------------------------------------------
// EngineSummary
// ---------------------------------------------------------------------------

/// Summary of the engine state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSummary {
    pub budget: f64,
    pub used: f64,
    pub remaining: f64,
    pub override_active: bool,
    pub pathway_count: usize,
    pub conserved: bool,
}

// ---------------------------------------------------------------------------
// AffordanceEngine
// ---------------------------------------------------------------------------

/// THE engine that evaluates every action against affordance walls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffordanceEngine {
    pub budget: f64,
    pub used: f64,
    pub override_active: bool,
    pub action_registry: ActionRegistry,
    pub growth_tracker: GrowthTracker,
    pub pathway_weights: HashMap<String, f64>,
}

impl AffordanceEngine {
    pub fn new(budget: f64) -> Self {
        Self {
            budget,
            used: 0.0,
            override_active: false,
            action_registry: ActionRegistry::new(),
            growth_tracker: GrowthTracker::new("default"),
            pathway_weights: HashMap::new(),
        }
    }

    /// Evaluate all walls this action hits (without executing).
    pub fn evaluate(&self, request: &ActionRequest) -> Vec<AffordanceWall> {
        let mut walls = Vec::new();
        let cost = self.action_registry.cost(&request.action);

        // Conservation budget wall
        if self.used + cost > self.budget {
            walls.push(AffordanceWall {
                wall_type: WallType::ConservationBudget,
                message: format!(
                    "Energy budget exceeded: need {:.2}, have {:.2} remaining",
                    cost,
                    self.budget - self.used
                ),
                blocking: true,
                suggestion: Some("Reduce energy consumption or request budget increase".into()),
            });
        }

        // Override wall
        if self.override_active {
            walls.push(AffordanceWall {
                wall_type: WallType::OverrideActive,
                message: "Captain override is active — actions may be redirected".into(),
                blocking: false,
                suggestion: Some("Follow override directives or wait for override to clear".into()),
            });
        }

        // Action-specific walls
        if let Some(def) = self.action_registry.get(&request.action) {
            if def.requires_approval {
                walls.push(AffordanceWall {
                    wall_type: WallType::ApprovalNeeded,
                    message: format!("'{}' requires approval before execution", request.action),
                    blocking: true,
                    suggestion: Some("Submit for approval or use an approved pathway".into()),
                });
            }

            if def.requires_crew {
                walls.push(AffordanceWall {
                    wall_type: WallType::CrewRequired,
                    message: format!("'{}' requires crew assignment", request.action),
                    blocking: true,
                    suggestion: Some("Assign a crew before executing".into()),
                });
            }

            if def.requires_decomposition {
                walls.push(AffordanceWall {
                    wall_type: WallType::DecompositionSuggested,
                    message: format!("'{}' benefits from decomposition into sub-intentions", request.action),
                    blocking: false,
                    suggestion: Some("Decompose into smaller, verifiable steps".into()),
                });
            }

            if def.triggers_growth {
                walls.push(AffordanceWall {
                    wall_type: WallType::GrowthOpportunity,
                    message: format!("'{}' triggers growth tracking", request.action),
                    blocking: false,
                    suggestion: None,
                });
            }

            if request.action == "field_read" {
                walls.push(AffordanceWall {
                    wall_type: WallType::FieldReading,
                    message: "Reading from field — data may be stale or incomplete".into(),
                    blocking: false,
                    suggestion: Some("Cross-reference with recent sensor data".into()),
                });
            }
        }

        walls
    }

    /// Evaluate walls and execute if not blocked.
    pub fn execute(&mut self, request: &ActionRequest) -> ExecutionResult {
        let walls = self.evaluate(request);
        let blocked = walls.iter().any(|w| w.blocking);
        let cost = self.action_registry.cost(&request.action);

        let mut learned: Vec<String> = Vec::new();

        // Extract lessons from walls
        for w in &walls {
            match w.wall_type {
                WallType::ConservationBudget => learned.push("conservation is enforced".to_string()),
                WallType::OverrideActive => learned.push("override was active".to_string()),
                WallType::ApprovalNeeded => learned.push("approval is required for this action".to_string()),
                WallType::CrewRequired => learned.push("crew assignment is required".to_string()),
                WallType::DecompositionSuggested => learned.push("decomposition improves reliability".to_string()),
                WallType::FieldReading => learned.push("field data may be incomplete".to_string()),
                WallType::GrowthOpportunity => learned.push("action contributed to growth".to_string()),
            }
        }

        let (success, energy_consumed, growth_earned) = if blocked {
            (false, 0.0, 0.0)
        } else {
            self.used += cost;
            let xp = self.growth_tracker.xp_for_action(&request.action);
            self.growth_tracker.add_xp(xp);

            // Record lessons
            for lesson in &learned {
                self.growth_tracker.learn(lesson, request.timestamp);
            }

            (true, cost, xp)
        };

        ExecutionResult {
            success,
            walls_hit: walls,
            energy_consumed,
            energy_remaining: self.budget - self.used,
            growth_earned,
            learned,
        }
    }

    pub fn is_conserved(&self) -> bool {
        self.used <= self.budget
    }

    pub fn set_override(&mut self, active: bool) {
        self.override_active = active;
    }

    pub fn suggest_pathway(&self, action: &str) -> Option<String> {
        // Find the pathway with highest weight related to this action
        let key = format!("pathway:{}", action);
        self.pathway_weights.get(&key).map(|w| {
            format!("pathway:{} (weight: {:.2})", action, w)
        }).or_else(|| {
            // Return the highest-weighted pathway overall
            self.pathway_weights
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(k, w)| format!("{} (weight: {:.2})", k, w))
        })
    }

    pub fn reinforce_pathway(&mut self, pathway: &str) {
        let entry = self.pathway_weights.entry(pathway.to_string()).or_insert(0.0);
        *entry = (*entry + 1.0).min(1.0);
    }

    pub fn prune_unused(&mut self, threshold: f64) {
        self.pathway_weights.retain(|_, w| *w >= threshold);
    }

    pub fn pathway_report(&self) -> String {
        if self.pathway_weights.is_empty() {
            return "No pathways recorded yet.".to_string();
        }
        let mut entries: Vec<_> = self.pathway_weights.iter().collect();
        entries.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        let lines: Vec<String> = entries
            .iter()
            .map(|(k, w)| format!("  {} → {:.2}", k, w))
            .collect();
        format!("Pathways ({} total):\n{}", entries.len(), lines.join("\n"))
    }

    pub fn engine_summary(&self) -> EngineSummary {
        EngineSummary {
            budget: self.budget,
            used: self.used,
            remaining: self.budget - self.used,
            override_active: self.override_active,
            pathway_count: self.pathway_weights.len(),
            conserved: self.is_conserved(),
        }
    }
}

// ---------------------------------------------------------------------------
// PathwayNode & SelfAssemblingDNA
// ---------------------------------------------------------------------------

/// A node in the self-assembling pathway tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathwayNode {
    pub pathway: String,
    pub weight: f64,
    pub times_used: u32,
    pub last_used: u64,
    pub children: Vec<PathwayNode>,
}

impl PathwayNode {
    pub fn use_pathway(&mut self, tick: u64) {
        self.times_used += 1;
        self.last_used = tick;
        self.weight = (self.weight + 0.1).min(1.0);
    }

    pub fn decay(&mut self, rate: f64) {
        self.weight = (self.weight - rate).max(0.0);
        for child in &mut self.children {
            child.decay(rate);
        }
    }

    pub fn is_active(&self) -> bool {
        self.weight > 0.1
    }

    pub fn strongest_child(&self) -> Option<&PathwayNode> {
        self.children
            .iter()
            .filter(|c| c.is_active())
            .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal))
    }
}

/// Snapshot of the DNA tree for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNASnapshot {
    pub root: PathwayNode,
    pub tick: u64,
}

/// The pathway evolution system — self-assembling DNA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAssemblingDNA {
    pub root: PathwayNode,
    pub tick: u64,
    pub decay_rate: f64,
}

impl SelfAssemblingDNA {
    pub fn new() -> Self {
        Self {
            root: PathwayNode {
                pathway: "plato".into(),
                weight: 1.0,
                times_used: 1,
                last_used: 0,
                children: Vec::new(),
            },
            tick: 0,
            decay_rate: 0.01,
        }
    }

    fn find_or_create_child<'a>(parent: &'a mut PathwayNode, name: &str) -> &'a mut PathwayNode {
        let idx = parent.children.iter().position(|c| c.pathway == name);
        let idx = idx.unwrap_or_else(|| {
            parent.children.push(PathwayNode {
                pathway: name.to_string(),
                weight: 0.5,
                times_used: 0,
                last_used: 0,
                children: Vec::new(),
            });
            parent.children.len() - 1
        });
        &mut parent.children[idx]
    }

    pub fn use_pathway(&mut self, path: &[&str]) {
        self.tick += 1;
        let tick = self.tick;
        // Walk the tree, creating nodes as needed
        let mut node = &mut self.root;
        node.use_pathway(tick);
        for segment in path {
            let segment = *segment;
            node = Self::find_or_create_child(node, segment);
            node.use_pathway(tick);
        }
    }

    pub fn suggest_pathway(&self, prefix: &[&str]) -> Option<String> {
        let mut node = &self.root;
        for segment in prefix {
            node = node.children.iter().find(|c| &c.pathway == segment)?;
        }
        node.strongest_child().map(|c| c.pathway.clone())
    }

    pub fn prune(&mut self) {
        Self::prune_node(&mut self.root);
    }

    fn prune_node(node: &mut PathwayNode) {
        node.children.retain(|c| c.is_active());
        for child in &mut node.children {
            Self::prune_node(child);
        }
    }

    pub fn snapshot(&self) -> DNASnapshot {
        DNASnapshot {
            root: self.root.clone(),
            tick: self.tick,
        }
    }

    pub fn growth_report(&self) -> String {
        let mut lines = Vec::new();
        Self::report_node(&self.root, 0, &mut lines);
        lines.join("\n")
    }

    fn report_node(node: &PathwayNode, depth: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(depth);
        lines.push(format!(
            "{}{} (w={:.2}, used={}×)",
            indent, node.pathway, node.weight, node.times_used
        ));
        let mut children: Vec<_> = node.children.iter().collect();
        children.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
        for child in children {
            Self::report_node(child, depth + 1, lines);
        }
    }
}

impl Default for SelfAssemblingDNA {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

impl fmt::Display for EngineSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Budget: {:.2} | Used: {:.2} | Remaining: {:.2} | Override: {} | Pathways: {} | Conserved: {}",
            self.budget, self.used, self.remaining, self.override_active, self.pathway_count, self.conserved
        )
    }
}

// ===========================================================================
// TESTS
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- ActionRequest ---
    #[test]
    fn test_action_request_new() {
        let req = ActionRequest::new("hermes", "sensor_read");
        assert_eq!(req.agent_id, "hermes");
        assert_eq!(req.action, "sensor_read");
        assert!(req.params.is_empty());
        assert_eq!(req.origin, ActionOrigin::Direct);
    }

    #[test]
    fn test_action_request_with_param() {
        let mut req = ActionRequest::new("hermes", "motor_control");
        req.with_param("speed", "100");
        assert_eq!(req.params.get("speed").unwrap(), "100");
    }

    #[test]
    fn test_action_request_calculate_cost() {
        let reg = ActionRegistry::new();
        let mut req = ActionRequest::new("hermes", "motor_control");
        let cost = req.calculate_cost(&reg);
        assert_eq!(cost, 5.0);
        assert_eq!(req.energy_cost, 5.0);
    }

    #[test]
    fn test_action_request_calculate_cost_unknown() {
        let reg = ActionRegistry::new();
        let mut req = ActionRequest::new("hermes", "unknown_action");
        let cost = req.calculate_cost(&reg);
        assert_eq!(cost, 1.0); // default
    }

    #[test]
    fn test_action_origin_variants() {
        let origins = vec![
            ActionOrigin::Direct,
            ActionOrigin::Intention("int-123".into()),
            ActionOrigin::Crew("alpha".into()),
            ActionOrigin::Autonomous,
        ];
        assert_eq!(origins.len(), 4);
        assert_eq!(ActionOrigin::Direct, ActionOrigin::Direct);
    }

    // --- ActionRegistry ---
    #[test]
    fn test_registry_preregistered_actions() {
        let reg = ActionRegistry::new();
        assert_eq!(reg.cost("motor_control"), 5.0);
        assert_eq!(reg.cost("sensor_read"), 0.5);
        assert_eq!(reg.cost("intention_submit"), 1.0);
        assert_eq!(reg.cost("crew_activate"), 2.0);
        assert_eq!(reg.cost("field_read"), 0.1);
        assert_eq!(reg.cost("bridge_send"), 0.5);
        assert_eq!(reg.cost("estop"), 0.0);
        assert_eq!(reg.cost("report"), 0.0);
    }

    #[test]
    fn test_registry_unknown_action_cost() {
        let reg = ActionRegistry::new();
        assert_eq!(reg.cost("nonexistent"), 1.0);
    }

    #[test]
    fn test_registry_get() {
        let reg = ActionRegistry::new();
        let def = reg.get("motor_control").unwrap();
        assert!(def.requires_approval);
        assert!(def.triggers_growth);
        assert_eq!(def.base_cost, 5.0);
    }

    #[test]
    fn test_registry_get_missing() {
        let reg = ActionRegistry::new();
        assert!(reg.get("nope").is_none());
    }

    #[test]
    fn test_registry_register_custom() {
        let mut reg = ActionRegistry::new();
        reg.register(ActionDef {
            name: "custom".into(),
            base_cost: 42.0,
            requires_crew: true,
            requires_decomposition: false,
            requires_approval: false,
            triggers_growth: false,
        });
        assert_eq!(reg.cost("custom"), 42.0);
        let def = reg.get("custom").unwrap();
        assert!(def.requires_crew);
    }

    #[test]
    fn test_registry_default() {
        let reg = ActionRegistry::default();
        assert_eq!(reg.cost("motor_control"), 5.0);
    }

    // --- AffordanceWall ---
    #[test]
    fn test_wall_blocking() {
        let wall = AffordanceWall {
            wall_type: WallType::ConservationBudget,
            message: "budget exceeded".into(),
            blocking: true,
            suggestion: Some("reduce usage".into()),
        };
        assert!(wall.blocks());
        assert_eq!(wall.message(), "budget exceeded");
        assert_eq!(wall.suggestion(), Some("reduce usage"));
    }

    #[test]
    fn test_wall_non_blocking() {
        let wall = AffordanceWall {
            wall_type: WallType::GrowthOpportunity,
            message: "growth!".into(),
            blocking: false,
            suggestion: None,
        };
        assert!(!wall.blocks());
        assert!(wall.suggestion().is_none());
    }

    // --- AffordanceEngine ---
    #[test]
    fn test_engine_new() {
        let engine = AffordanceEngine::new(100.0);
        assert_eq!(engine.budget, 100.0);
        assert_eq!(engine.used, 0.0);
        assert!(!engine.override_active);
        assert!(engine.is_conserved());
    }

    #[test]
    fn test_engine_execute_simple() {
        let mut engine = AffordanceEngine::new(100.0);
        let req = ActionRequest::new("hermes", "sensor_read");
        let result = engine.execute(&req);
        assert!(result.success);
        assert_eq!(result.energy_consumed, 0.5);
        assert_eq!(result.energy_remaining, 99.5);
        assert!(!result.walls_hit.iter().any(|w| w.blocks()));
    }

    #[test]
    fn test_engine_execute_blocks_on_approval() {
        let mut engine = AffordanceEngine::new(100.0);
        let req = ActionRequest::new("hermes", "motor_control");
        let result = engine.execute(&req);
        assert!(!result.success);
        assert_eq!(result.energy_consumed, 0.0);
        assert!(result.walls_hit.iter().any(|w| w.wall_type == WallType::ApprovalNeeded && w.blocks()));
    }

    #[test]
    fn test_engine_budget_exhaustion() {
        let mut engine = AffordanceEngine::new(1.0);
        // sensor_read costs 0.5
        let req1 = ActionRequest::new("hermes", "sensor_read");
        let result1 = engine.execute(&req1);
        assert!(result1.success);
        // motor_control costs 5.0 — over budget
        let req2 = ActionRequest::new("hermes", "motor_control");
        let result2 = engine.execute(&req2);
        // blocked by both budget and approval
        assert!(!result2.success);
        assert!(result2.walls_hit.iter().any(|w| w.wall_type == WallType::ConservationBudget));
    }

    #[test]
    fn test_engine_is_conserved() {
        let mut engine = AffordanceEngine::new(1.0);
        assert!(engine.is_conserved());
        // use up budget
        let req = ActionRequest::new("hermes", "bridge_send");
        engine.execute(&req); // 0.5
        engine.execute(&req); // 1.0 total
        assert!(engine.is_conserved());
        engine.used = 1.1;
        assert!(!engine.is_conserved());
    }

    #[test]
    fn test_engine_override() {
        let mut engine = AffordanceEngine::new(100.0);
        engine.set_override(true);
        assert!(engine.override_active);
        let req = ActionRequest::new("hermes", "sensor_read");
        let result = engine.execute(&req);
        assert!(result.success);
        assert!(result.walls_hit.iter().any(|w| w.wall_type == WallType::OverrideActive));
        assert!(result.learned.contains(&"override was active".to_string()));
    }

    #[test]
    fn test_engine_set_override_false() {
        let mut engine = AffordanceEngine::new(100.0);
        engine.set_override(true);
        engine.set_override(false);
        assert!(!engine.override_active);
    }

    #[test]
    fn test_engine_pathway_reinforce() {
        let mut engine = AffordanceEngine::new(100.0);
        engine.reinforce_pathway("sensor_first");
        engine.reinforce_pathway("sensor_first");
        assert_eq!(engine.pathway_weights.get("sensor_first"), Some(&1.0));
    }

    #[test]
    fn test_engine_suggest_pathway() {
        let mut engine = AffordanceEngine::new(100.0);
        engine.reinforce_pathway("pathway:sensor_read");
        engine.reinforce_pathway("pathway:sensor_read");
        let suggestion = engine.suggest_pathway("sensor_read");
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_engine_suggest_pathway_empty() {
        let engine = AffordanceEngine::new(100.0);
        assert!(engine.suggest_pathway("anything").is_none());
    }

    #[test]
    fn test_engine_prune_unused() {
        let mut engine = AffordanceEngine::new(100.0);
        engine.reinforce_pathway("keep_me");
        engine.pathway_weights.insert("low".into(), 0.05);
        engine.prune_unused(0.1);
        assert!(engine.pathway_weights.contains_key("keep_me"));
        assert!(!engine.pathway_weights.contains_key("low"));
    }

    #[test]
    fn test_engine_pathway_report() {
        let mut engine = AffordanceEngine::new(100.0);
        let report = engine.pathway_report();
        assert!(report.contains("No pathways"));
        engine.reinforce_pathway("test_path");
        let report = engine.pathway_report();
        assert!(report.contains("test_path"));
    }

    #[test]
    fn test_engine_engine_summary() {
        let engine = AffordanceEngine::new(100.0);
        let summary = engine.engine_summary();
        assert_eq!(summary.budget, 100.0);
        assert!(summary.conserved);
        assert!(!summary.override_active);
        let display = format!("{}", summary);
        assert!(display.contains("100.00"));
    }

    #[test]
    fn test_engine_field_read_wall() {
        let mut engine = AffordanceEngine::new(100.0);
        let req = ActionRequest::new("hermes", "field_read");
        let result = engine.execute(&req);
        assert!(result.success);
        assert!(result.walls_hit.iter().any(|w| w.wall_type == WallType::FieldReading));
    }

    #[test]
    fn test_engine_intention_submit_wall() {
        let engine = AffordanceEngine::new(100.0);
        let req = ActionRequest::new("hermes", "intention_submit");
        let walls = engine.evaluate(&req);
        assert!(walls.iter().any(|w| w.wall_type == WallType::DecompositionSuggested));
    }

    #[test]
    fn test_engine_estop_free() {
        let mut engine = AffordanceEngine::new(0.0);
        let req = ActionRequest::new("hermes", "estop");
        let result = engine.execute(&req);
        assert!(result.success);
        assert_eq!(result.energy_consumed, 0.0);
    }

    #[test]
    fn test_engine_report_free() {
        let mut engine = AffordanceEngine::new(0.0);
        let req = ActionRequest::new("hermes", "report");
        let result = engine.execute(&req);
        assert!(result.success);
    }

    // --- ExecutionResult ---
    #[test]
    fn test_execution_result_summary() {
        let result = ExecutionResult {
            success: true,
            walls_hit: vec![],
            energy_consumed: 0.5,
            energy_remaining: 99.5,
            growth_earned: 2.0,
            learned: vec!["something".into()],
        };
        let s = result.summary();
        assert!(s.contains("true"));
        assert!(s.contains("0.50"));
        assert!(s.contains("99.50"));
        assert!(s.contains("something"));
    }

    #[test]
    fn test_execution_result_blocked_summary() {
        let result = ExecutionResult {
            success: false,
            walls_hit: vec![AffordanceWall {
                wall_type: WallType::ConservationBudget,
                message: "over budget".into(),
                blocking: true,
                suggestion: None,
            }],
            energy_consumed: 0.0,
            energy_remaining: 0.0,
            growth_earned: 0.0,
            learned: vec!["conservation is enforced".into()],
        };
        let s = result.summary();
        assert!(s.contains("false"));
        assert!(s.contains("over budget"));
    }

    // --- GrowthTracker ---
    #[test]
    fn test_growth_tracker_new() {
        let tracker = GrowthTracker::new("hermes");
        assert_eq!(tracker.agent_id, "hermes");
        assert_eq!(tracker.xp, 0.0);
        assert_eq!(tracker.level(), 1);
    }

    #[test]
    fn test_growth_tracker_add_xp() {
        let mut tracker = GrowthTracker::new("hermes");
        tracker.add_xp(50.0);
        assert_eq!(tracker.xp, 50.0);
        assert_eq!(tracker.level(), 1);
        tracker.add_xp(50.0);
        assert_eq!(tracker.level(), 2);
        tracker.add_xp(100.0);
        assert_eq!(tracker.level(), 3);
    }

    #[test]
    fn test_growth_tracker_learn() {
        let mut tracker = GrowthTracker::new("hermes");
        tracker.learn("conservation is enforced", 1);
        assert_eq!(tracker.learned_lessons.len(), 1);
        assert_eq!(tracker.learned_lessons[0].times_reinforced, 1);
        tracker.learn("conservation is enforced", 2);
        assert_eq!(tracker.learned_lessons.len(), 1);
        assert_eq!(tracker.learned_lessons[0].times_reinforced, 2);
    }

    #[test]
    fn test_growth_tracker_xp_for_action() {
        let tracker = GrowthTracker::new("hermes");
        assert_eq!(tracker.xp_for_action("motor_control"), 10.0);
        assert_eq!(tracker.xp_for_action("sensor_read"), 2.0);
        assert_eq!(tracker.xp_for_action("field_read"), 1.0);
        assert_eq!(tracker.xp_for_action("unknown"), 0.5);
    }

    #[test]
    fn test_growth_tracker_top_lessons() {
        let mut tracker = GrowthTracker::new("hermes");
        tracker.learn("a", 1);
        tracker.learn("b", 1);
        tracker.learn("b", 2);
        tracker.learn("c", 1);
        tracker.learn("c", 2);
        tracker.learn("c", 3);
        let top = tracker.top_lessons(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].description, "c");
        assert_eq!(top[1].description, "b");
    }

    #[test]
    fn test_growth_tracker_prune_lessons() {
        let mut tracker = GrowthTracker::new("hermes");
        tracker.learn("keep", 1);
        tracker.learn("keep", 2);
        tracker.learn("keep", 3);
        tracker.learn("remove", 1);
        tracker.prune_lessons(2);
        assert_eq!(tracker.learned_lessons.len(), 1);
        assert_eq!(tracker.learned_lessons[0].description, "keep");
    }

    #[test]
    fn test_growth_tracker_summary() {
        let mut tracker = GrowthTracker::new("hermes");
        tracker.add_xp(25.0);
        tracker.learn("test lesson", 1);
        let summary = tracker.growth_summary();
        assert!(summary.contains("hermes"));
        assert!(summary.contains("Level 1"));
        assert!(summary.contains("test lesson"));
    }

    // --- PathwayNode ---
    #[test]
    fn test_pathway_node_use() {
        let mut node = PathwayNode {
            pathway: "test".into(),
            weight: 0.5,
            times_used: 0,
            last_used: 0,
            children: vec![],
        };
        node.use_pathway(10);
        assert_eq!(node.times_used, 1);
        assert_eq!(node.last_used, 10);
        assert!((node.weight - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pathway_node_use_max_weight() {
        let mut node = PathwayNode {
            pathway: "test".into(),
            weight: 0.95,
            times_used: 0,
            last_used: 0,
            children: vec![],
        };
        node.use_pathway(1);
        assert!((node.weight - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pathway_node_decay() {
        let mut node = PathwayNode {
            pathway: "test".into(),
            weight: 0.5,
            times_used: 1,
            last_used: 1,
            children: vec![],
        };
        node.decay(0.1);
        assert!((node.weight - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pathway_node_decay_floor() {
        let mut node = PathwayNode {
            pathway: "test".into(),
            weight: 0.05,
            times_used: 1,
            last_used: 1,
            children: vec![],
        };
        node.decay(0.1);
        assert!((node.weight - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pathway_node_is_active() {
        let node = PathwayNode {
            pathway: "test".into(),
            weight: 0.5,
            times_used: 1,
            last_used: 1,
            children: vec![],
        };
        assert!(node.is_active());
        let inactive = PathwayNode {
            pathway: "test".into(),
            weight: 0.05,
            times_used: 1,
            last_used: 1,
            children: vec![],
        };
        assert!(!inactive.is_active());
    }

    #[test]
    fn test_pathway_node_strongest_child() {
        let parent = PathwayNode {
            pathway: "root".into(),
            weight: 1.0,
            times_used: 1,
            last_used: 1,
            children: vec![
                PathwayNode {
                    pathway: "weak".into(),
                    weight: 0.2,
                    times_used: 1,
                    last_used: 1,
                    children: vec![],
                },
                PathwayNode {
                    pathway: "strong".into(),
                    weight: 0.8,
                    times_used: 5,
                    last_used: 5,
                    children: vec![],
                },
            ],
        };
        let strongest = parent.strongest_child().unwrap();
        assert_eq!(strongest.pathway, "strong");
    }

    #[test]
    fn test_pathway_node_strongest_child_none() {
        let parent = PathwayNode {
            pathway: "root".into(),
            weight: 1.0,
            times_used: 1,
            last_used: 1,
            children: vec![],
        };
        assert!(parent.strongest_child().is_none());
    }

    // --- SelfAssemblingDNA ---
    #[test]
    fn test_dna_new() {
        let dna = SelfAssemblingDNA::new();
        assert_eq!(dna.root.pathway, "plato");
        assert!((dna.root.weight - 1.0).abs() < f64::EPSILON);
        assert_eq!(dna.tick, 0);
    }

    #[test]
    fn test_dna_use_pathway() {
        let mut dna = SelfAssemblingDNA::new();
        dna.use_pathway(&["hardware", "motor", "spin"]);
        assert_eq!(dna.tick, 1);
        assert_eq!(dna.root.children.len(), 1);
        let hw = &dna.root.children[0];
        assert_eq!(hw.pathway, "hardware");
        assert_eq!(hw.children.len(), 1);
        assert_eq!(hw.children[0].pathway, "motor");
        assert_eq!(hw.children[0].children[0].pathway, "spin");
    }

    #[test]
    fn test_dna_use_pathway_reinforce() {
        let mut dna = SelfAssemblingDNA::new();
        dna.use_pathway(&["hardware", "motor"]);
        dna.use_pathway(&["hardware", "motor"]);
        let hw = dna.root.children.iter().find(|c| c.pathway == "hardware").unwrap();
        let motor = hw.children.iter().find(|c| c.pathway == "motor").unwrap();
        assert_eq!(motor.times_used, 2);
    }

    #[test]
    fn test_dna_suggest_pathway() {
        let mut dna = SelfAssemblingDNA::new();
        dna.use_pathway(&["hardware", "motor"]);
        dna.use_pathway(&["hardware", "sensor"]);
        dna.use_pathway(&["hardware", "sensor"]);
        let suggestion = dna.suggest_pathway(&["hardware"]);
        assert_eq!(suggestion, Some("sensor".to_string()));
    }

    #[test]
    fn test_dna_suggest_pathway_no_match() {
        let dna = SelfAssemblingDNA::new();
        let suggestion = dna.suggest_pathway(&["nonexistent"]);
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_dna_prune() {
        let mut dna = SelfAssemblingDNA::new();
        dna.use_pathway(&["hardware", "motor"]);
        // Manually set a child to inactive
        dna.root.children[0].children[0].weight = 0.05;
        dna.prune();
        // motor should be pruned from hardware's children
        assert!(dna.root.children[0].children.is_empty());
    }

    #[test]
    fn test_dna_snapshot() {
        let mut dna = SelfAssemblingDNA::new();
        dna.use_pathway(&["test"]);
        let snap = dna.snapshot();
        assert_eq!(snap.tick, 1);
        assert_eq!(snap.root.pathway, "plato");
    }

    #[test]
    fn test_dna_growth_report() {
        let mut dna = SelfAssemblingDNA::new();
        let report = dna.growth_report();
        assert!(report.contains("plato"));
        dna.use_pathway(&["hardware", "motor"]);
        let report = dna.growth_report();
        assert!(report.contains("hardware"));
        assert!(report.contains("motor"));
    }

    #[test]
    fn test_dna_default() {
        let dna = SelfAssemblingDNA::default();
        assert_eq!(dna.root.pathway, "plato");
    }

    // --- Serde round-trip ---
    #[test]
    fn test_serde_action_request() {
        let mut req = ActionRequest::new("hermes", "sensor_read");
        req.with_param("key", "val");
        let json = serde_json::to_string(&req).unwrap();
        let de: ActionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(de.agent_id, "hermes");
        assert_eq!(de.params.get("key").unwrap(), "val");
    }

    #[test]
    fn test_serde_engine() {
        let mut engine = AffordanceEngine::new(100.0);
        engine.reinforce_pathway("test");
        let req = ActionRequest::new("hermes", "sensor_read");
        engine.execute(&req);
        let json = serde_json::to_string(&engine).unwrap();
        let de: AffordanceEngine = serde_json::from_str(&json).unwrap();
        assert_eq!(de.budget, 100.0);
        assert!((de.used - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_serde_dna() {
        let mut dna = SelfAssemblingDNA::new();
        dna.use_pathway(&["a", "b"]);
        let json = serde_json::to_string(&dna).unwrap();
        let de: SelfAssemblingDNA = serde_json::from_str(&json).unwrap();
        assert_eq!(de.root.children[0].pathway, "a");
    }

    #[test]
    fn test_serde_execution_result() {
        let result = ExecutionResult {
            success: true,
            walls_hit: vec![],
            energy_consumed: 1.0,
            energy_remaining: 99.0,
            growth_earned: 2.0,
            learned: vec!["test".into()],
        };
        let json = serde_json::to_string(&result).unwrap();
        let de: ExecutionResult = serde_json::from_str(&json).unwrap();
        assert!(de.success);
        assert_eq!(de.learned[0], "test");
    }

    #[test]
    fn test_serde_wall_type() {
        let wt = WallType::ConservationBudget;
        let json = serde_json::to_string(&wt).unwrap();
        let de: WallType = serde_json::from_str(&json).unwrap();
        assert_eq!(de, WallType::ConservationBudget);
    }

    // --- Integration tests ---
    #[test]
    fn test_full_workflow() {
        let mut engine = AffordanceEngine::new(100.0);

        // Sensor read — should succeed
        let req = ActionRequest::new("hermes", "sensor_read");
        let result = engine.execute(&req);
        assert!(result.success);
        assert!(!result.learned.is_empty());

        // Bridge send — should succeed
        let req = ActionRequest::new("hermes", "bridge_send");
        let result = engine.execute(&req);
        assert!(result.success);

        // Check conservation
        assert!(engine.is_conserved());

        // Engine summary
        let summary = engine.engine_summary();
        assert!(summary.conserved);
        assert!((summary.used - 1.0).abs() < f64::EPSILON);

        // Growth tracker should have XP
        assert!(engine.growth_tracker.xp > 0.0);
    }

    #[test]
    fn test_conservation_blocks_multiple() {
        let mut engine = AffordanceEngine::new(2.0);
        // sensor_read costs 0.5, can do 4 times
        for _ in 0..4 {
            let req = ActionRequest::new("hermes", "sensor_read");
            let result = engine.execute(&req);
            assert!(result.success);
        }
        // 5th time: over budget
        let req = ActionRequest::new("hermes", "sensor_read");
        let result = engine.execute(&req);
        assert!(!result.success);
    }

    #[test]
    fn test_growth_through_repeated_actions() {
        let mut engine = AffordanceEngine::new(1000.0);
        for _ in 0..5 {
            let req = ActionRequest::new("hermes", "sensor_read");
            engine.execute(&req);
        }
        assert!(engine.growth_tracker.level() >= 1);
        assert!(!engine.growth_tracker.learned_lessons.is_empty());
    }
}
