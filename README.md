# pi-clipper

Tray-first Tauri app that monitors clipboard changes, detects terminal-origin copies, sanitizes snippets, logs actions, and rewrites cleaned text back to clipboard.

## Phase 1 MVP — Implemented

### Core Components

#### Clipboard Monitoring & Deduplication
- `src-tauri/src/pipeline/observer.rs`: Polls clipboard every 200ms
- `src-tauri/src/pipeline/watcher.rs`: Event struct with hash comparison
- Dedupe guard prevents reprocessing same content within window

#### Terminal Context Detection (Windows)
- `src-tauri/src/context/windows.rs`: Win32 API calls
- Detects foreground window process name
- Matches against: `WindowsTerminal.exe`, `powershell.exe`, `pwsh.exe`, `cmd.exe`, `wezterm-gui.exe`, `alacritty.exe`, `conhost.exe`
- Confidence scoring (0.85 for terminal, 0.0 otherwise)
- macOS/Linux adapters in context/ with stubs (future work)

#### Rule Engine & Built-in Rules
- `src-tauri/src/rules/engine.rs`: Deterministic, ordered rule execution
- `src-tauri/src/rules/rule_types.rs`: Built-in rule types (`RegexReplace`, `Trim`, `LineFilter`, `UnindentOneIfAll`)
- `src-tauri/src/rules/builtins.rs`: Default rules:
  - `strip_ansi`: Remove ANSI escape sequences
  - `strip_sh_prompt`: Remove shell prompts (`$`, `%`, `>`, `#`)
  - `strip_powershell_prompt`: Remove PowerShell prompts
  - `strip_line_numbers`: Remove leading line numbers (`^\d+\s+`)
  - `normalize_whitespace`: Disabled by default (too destructive for code/JSON)
  - `unindent_one_if_all`: If all non-empty lines share one leading space, remove one from all
  - `trim_blank_edges`: Trim leading/trailing whitespace from full text
- External YAML rule config supported (`config/default-rules.yaml`)

#### Clipboard Rewrite & Loop Guard
- `src-tauri/src/pipeline/writer.rs`: Writes cleaned text to clipboard
- Self-write fingerprint tracking (hash + 500ms window)
- Prevents infinite rewrite loops

#### Event Logging
- `src-tauri/src/pipeline/logger.rs`: JSONstructured logs
- Log schema includes:
  - timestamp, event_id, status, error
  - context (is_terminal, confidence, process_name, window_title)
  - input/output hashes and previews (truncated to 200 chars)
  - applied actions list, duration_ms
- 7-day log retention with auto-rotation
- "Open logs folder" tray action

#### Tray App
- System tray with menu:
  - Enable/Disable monitoring
  - Reload rules (stub)
  - Open logs folder
  - Quit
- No main window (tray-only)

### Architecture Alignment

All components align with `ARCHITECTURE.md`:
- Pipeline flow: observer → context detection → sanitizer → writer → logger
- Config management (`src-tauri/src/config/` for future YAML loading)
- Agent responsibilities per `AGENTS.md` implemented

## Customizing Rules

### Where Rules Are Defined

Rules are defined in **`src-tauri/src/rules/builtins.rs`** in the `default_rules()` function. Each rule is a `RuleDefinition` that specifies:
- `id`: Unique identifier
- `enabled`: Whether the rule is active
- `order`: Execution order (lower numbers run first)
- `rule_type`: Type of rule (`RegexReplace`, `Trim`, `LineFilter`, `UnindentOneIfAll`)
- `params`: Rule-specific configuration

### Rule Types

#### 1. RegexReplace
Replaces text matching a regex pattern.

```rust
RuleDefinition {
    id: "my_rule".to_string(),
    enabled: true,
    order: 50,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"your_regex_here",
        "replace": "replacement_text",
        "multiline": true  // true for multi-line patterns
    })
}
```

#### 2. Trim
Removes leading and trailing whitespace from the entire text.

```rust
RuleDefinition {
    id: "trim_edges".to_string(),
    enabled: true,
    order: 90,
    rule_type: RuleType::Trim,
    params: serde_json::json!({})
}
```

#### 3. LineFilter
Filters individual lines.

```rust
RuleDefinition {
    id: "filter_empty".to_string(),
    enabled: true,
    order: 30,
    rule_type: RuleType::LineFilter,
    params: serde_json::json!({
        "keep_empty": false,   // Remove empty lines
        "trim_lines": true     // Trim whitespace from each line
    })
}
```

#### 4. UnindentOneIfAll
If all non-empty lines start with exactly one leading space, removes one leading space from each non-empty line. Useful for terminal copies where every line gets one extra margin space.

```rust
RuleDefinition {
    id: "unindent_one_if_all".to_string(),
    enabled: true,
    order: 80,
    rule_type: RuleType::UnindentOneIfAll,
    params: serde_json::json!({})
}
```

### Disabling a Rule

To disable a rule, simply set `enabled: false`:

```rust
RuleDefinition {
    id: "normalize_whitespace".to_string(),
    enabled: false,  // ← Set to false to disable
    order: 40,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({...})
}
```

### Modifying a Rule

You can modify the `params` section to adjust rule behavior.

#### Example: Fixing the `normalize_whitespace` Issue

