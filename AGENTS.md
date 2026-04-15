# AGENTS.md — Project Execution Plan and Ownership

This file defines practical agent roles and delivery workflow for building the Tauri clipboard cleaner.

## 1) Agent Roles

### 1. Product Agent
**Owns:** requirements, scope control, acceptance criteria
- Maintains `PRD.md`
- Tracks non-goals and MVP boundaries
- Validates feature requests against product goals

### 2. Architecture Agent
**Owns:** system design and module boundaries
- Maintains `ARCHITECTURE.md`
- Defines component APIs and data contracts
- Reviews cross-platform strategy decisions

### 3. Rust Core Agent
**Owns:** core pipeline implementation
- Implements watcher, dedupe, context, sanitizer, writer, logger
- Ensures deterministic rule execution
- Adds tests for rule logic and pipeline behavior

### 4. Platform Adapter Agent
**Owns:** OS-specific active-window detection
- Implements Windows/macOS/Linux adapters
- Normalizes metadata into common `ContextDecision`
- Documents permissions/limitations per OS

### 5. Rules & Heuristics Agent
**Owns:** cleanup quality and rule configuration
- Curates default rule sets and signatures
- Tunes false-positive/false-negative behavior
- Maintains test fixtures for sanitization

### 6. Observability Agent
**Owns:** logging, diagnostics, retention
- Defines JSONL schema
- Implements redaction/truncation defaults
- Adds troubleshooting docs and log rotation policies

---

## A) Rules & Heuristics Agent — Rule Configuration Guide

### Where Rules Are Defined

All rules are defined in **`src-tauri/src/rules/builtins.rs`** in the `default_rules()` function.

### Rule Structure

Each rule is a `RuleDefinition` with these fields:
- `id`: Unique identifier
- `enabled`: Whether the rule is active (`true`/`false`)
- `order`: Execution order (lower numbers execute first)
- `rule_type`: Type of rule (`RegexReplace`, `Trim`, `LineFilter`, `UnindentOneIfAll`)
- `params`: Rule-specific configuration (JSON)

### Rule Types

#### `RegexReplace`
Pattern-based text replacement.

```rust
RuleDefinition {
    id: "example".to_string(),
    enabled: true,
    order: 50,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"your_regex_here",
        "replace": "replacement_text",
        "multiline": true
    })
}
```

#### `Trim`
Removes leading/trailing whitespace from entire text.
```rust
RuleDefinition {
    id: "trim".to_string(),
    enabled: true,
    order: 90,
    rule_type: RuleType::Trim,
    params: serde_json::json!({})
}
```

#### `LineFilter`
Filters individual lines.
```rust
RuleDefinition {
    id: "filter".to_string(),
    enabled: true,
    order: 30,
    rule_type: RuleType::LineFilter,
    params: serde_json::json!({
        "keep_empty": false,
        "trim_lines": true
    })
}
```

#### `UnindentOneIfAll`
If all non-empty lines start with one leading space, remove one leading space from each non-empty line.
```rust
RuleDefinition {
    id: "unindent_one_if_all".to_string(),
    enabled: true,
    order: 80,
    rule_type: RuleType::UnindentOneIfAll,
    params: serde_json::json!({})
}
```

### Modifying Existing Rules

#### Case Study: Fixing `normalize_whitespace` for Code/JSON Preservation

**Problem:** The default `normalize_whitespace` rule collapses ALL spaces/tabs into single spaces, destroying code indentation and JSON/YAML structure.

**Legacy (problematic) rule:**
```rust
RuleDefinition {
    id: "normalize_whitespace".to_string(),
    enabled: true,
    order: 40,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"[ \t]+",
        "replace": " ",
        "multiline": false
    })
}
```

**Effect:**
```yaml
# Before
kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
  {
    "op": "add",
    ...

# After (BAD - indentation destroyed)
kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
{
"op": "add",
...
```

**Solution A: Disable completely**
```rust
RuleDefinition {
    id: "normalize_whitespace".to_string(),
    enabled: false,  // ← Disable
    // ...
}
```

**Solution B: Make selective**
Only collapse 2+ consecutive spaces (preserve tabs):
```rust
RuleDefinition {
    id: "normalize_whitespace".to_string(),
    enabled: true,
    order: 40,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"  +",        // 2+ spaces
        "replace": " ",
        "multiline": false
    })
}
```

**Solution C: Replace with line ending normalization only**
```rust
RuleDefinition {
    id: "normalize_line_endings".to_string(),
    enabled: true,
    order: 40,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"\r\n?|\n",
        "replace": "\n",
        "multiline": true
    })
}
```

