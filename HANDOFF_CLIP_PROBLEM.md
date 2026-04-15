# Handoff: Clipboard Indentation Loss Bug

## Summary

User reports that when copying from Windows Terminal, text indentation is being lost when processed by pi-clipper. Indentation is preserved when pasting directly to Notepad (without app running), but is lost when pi-clipper is active.

## Problem Statement

**What user expects:**
```bash
kubectl patch clusterrole test --type='json' -p='[
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
```

**What user gets:**
```bash
kubectl patch clusterrole test --type='json' -p='[
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
```

## Investigation History

### User Confirmed
- Source: Windows Terminal
- Direct paste to Notepad (without app): ✅ Indentation preserved
- With pi-clipper: ❌ Indentation lost
- App is running (tray icon visible)

### Previous Fixes Attempted

**1. Disabled `normalize_whitespace` rule:**
```rust
RuleDefinition {
    id: "normalize_whitespace".to_string(),
    enabled: false,  // Changed from true
    order: 40,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"[ \t]+",
        "replace": " ",
        "multiline": false
    }),
}
```

**2. Fixed `strip_sh_prompt` regex:**
```rust
// BEFORE (BUGGY)
pattern: r"^[\$%>#]?\s+"  // "?" makes prompt optional → matches ANY leading whitespace

// AFTER (FIXED)
pattern: r"^[\$%>#]\s+"   // Prompt required → only matches actual prompts
```

### Test Results

All 5 tests in `src-tauri/tests/test_rules.rs` pass:
- ✅ test_kubectl_json_command_preserves_indentation
- ✅ test_kubectl_json_command_with_proper_indentation
- ✅ test_strip_sh_prompt_works
- ✅ test_strip_powershell_prompt_works
- ✅ test_trim_edges_removes_leading_trailing_whitespace

**But the actual app still loses indentation.**

### Log Analysis

