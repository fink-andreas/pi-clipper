# Rules Configuration Quick Reference

## File Location

**`src-tauri/src/rules/builtins.rs`** → `default_rules()` function

## Quick Actions

| Task | How |
|------|-----|
| Disable a rule | Set `enabled: false` |
| Change order | Adjust `order` number |
| Modify pattern | Edit `params.pattern` |
| Change replacement | Edit `params.replace` |

## Rule Types

### 1. RegexReplace

```rust
RuleDefinition {
    id: "rule_id".to_string(),
    enabled: true,
    order: 50,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"regex_pattern",
        "replace": "replacement",
        "multiline": true
    })
}
```

### 2. Trim

```rust
RuleDefinition {
    id: "rule_id".to_string(),
    enabled: true,
    order: 90,
    rule_type: RuleType::Trim,
    params: serde_json::json!({})
}
```

### 3. LineFilter

```rust
RuleDefinition {
    id: "rule_id".to_string(),
    enabled: true,
    order: 30,
    rule_type: RuleType::LineFilter,
    params: serde_json::json!({
        "keep_empty": false,
        "trim_lines": true
    })
}
```

## Current Built-in Rules

| ID | Order | Type | Description |
|----|-------|------|-------------|
| `strip_ansi` | 10 | RegexReplace | Remove ANSI escape sequences |
| `strip_sh_prompt` | 20 | RegexReplace | Remove shell prompts (`$`, `%`, `>`) |
| `strip_powershell_prompt` | 21 | RegexReplace | Remove PowerShell prompts |
| `strip_line_numbers` | 30 | RegexReplace | Remove leading line numbers |
| `normalize_whitespace` | 40 | RegexReplace | Collapse spaces/tabs (⚠️ Destructive to code) |
| `trim_edges` | 90 | Trim | Remove leading/trailing whitespace |

## Common Patterns

### Remove prefix from each line
```rust
params: serde_json::json!({
    "pattern": r"^> ",
    "replace": "",
    "multiline": true
})
```

### Remove email addresses
```rust
params: serde_json::json!({
    "pattern": r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
    "replace": "[REDACTED]",
    "multiline": false
})
```

### Normalize line endings
```rust
params: serde_json::json!({
    "pattern": r"\r\n?|\n",
    "replace": "\n",
    "multiline": true
})
```

### Remove timestamps
```rust
params: serde_json::json!({
    "pattern": r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} ",
    "replace": "",
    "multiline": true
})
```

## Rule Order Cheat Sheet

```
10 - ANSI/control chars  → Preserve clean text
20 - Shell prompts      → Remove command prefixes
30 - Line numbers       → Strip numbering
40 - Spacing            → Normalize (⚠️ Careful with code)
90 - Trim edges         → Final cleanup
```

## The `normalize_whitespace` Issue

### Problem
This rule destroys code indentation:

```rust
pattern: r"[ \t]+"
replace: " "
```

**Before:**
```yaml
kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
  {
    "op": "add",
    ...
```

**After (BAD):**
```yaml
kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
{
"op": "add",
...
```

### Fix: Disable or change the pattern

**Option A: Disable**
```rust
enabled: false  // Recommended for code workflows
```

**Option B: Make selective**
```rust
pattern: r"  +"  // Only collapse 2+ consecutive spaces
```

## Recommended Safe Configuration

For code, JSON, YAML heavy workflows:

```rust
pub fn default_rules() -> Vec<RuleDefinition> {
    vec![
        // Strip ANSI codes
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
        // Strip shell prompts
        RuleDefinition {
            id: "strip_sh_prompt".to_string(),
            enabled: true,
            order: 20,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                "pattern": r"^[\$%>#]?\s+",
                "replace": "",
                "multiline": true
            }),
        },
        // Strip PowerShell prompts
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
        // Strip line numbers
        RuleDefinition {
            id: "strip_line_numbers".to_string(),
            enabled: true,
            order: 30,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                "pattern": r"^\s*\d+\s+",
                "replace": "",
                "multiline": true
            }),
        },
        // Normalize line endings (not whitespace)
        RuleDefinition {
            id: "normalize_line_endings".to_string(),
            enabled: true,
            order: 40,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                "pattern": r"\r\n?|\n",
                "replace": "\n",
                "multiline": true
            }),
        },
        // Trim edges
        RuleDefinition {
            id: "trim_edges".to_string(),
            enabled: true,
            order: 90,
            rule_type: RuleType::Trim,
            params: serde_json::json!({}),
        },
    ]
}
```

**Changes from default:**
- ❌ Removed `normalize_whitespace` (destructive)
- ✅ Added `normalize_line_endings` (safe)

## Testing Changes

```bash
# 1. Edit src-tauri/src/rules/builtins.rs
# 2. Rebuild
npm run build

# 3. Run
.\run.bat

# 4. Test clipboard copy from terminal
# 5. Check logs: Right-click tray → Open logs folder → events.jsonl
```

## Regex Syntax

| Pattern | Matches | Example |
|---------|---------|---------|
| `^` | Start of line | `^PS>` matches at line start |
| `$` | End of line | `prompt$` matches at line end |
| `\s+` | One or more whitespace | Collapses spaces/tabs |
| `\d+` | One or more digits | Matches line numbers |
| `.*` | Any characters | `.*password.*` matches "password" anywhere |
| `[abc]` | Any of a, b, or c | `[cmd]` matches "c", "m", or "d" |
| `[^abc]` | Any except a, b, c | `[^>]` matches anything but ">" |
| `|` | OR | `a|b` matches "a" or "b" |

Multiline mode (`multiline: true`) affects `^` and `$`:
- True: Start/end of each line
- False: Start/end of entire string (default)