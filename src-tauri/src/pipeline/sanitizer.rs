use crate::rules::engine::apply_rules as apply_rule_engine;
use crate::rules::rule_types::RuleDefinition;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuleAction {
    pub rule_id: String,
    pub changed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SanitizeResult {
    pub output: String,
    pub changed: bool,
    pub actions: Vec<RuleAction>,
}

pub fn sanitize(input: &str, rules: &[RuleDefinition]) -> SanitizeResult {
    apply_rule_engine(input, rules)
}