### Adding New Rules

```rust
pub fn default_rules() -> Vec<RuleDefinition> {
    vec![
        // ... existing rules ...
        RuleDefinition {
            id: "remove_github_prefix".to_string(),
            enabled: true,
            order: 25,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                "pattern": r"^> ",
                "replace": "",
                "multiline": true
            })
        }
    ]
}
```

### Recommended Safe Rule Set

Use the current defaults in `src-tauri/src/rules/builtins.rs` as baseline:
- `strip_ansi` (order 10)
- `strip_sh_prompt` with strict marker `^[\$%>#]\s+` (order 20)
- `strip_powershell_prompt` (order 21)
- `strip_line_numbers` with strict marker `^\d+\s+` (order 30)
- `normalize_whitespace` disabled by default (order 40)
- `unindent_one_if_all` (order 80)
- `trim_blank_edges` (order 90)

### Rule Ordering Guidelines

| Order | Phase | Examples |
|-------|-------|----------|
| 1-20  | Content preservation | `strip_ansi` |
| 20-30 | Prompt removal | `strip_sh_prompt`, `strip_powershell_prompt` |
| 30-50 | Structured cleanup | `strip_line_numbers` |
| 80-90 | Final polish | `unindent_one_if_all`, `trim_blank_edges` |

Lower `order` values execute first.

### Testing Rule Changes (required)

1. Edit or add rule definitions in `src-tauri/src/rules/builtins.rs`
2. If needed, add rule type in `src-tauri/src/rules/rule_types.rs`
3. If needed, implement rule behavior in `src-tauri/src/rules/engine.rs`
4. Add/adjust tests in:
   - `src-tauri/tests/test_clipboard_integration.rs`
   - `src-tauri/tests/test_rules.rs`
5. Run regression checks:
   ```bash
   npm run verify:clip
   ```
   or
   ```bash
   .\verify-clip.bat
   ```
6. Launch verified build:
   ```bash
   .\run-verified.bat
   ```
7. Manual sanity check + logs (`events.jsonl`)

---
**Owns:** logging, diagnostics, retention
- Defines JSONL schema
- Implements redaction/truncation defaults
- Adds troubleshooting docs and log rotation policies

### 7. QA Agent
**Owns:** end-to-end reliability and release gates
- Runs test matrix across Win/Linux/macOS
- Verifies tray behavior and clipboard correctness
- Executes regression fixtures before release

---

## 2) Delivery Phases

### Phase 0 — Foundation
- Initialize Tauri app with tray-first behavior.
- Add config/bootstrap and app state skeleton.
- Create basic logging infra.

### Phase 1 — Core MVP Pipeline
- Clipboard watcher + dedupe/loop guard.
- Terminal context detection adapter abstraction.
- Sanitizer with minimal built-in rules.
- Clipboard rewrite + event logging.

### Phase 2 — Rule Management
- External YAML rules/signature files.
- Runtime reload from tray menu.
- Validation and fallback defaults.

### Phase 3 — Hardening
- Cross-platform QA pass.
- Improve detection confidence scoring.
- Performance tuning and error handling polish.

---

## 3) Definition of Done (MVP)
A change is considered done when:
1. It maps to an item in `PRD.md`.
2. Architecture impact is reflected in `ARCHITECTURE.md` if needed.
3. Includes tests or explicit test notes.
4. Includes logging/diagnostics for failure modes.
5. Works on at least one target OS and does not regress others.

---

## 4) Coding & Collaboration Rules
- Keep platform-specific code isolated in `context/{windows,macos,linux}.rs`.
- Keep rule pipeline pure and deterministic where possible.
- Prefer config-driven behavior over hardcoded terminal signatures.
- Log decisions and rule actions in structured format.
- Avoid storing full clipboard content unless explicitly enabled.

---

## 5) Prioritized Backlog (Initial)
1. Tray app bootstrap (no main window requirement).
2. Clipboard watcher with dedupe.
3. Windows terminal detection first (highest confidence path).
4. Rule engine with 3–5 essential rules.
5. Clipboard rewrite guard.
6. JSONL event logs + open-log-folder tray action.
7. Linux/macOS adapters.
8. Rule reload and signature config.

---

## 6) Handoff Artifacts
Each significant implementation PR should include:
- What requirement(s) it satisfies (reference PRD section)
- Any architecture changes (reference ARCHITECTURE section)
- Test evidence and known limitations
- Follow-up tasks if platform gaps remain
