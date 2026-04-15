use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    RegexReplace,
    Trim,
    LineFilter,
    UnindentOneIfAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub id: String,
    pub enabled: bool,
    pub order: u32,
    pub rule_type: RuleType,
    pub params: serde_json::Value,
}

#[derive(Debug)]
pub enum CompiledRule {
    RegexReplace {
        id: String,
        enabled: bool,
        order: u32,
        pattern: Regex,
        replace: String,
        multiline: bool,
    },
    Trim {
        id: String,
        enabled: bool,
        order: u32,
    },
    LineFilter {
        id: String,
        enabled: bool,
        order: u32,
        keep_empty: bool,
        trim_lines: bool,
    },
    UnindentOneIfAll {
        id: String,
        enabled: bool,
        order: u32,
    },
}

impl CompiledRule {
    pub fn id(&self) -> &str {
        match self {
            CompiledRule::RegexReplace { id, .. } => id,
            CompiledRule::Trim { id, .. } => id,
            CompiledRule::LineFilter { id, .. } => id,
            CompiledRule::UnindentOneIfAll { id, .. } => id,
        }
    }

    pub fn order(&self) -> u32 {
        match self {
            CompiledRule::RegexReplace { order, .. } => *order,
            CompiledRule::Trim { order, .. } => *order,
            CompiledRule::LineFilter { order, .. } => *order,
            CompiledRule::UnindentOneIfAll { order, .. } => *order,
        }
    }

    pub fn enabled(&self) -> bool {
        match self {
            CompiledRule::RegexReplace { enabled, .. } => *enabled,
            CompiledRule::Trim { enabled, .. } => *enabled,
            CompiledRule::LineFilter { enabled, .. } => *enabled,
            CompiledRule::UnindentOneIfAll { enabled, .. } => *enabled,
        }
    }
}

impl TryFrom<&RuleDefinition> for CompiledRule {
    type Error = anyhow::Error;

    fn try_from(def: &RuleDefinition) -> Result<Self, Self::Error> {
        match def.rule_type {
            RuleType::RegexReplace => {
                let pattern_str = def
                    .params
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("missing 'pattern' in regex_replace rule"))?;

                let replace = def
                    .params
                    .get("replace")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let multiline = def
                    .params
                    .get("multiline")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let mut builder = RegexBuilder::new(pattern_str);
                if multiline {
                    builder.multi_line(true);
                }

                let pattern = builder.build()?;

                Ok(CompiledRule::RegexReplace {
                    id: def.id.clone(),
                    enabled: def.enabled,
                    order: def.order,
                    pattern,
                    replace: replace.to_string(),
                    multiline,
                })
            }
            RuleType::Trim => Ok(CompiledRule::Trim {
                id: def.id.clone(),
                enabled: def.enabled,
                order: def.order,
            }),
            RuleType::LineFilter => {
                let keep_empty = def
                    .params
                    .get("keep_empty")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let trim_lines = def
                    .params
                    .get("trim_lines")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                Ok(CompiledRule::LineFilter {
                    id: def.id.clone(),
                    enabled: def.enabled,
                    order: def.order,
                    keep_empty,
                    trim_lines,
                })
            }
            RuleType::UnindentOneIfAll => Ok(CompiledRule::UnindentOneIfAll {
                id: def.id.clone(),
                enabled: def.enabled,
                order: def.order,
            }),
        }
    }
}