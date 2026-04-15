use crate::pipeline::sanitizer::{RuleAction, SanitizeResult};
use crate::rules::rule_types::{CompiledRule, RuleDefinition};

pub fn apply_rules(input: &str, rules: &[RuleDefinition]) -> SanitizeResult {
    let mut actions = Vec::new();
    let mut current = input.to_string();
    let mut changed = false;

    let compiled_rules: Vec<CompiledRule> = rules
        .iter()
        .filter_map(|def| CompiledRule::try_from(def).ok())
        .collect();

    let mut sorted_rules = compiled_rules;
    sorted_rules.sort_by_key(|r| r.order());

    for rule in &sorted_rules {
        if !rule.enabled() {
            continue;
        }

        match apply_single_rule(&current, rule) {
            Some(rule_result) if !rule_result.is_empty() && rule_result != current => {
                actions.push(RuleAction {
                    rule_id: rule.id().to_string(),
                    changed: true,
                });
                current = rule_result;
                changed = true;
            }
            Some(_) => {
                actions.push(RuleAction {
                    rule_id: rule.id().to_string(),
                    changed: false,
                });
            }
            None => {
                actions.push(RuleAction {
                    rule_id: rule.id().to_string(),
                    changed: false,
                });
            }
        }
    }

    SanitizeResult {
        output: current,
        changed,
        actions,
    }
}

fn apply_single_rule(input: &str, rule: &CompiledRule) -> Option<String> {
    match rule {
        CompiledRule::RegexReplace { pattern, replace, .. } => {
            let result = pattern.replace_all(input, replace).to_string();
            Some(result)
        }
        CompiledRule::Trim { .. } => Some(input.trim().to_string()),
        CompiledRule::LineFilter {
            keep_empty,
            trim_lines,
            ..
        } => {
            let lines: Vec<String> = input
                .lines()
                .map(|line| {
                    if *trim_lines {
                        line.trim().to_string()
                    } else {
                        line.to_string()
                    }
                })
                .filter(|line| *keep_empty || !line.is_empty())
                .collect();

            Some(lines.join("\n"))
        }
        CompiledRule::UnindentOneIfAll { .. } => Some(unindent_one_space_if_all_lines_have_it(input)),
    }
}

fn unindent_one_space_if_all_lines_have_it(input: &str) -> String {
    let has_crlf = input.contains("\r\n");
    let sep = if has_crlf { "\r\n" } else { "\n" };
    let mut lines: Vec<&str> = if has_crlf {
        input.split("\r\n").collect()
    } else {
        input.split('\n').collect()
    };

    let non_empty_lines: Vec<&str> = lines.iter().copied().filter(|l| !l.is_empty()).collect();

    if non_empty_lines.is_empty() || !non_empty_lines.iter().all(|line| line.starts_with(' ')) {
        return input.to_string();
    }

    for line in &mut lines {
        if !line.is_empty() && line.starts_with(' ') {
            *line = &line[1..];
        }
    }

    lines.join(sep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_replace_basic() {
        let rule = CompiledRule::RegexReplace {
            id: "test".to_string(),
            enabled: true,
            order: 1,
            pattern: regex::Regex::new(r"\s+").unwrap(),
            replace: " ".to_string(),
        };

        let input = "hello   world";
        let result = apply_single_rule(input, &rule);
        assert_eq!(result, Some("hello world".to_string()));
    }

    #[test]
    fn test_trim_rule() {
        let rule = CompiledRule::Trim {
            id: "trim".to_string(),
            enabled: true,
            order: 1,
        };

        let input = "   hello world   ";
        let result = apply_single_rule(input, &rule);
        assert_eq!(result, Some("hello world".to_string()));
    }

    #[test]
    fn test_line_filter_no_empty() {
        let rule = CompiledRule::LineFilter {
            id: "filter".to_string(),
            enabled: true,
            order: 1,
            keep_empty: false,
            trim_lines: true,
        };

        let input = "  a  \n\n  b  \n\n  c  ";
        let result = apply_single_rule(input, &rule);
        assert_eq!(result, Some("a\nb\nc".to_string()));
    }

    #[test]
    fn test_ordered_rule_execution() {
        let rules = vec![
            RuleDefinition {
                id: "replace".to_string(),
                enabled: true,
                order: 10,
                rule_type: crate::rules::rule_types::RuleType::RegexReplace,
                params: serde_json::json!({
                    "pattern": r"\s+",
                    "replace": " "
                }),
            },
            RuleDefinition {
                id: "trim".to_string(),
                enabled: true,
                order: 20,
                rule_type: crate::rules::rule_types::RuleType::Trim,
                params: serde_json::json!({}),
            },
        ];

        let input = "  hello   world  ";
        let result = apply_rules(input, &rules);
        assert_eq!(result.output, "hello world");
        assert!(result.changed);
        assert_eq!(result.actions.len(), 2);
    }

    #[test]
    fn test_disabled_rules_skipped() {
        let rules = vec![
            RuleDefinition {
                id: "enabled".to_string(),
                enabled: true,
                order: 10,
                rule_type: crate::rules::rule_types::RuleType::RegexReplace,
                params: serde_json::json!({
                    "pattern": "hello",
                    "replace": "hi"
                }),
            },
            RuleDefinition {
                id: "disabled".to_string(),
                enabled: false,
                order: 20,
                rule_type: crate::rules::rule_types::RuleType::RegexReplace,
                params: serde_json::json!({
                    "pattern": "hi",
                    "replace": "bye"
                }),
            },
        ];

        let input = "hello world";
        let result = apply_rules(input, &rules);
        assert_eq!(result.output, "hi world");
    }
}