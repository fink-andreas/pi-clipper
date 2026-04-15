use pi_clipper::pipeline::sanitizer::sanitize;
use pi_clipper::rules::builtins::default_rules;

#[test]
fn windows_terminal_style_kubectl_with_crlf_preserves_indentation() {
    let input = "kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[\r\n  {\r\n    \"op\": \"add\",\r\n    \"path\": \"/rules/-\",\r\n    \"value\": {\r\n      \"apiGroups\": [\"\"],\r\n      \"resources\": [\"secrets\"],\r\n      \"verbs\": [\"get\", \"list\", \"watch\", \"create\", \"update\", \"patch\", \"delete\"]\r\n    }\r\n  }\r\n]'";

    let result = sanitize(input, &default_rules());

    assert_eq!(
        result.output, input,
        "CRLF clipboard content should preserve indentation and formatting"
    );
    assert!(
        !result.changed,
        "No rule should mutate already-clean kubectl JSON payload"
    );
}

#[test]
fn strip_line_numbers_only_when_number_starts_line() {
    let input = "1 first\n2 second\n  {\n    \"op\": \"add\"\n  }";

    let result = sanitize(input, &default_rules());

    assert_eq!(result.output, "first\nsecond\n  {\n    \"op\": \"add\"\n  }");
    assert!(result.changed, "Line numbers at column 1 should be removed");
    assert!(result.actions.iter().any(|a| a.rule_id == "strip_line_numbers"));
}

#[test]
fn strip_line_numbers_does_not_consume_indented_json_lines() {
    let input = "payload='[\n  {\n    \"op\": \"add\",\n    \"path\": \"/rules/-\"\n  }\n]'";

    let result = sanitize(input, &default_rules());

    assert_eq!(
        result.output, input,
        "Indented non-numbered JSON lines must remain untouched"
    );
    assert!(
        !result.changed,
        "No mutation expected for normal indented JSON snippets"
    );
}

#[test]
fn uniform_single_leading_space_is_removed_from_all_lines() {
    let input = " kubectl patch clusterrole test --type='json' -p='[\n   {\n     \"op\": \"add\"\n   }\n ]'";

    let result = sanitize(input, &default_rules());

    let expected = "kubectl patch clusterrole test --type='json' -p='[\n  {\n    \"op\": \"add\"\n  }\n]'";

    assert_eq!(result.output, expected);
    assert!(result.changed);
    assert!(result.actions.iter().any(|a| a.rule_id == "unindent_one_if_all"));
}