The default `normalize_whitespace` rule is too aggressive—it collapses all spaces/tabs into single spaces, destroying code indentation. Here's how to fix it:

**Problem:**
```rust
// LEGACY EXAMPLE - Collapses all spaces/tabs
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

This turns:
```yaml
kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
  {
    "op": "add",
    "path": "/rules/-",
    ...
```

Into:
```yaml
kubectl patch clusterrole <clientname>-cluster-admin-role --type='json' -p='[
{
"op": "add",
"path": "/rules/-",
...
```

**Solution 1: Disable the Rule Completely**
```rust
RuleDefinition {
    id: "normalize_whitespace".to_string(),
    enabled: false,  // ← Disable this rule
    order: 40,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"[ \t]+",
        "replace": " ",
        "multiline": false
    })
}
```

**Solution 2: Make It More Selective**
Only collapse multiple spaces in a row (but preserve tabs for indentation):

```rust
RuleDefinition {
    id: "normalize_whitespace".to_string(),
    enabled: true,
    order: 40,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"  +",        // Match 2+ consecutive spaces
        "replace": " ",            // Replace with single space
        "multiline": false
    })
}
```

**Solution 3: Normalize Only Line Endings**
```rust
RuleDefinition {
    id: "normalize_line_endings".to_string(),
    enabled: true,
    order: 40,
    rule_type: RuleType::RegexReplace,
    params: serde_json::json!({
        "pattern": r"\r\n?|\n",   // Match any line ending
        "replace": "\n",           // Normalize to \n
        "multiline": true
    })
}
```

### Adding Custom Rules

Add new rules to the `default_rules()` function:

```rust
pub fn default_rules() -> Vec<RuleDefinition> {
    vec![
        // ... existing rules ...
        RuleDefinition {
            id: "my_custom_rule".to_string(),
            enabled: true,
            order: 100,
            rule_type: RuleType::RegexReplace,
            params: serde_json::json!({
                "pattern": r"your-pattern",
                "replace": "replacement"
            })
        }
    ]
}
```

### Rule Order Matters

Rules execute in order based on their `order` value. Lower numbers run first.

**Recommended order:**
1. Remove ANSI codes (preserve visual text)
2. Strip prompts (remove command prefixes)
3. Remove line numbers
4. Optional normalization (disabled by default for code safety)
5. Shared-margin unindent pass (`UnindentOneIfAll`)
6. Trim edges (final polish)

**Example:**
```rust
rule_type: RegexReplace,     order: 10, // ANSI codes
rule_type: RegexReplace,     order: 20, // Shell prompts
rule_type: RegexReplace,     order: 30, // Line numbers
rule_type: RegexReplace,     order: 40, // Optional whitespace normalization (usually disabled)
rule_type: UnindentOneIfAll, order: 80, // Remove one shared leading space
rule_type: Trim,             order: 90, // Final trim
```

### Testing Your Changes

1. Modify `src-tauri/src/rules/builtins.rs`
2. Run clipboard regression tests:
   ```bash
   .\verify-clip.bat
   ```
   or
   ```bash
   npm run verify:clip
   ```
3. Run the app:
   ```bash
   .\run.bat
   ```
   `run.bat` always kills any existing `pi-clipper.exe`, builds a fresh release binary, and launches it. This prevents stale-runtime confusion.

   Optional one-shot command (tests + launch):
   ```bash
   .\run-verified.bat
   ```
4. Copy text from a terminal
5. Check the result:
   - Paste to see cleaned text
   - Right-click tray → "Open logs folder" → view `events.jsonl`

### Current Safe Defaults (Code/JSON-friendly)

Current built-in defaults in `src-tauri/src/rules/builtins.rs` are:
- `strip_ansi` (enabled)
- `strip_sh_prompt` with strict prompt marker `^[\$%>#]\s+` (enabled)
- `strip_powershell_prompt` (enabled)
- `strip_line_numbers` with strict start-of-line digits `^\d+\s+` (enabled)
- `normalize_whitespace` (disabled)
- `unindent_one_if_all` (enabled)
- `trim_blank_edges` (enabled)

### Rule + Test Workflow (required)

When adding/changing a rule:
1. Update rule type if needed in `src-tauri/src/rules/rule_types.rs`
2. Implement logic in `src-tauri/src/rules/engine.rs`
3. Register/configure in `src-tauri/src/rules/builtins.rs`
4. Add or update integration coverage in `src-tauri/tests/test_clipboard_integration.rs`
5. Add or update unit coverage in `src-tauri/tests/test_rules.rs`
6. Run regression gate:
   - `npm run verify:clip` or `.\verify-clip.bat`
7. Launch verified build:
   - `.\run-verified.bat`

## Scaffolded docs
- `PRD.md` — product requirements
- `ARCHITECTURE.md` — system design
- `AGENTS.md` — roles and plan

## Development

```bash
# Development mode
npm run dev

# Build for release
npm run build
```

## Next Steps (Phase 2)

1. Implement YAML rule loading and reload from tray
2. Add macOS/Linux context detection adapters
3. Improve confidence scoring and terminal signature configuration
4. Add golden-file tests for sanitizer pipelines
5. Performance benchmarking and tuning

## Testing

Run unit tests:
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## Privacy

- All processing is local-only
- Full clipboard content is not stored by default (only truncated previews)
- No outbound network calls