Latest log entry (from copying kubectl command):
```json
{
  "timestamp": "2026-03-20T11:38:57.730676100Z",
  "event_id": "0151afec-d26e-4634-ae82-d4765172e98d",
  "context": {
    "is_terminal": true,
    "confidence": 0.85,
    "process_name": "WindowsTerminal.exe",
    "window_title": null
  },
  "input_hash": "d95b594f1160904588acce3654b8cd7e0323daed8f1497e5fc22a585786c1ce2",
  "output_hash": "d95b594f1160904588acce3654b8cd7e0323daed8f1497e5fc22a585786c1ce2",
  "input_preview": "```bash\r\nkubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[\r\n{\r\n\"op\": \"add\",",
  "output_preview": null,
  "changed": false,
  "actions": ["strip_ansi", "strip_sh_prompt", "strip_powershell_prompt", "strip_line_numbers", "trim_blank_edges"],
  "duration_ms": 1,
  "status": "ok",
  "error": null
}
```

**Key observations:**
- `input_hash` == `output_hash` → Content reportedly unchanged
- `changed: false` → Sanitizer reports no changes
- `input_preview` shows content **already without indentation** (`{\r\n` instead of `  {\r\n`)
- Actions ran but none caused changes

**BUT:** User reports indentation is lost when pasting.

## Hypothesis #1: `strip_line_numbers` Regex Bug

**Current rule definition:**
```rust
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
}
```

**Pattern breakdown:**
- `^` → Start of line
- `\s*` → Zero or more whitespace characters
- `\d+` → One or more digits
- `\s+` → One or more whitespace characters

**Potential issue:**
If a line has NO numbers but DOES have leading whitespace, this pattern might still match if the regex engine backtracks. While `\d+` should fail without digits, there could be edge cases.

**Test if this is the issue:**
```bash
# Disable the rule and rebuild
# In src-tauri/src/rules/builtins.rs, set:
enabled: false

# Rebuild and test
npm run build
.\run.bat
```

**Better regex:**
```rust
pattern: r"^\s*\d+\s+"  // If this is causing issues, try:
// pattern: r"^\d+\s+"   // WITHOUT preceding \s*
// OR
// pattern: r"^\d+\s+(?=\S)"  // Must be followed by non-whitespace
```

## Hypothesis #2: Race Condition or Timing Issue

When copying from Windows Terminal, there might be a race where:
1. Terminal clipboard gets content with proper indentation
2. Our app reads clipboard (gets proper content)
3. Terminal modifies clipboard (removes formatting/indentation)
4. Our app writes back (what we read earlier?)
5. User pastes (gets what we wrote)

**Evidence against:** The `input_preview` shows no indentation, suggesting the clipboard was already stripped when we read it.

## Hypothesis #3: Windows Terminal Clipboard Behavior

Windows Terminal might have unusual clipboard behavior:
- Could be writing to clipboard in multiple passes
- Could be using different clipboard formats (CF_UNICODETEXT vs CF_TEXT)
- Could have built-in stripping that happens after initial copy

**Test:** Try copying from a different terminal (cmd.exe, PowerShell directly without Windows Terminal) to see if behavior changes.

## Hypothesis #4: Clip loop or rewrite issue

Looking at `src-tauri/src/pipeline/writer.rs`:

- Writes happen when `sanitize()` returns changed content
- Has self-write detection (500ms window)
- Returns `Ok(true)` when successful

**Potential issue:** If the app writes back to clipboard even when content hasn't meaningfully changed (just hash difference), it could be stripping something.

## Files to Investigate

### Primary
- **`src-tauri/src/rules/builtins.rs`** - Rule definitions (especially `strip_line_numbers`)
- **`src-tauri/src/rules/engine.rs`** - Rule application logic
- **`src-tauri/src/pipeline/sanitizer.rs`** - Main sanitize function
- **`src-tauri/src/pipeline/writer.rs`** - Clipboard rewrite logic

### Secondary
- **`src-tauri/src/pipeline/observer.rs`** - Clipboard polling
- **`src-tauri/src/pipeline/watcher.rs`** - Deduplication logic
- **`src-tauri/src/pipeline/mod.rs`** - Pipeline orchestration

## Test Files Created

- **`test_input.txt`** - Clean test input with proper indentation
- **`src-tauri/tests/test_rules.rs`** - Unit tests for rule behavior

## Recommended Investigation Steps

### Step 1: Add Debug Logging

Add detailed logging to see exactly what each rule is doing:

```rust
// In sanitizer.rs, after each rule application
tracing::debug!(
    "Applied rule '{}': changed={}, input_len={}, output_len={}",
    rule.id,
    result.changed,
    input.len(),
    result.output.len()
);

// Log input/output with visible whitespace
tracing::debug!("Input: {:?}", input.replace(' ', '·').replace('\n', '⏎'));
tracing::debug!("Output: {:?}", result.output.replace(' ', '·').replace('\n', '⏎'));
```

### Step 2: Disable All Rules Temporarily

In `src-tauri/src/rules/builtins.rs`, set all `enabled: false` and rebuild:

```rust
pub fn default_rules() -> Vec<RuleDefinition> {
    vec![
        // All rules disabled
        RuleDefinition {
            id: "strip_ansi".to_string(),
            enabled: false,  // ← Disable
            // ...
        },
        // ... disable all
    ]
}
```

If indentation is preserved with all rules disabled, re-enable rules one by one to find the culprit.

### Step 3: Test with Known Good Input

Use `test_input.txt`:
1. Open file in Notepad
2. Select all (Ctrl+A)
3. Copy (Ctrl+C)
4. Paste to see if indentation preserved
5. Check logs: `%APPDATA%\pi-clipper\logs\events.jsonl`

### Step 4: Inspect Clipboard Content

Create a simple test to inspect actual clipboard bytes:

```rust
// In main loop, add debug output
let clipboard_content = observer.read_text()?;
tracing::info!(
    "Clipboard: bytes={}, first_line={:?}",
    clipboard_content.len(),
    clipboard_content.lines().next()
);
```

### Step 5: Check `strip_line_numbers` Behavior

Write a specific test for this rule:

```rust
#[test]
fn test_strip_line_numbers_no_nums() {
    let input = "  {\n    \"op\": \"add\"\n  }";
    let result = apply_rule("strip_line_numbers", input);

    assert_eq!(result, input, "Should not remove indentation when no line numbers");
}
```

## Current State

- ✅ app running and built
- ✅ tests passing (but don't reproduce user issue)
- ❌ actual user copy-paste loses indentation
- ❌ logs show `changed: false` but clipboard is modified

## User Environment

- OS: Windows (from logs showing WindowsTerminal.exe)
- Terminal: Windows Terminal
- App: pi-clipper v0.1.0 (Phase 1 MVP)
- Log location: `%APPDATA%\pi-clipper\logs\events.jsonl`

## Most Likely Root Cause (Ranked)

1. **`strip_line_numbers` regex bug** - Pattern `^\s*\d+\s+` might be matching lines with just whitespace
2. **Another rule issue** - Hidden behavior not caught by tests
3. **Pipeline logic bug** - Writes happening when they shouldn't
4. **Windows Terminal behavior** - Terminal modifying clipboard after copy
5. **arboard library quirk** - Cross-platform clipboard library behaving unexpectedly

## Next Action

**Start with Hypothesis #1:**

1. Disable `strip_line_numbers` rule:
   ```rust
   RuleDefinition {
       id: "strip_line_numbers".to_string(),
       enabled: false,  // ← Try this first
       // ...
   }
   ```

2. Rebuild:
   ```bash
   npm run build
   ```

3. Have user test copying from Windows Terminal

4. If fixed, focus on fixing the regex pattern

5. If not fixed, proceed to disable other rules systematically

## Contact

User report timestamp: 2026-03-20
Last log entry: 2026-03-20T11:38:57.730676100Z
Test file location: `test_input.txt`
Unit tests: `src-tauri/tests/test_rules.rs`