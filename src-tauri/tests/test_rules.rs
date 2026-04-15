use pi_clipper::pipeline::sanitizer::sanitize;
use pi_clipper::rules::builtins::default_rules;

#[test]
fn test_kubectl_json_command_preserves_indentation() {
    let input = r#"kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
{
"op": "add",
"path": "/rules/-",
"value": {
"apiGroups": [""],
"resources": ["secrets"],
"verbs": ["get", "list", "watch", "create", "update", "patch", "delete"]
}
}
]'"#;

    let rules = default_rules();
    let result = sanitize(input, &rules);

    // With normalize_whitespace disabled, indentation should be preserved
    let expected = input; // Should remain unchanged

    assert_eq!(result.output, expected);
    assert!(!result.changed, "Expected no changes with current rules");
}

#[test]
fn test_kubectl_json_command_with_proper_indentation() {
    let input = r#"kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
  {
    "op": "add",
    "path": "/rules/-",
    "value": {
      "apiGroups": [""],
      "resources": ["secrets"],
      "verbs": ["get", "list", "watch", "create", "update", "patch", "delete"]
    }
  }
]'"#;

    let rules = default_rules();
    let result = sanitize(input, &rules);

    println!("Input:\n{}", input);
    println!("Output:\n{}", result.output);
    println!("Changed: {}", result.changed);
    println!("Actions: {:?}", result.actions);

    // With normalize_whitespace disabled, proper indentation should be preserved
    let expected = input;

    assert_eq!(result.output, expected, "Output should preserve indentation");
    assert!(!result.changed, "Expected no changes with conservative rule set");
}

#[test]
fn test_strip_sh_prompt_works() {
    let input = r#"$ echo "hello"
hello"#;

    let rules = default_rules();
    let result = sanitize(input, &rules);

    println!("Input:\n{}", input);
    println!("Output:\n{}", result.output);

    assert!(!result.output.contains("$ "));
    assert!(result.output.contains("echo"));
}

#[test]
fn test_strip_powershell_prompt_works() {
    let input = r#"PS C:\> Get-Process
output"#;

    let rules = default_rules();
    let result = sanitize(input, &rules);

    println!("Input:\n{}", input);
    println!("Output:\n{}", result.output);

    assert!(!result.output.contains("PS"));
    assert!(result.output.contains("Get-Process"));
}

#[test]
fn test_trim_edges_removes_leading_trailing_whitespace() {
    let input = r#"
  some content

"#;

    let rules = default_rules();
    let result = sanitize(input, &rules);

    println!("Input: {:?}", input);
    println!("Output: {:?}", result.output);

    assert_eq!(result.output.trim(), result.output);
}

#[test]
fn test_strip_markdown_fence_with_language() {
    let input = r####"```bash
kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
  {
    "op": "add",
    "path": "/rules/-",
    "value": {
      "apiGroups": [""],
      "resources": ["secrets"],
      "verbs": ["get", "list", "watch", "create", "update", "patch", "delete"]
    }
  }
]'
```"####;

    let expected = r####"kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
  {
    "op": "add",
    "path": "/rules/-",
    "value": {
      "apiGroups": [""],
      "resources": ["secrets"],
      "verbs": ["get", "list", "watch", "create", "update", "patch", "delete"]
    }
  }
]'"####;

    let result = sanitize(input, &default_rules());

    assert_eq!(result.output, expected);
    assert!(result.changed);
    assert!(result
        .actions
        .iter()
        .any(|a| a.rule_id == "strip_markdown_code_fence_wrapper" && a.changed));
}

#[test]
fn test_strip_markdown_fence_without_language() {
    let input = r####"```
echo hello
```"####;

    let result = sanitize(input, &default_rules());

    assert_eq!(result.output, "echo hello");
    assert!(result.changed);
}

#[test]
fn test_does_not_strip_when_only_one_fence_marker_exists() {
    let input = r####"```rust
fn main() {}
"####;

    let result = sanitize(input, &default_rules());

    assert_eq!(result.output, "```rust\nfn main() {}");
    assert!(!result
        .actions
        .iter()
        .any(|a| a.rule_id == "strip_markdown_code_fence_wrapper" && a.changed));
}