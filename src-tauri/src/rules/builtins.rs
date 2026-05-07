use crate::rules::rule_types::{RuleDefinition, RuleType};

pub fn default_rules() -> Vec<RuleDefinition> {
    vec![
        RuleDefinition {
            id: "strip_ansi".to_string(),
            enabled: true,
            order: 10,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                "pattern": "\\x1B\\[[0-9;]*[A-Za-z]",
                "replace": "",
                "multiline": true
            }),
        },
        RuleDefinition {
            id: "strip_sh_prompt".to_string(),
            enabled: true,
            order: 20,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                // Match shell prompts: "$ ", ">", "# ", or SSH-style "user@host:~$ "
                "pattern": "^(?:\\$|%|#|>)\\s+|\\w+@[\\w.-]+:[^\\s$#]*[#$]\\s*",
                "replace": "",
                "multiline": true
            }),
        },
        RuleDefinition {
            id: "strip_powershell_prompt".to_string(),
            enabled: true,
            order: 21,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                "pattern": r"^PS [^>]+>\s+",
                "replace": "",
                "multiline": true
            }),
        },
        RuleDefinition {
            id: "strip_line_numbers".to_string(),
            enabled: true,
            order: 30,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                // Only match lines that are purely line numbers like "1. ", "(1) ", "1)"
                // Does NOT match command output like "1024" on its own line
                "pattern": r"^\d+[\.:) ][\t ]+",
                "replace": "",
                "multiline": true
            }),
        },
        RuleDefinition {
            id: "normalize_whitespace".to_string(),
            enabled: false,
            order: 40,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                "pattern": r"[ \t]+",
                "replace": " ",
                "multiline": false
            }),
        },
        RuleDefinition {
            id: "strip_markdown_code_fence_wrapper".to_string(),
            enabled: true,
            order: 70,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                "pattern": r"(?s)^```[ \t]*([a-zA-Z0-9_+.-]+)?[ \t]*\r?\n(.*?)\r?\n```[ \t]*$",
                "replace": "$2",
                "multiline": false
            }),
        },
        RuleDefinition {
            id: "unindent_one_if_all".to_string(),
            enabled: true,
            order: 80,
            rule_type: RuleType::UnindentOneIfAll,
            params: serde_json::json!({}),
        },
        RuleDefinition {
            id: "trim_blank_edges".to_string(),
            enabled: true,
            order: 90,
            rule_type: RuleType::Trim,
            params: serde_json::json!({}),
        },
    ]
}
