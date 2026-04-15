# ARCHITECTURE — Clipboard Terminal Snippet Cleaner

## 1) High-Level Architecture

```
OS Clipboard Events
        │
        ▼
 ClipboardWatcher ──► Event Deduper ──► ContextDetector
                                         │
                                         ▼
                                   RuleEngine/Sanitizer
                                         │
                         ┌───────────────┴───────────────┐
                         ▼                               ▼
                  ClipboardWriter                    EventLogger
                         │                               │
                         ▼                               ▼
                 System Clipboard                 JSONL logs + rotation
```

The app is a Tauri desktop process with no mandatory main window. It lives primarily in the system tray and runs a Rust core pipeline.

---

## 2) Runtime Components

### 2.1 Tauri Shell (Rust)
- Initializes app state, tray menu, and background tasks.
- Hosts lifecycle actions:
  - enable/disable monitoring
  - reload config/rules
  - open log/config paths
  - quit

### 2.2 ClipboardWatcher
- Subscribes/polls clipboard text changes using a cross-platform crate.
- Emits `ClipboardChanged` events with text + timestamp.
- Ignores non-text payloads in v1.

### 2.3 Event Deduper / Loop Guard
- Maintains recent hashes and self-write timestamps.
- Drops duplicate or self-induced events.

### 2.4 ContextDetector (OS adapters)
- Captures foreground window metadata at event time.
- Uses OS-specific adapters:
  - Windows adapter
  - macOS adapter
  - Linux adapter (X11/Wayland best-effort)
- Produces `ContextDecision { is_terminal, confidence, matched_signature }`.

### 2.5 RuleEngine / Sanitizer
- Applies ordered rule pipeline to input text when `is_terminal=true`.
- Each rule returns:
  - transformed text
  - action metadata
  - changed/not-changed flag
- Supports dynamic config reload.

### 2.6 ClipboardWriter
- Writes cleaned text back to system clipboard only when changed.
- Registers write fingerprint to avoid immediate reprocessing loop.

### 2.7 EventLogger
- Appends structured JSONL records.
- Handles log rotation and retention.

### 2.8 ConfigManager
- Loads config/rule files from user config dir.
- Validates schema and applies defaults on error.

---

## 3) Suggested Project Structure

```
src-tauri/
  src/
    main.rs
    app_state.rs
    tray.rs
    pipeline/
      mod.rs
      watcher.rs
      dedupe.rs
      context.rs
      sanitizer.rs
      writer.rs
      logger.rs
    context/
      mod.rs
      windows.rs
      macos.rs
      linux.rs
    rules/
      mod.rs
      rule_types.rs
      engine.rs
      builtins.rs
    config/
      mod.rs
      schema.rs
      loader.rs
  tauri.conf.json

config/
  default-rules.yaml
  default-signatures.yaml
```

---

## 4) Data Contracts

### 4.1 Clipboard Event
```json
{
  "timestamp": "2026-03-20T11:30:00Z",
  "text": "raw clipboard content",
  "hash": "sha256..."
}
```

### 4.2 Context Decision
```json
{
  "is_terminal": true,
  "confidence": 0.92,
  "process_name": "WindowsTerminal.exe",
  "window_title": "Windows PowerShell",
  "matched_signature": "windows-terminal"
}
```

### 4.3 Rule Action Record
```json
{
  "rule_id": "strip_shell_prompt",
  "changed": true,
  "details": {
    "lines_affected": 4
  }
}
```

### 4.4 Log Record
```json
{
  "timestamp": "2026-03-20T11:30:00Z",
  "event_id": "uuid",
  "context": { "...": "..." },
  "input_hash": "...",
  "output_hash": "...",
  "changed": true,
  "actions": ["strip_ansi", "strip_shell_prompt"],
  "duration_ms": 12,
  "status": "ok"
}
```

---

## 5) Rule System Design

### Rule Definition (YAML)
- `id`: unique rule name
- `enabled`: boolean
- `order`: pipeline order
- `type`: known rule type (`regex_replace`, `line_filter`, `trim`, etc.)
- `params`: rule-specific options

Example:
```yaml
- id: strip_powershell_prompt
  enabled: true
  order: 20
  type: regex_replace
  params:
    pattern: "^(PS [^>]+>\s+)"
    replace: ""
    multiline: true
```

### Engine Behavior
1. Load ordered enabled rules.
2. Apply rules deterministically.
3. Collect action metadata.
4. Return final text + action list.

---

## 6) Cross-Platform Notes

### Windows
- Foreground window + process resolution via Win32 APIs.
- Strong terminal detection reliability expected.

### macOS
- Foreground app/window info may require accessibility permissions depending on API choice.
- Must handle permission-denied gracefully.

### Linux
- X11 and Wayland differ significantly for active window info.
- Implement adapter abstraction and fallback to weaker confidence when metadata is incomplete.

---

## 7) Security & Privacy
- Local-only processing/logging.
- No outbound network by default.
- Redact or truncate logged content by default; full content logging opt-in only.

---

## 8) Performance Strategy
- Hash-first dedupe to reduce unnecessary processing.
- Rule pipeline optimized for linear text scanning.
- Background worker thread/channel to keep tray UI responsive.

---

## 9) Testing Strategy
- Unit tests for rule engine and individual built-in rules.
- Contract tests for config parsing/validation.
- Adapter tests per OS (where feasible).
- Integration tests with simulated clipboard events.
- Golden-file tests for sanitizer input/output fixtures.

---

## 10) Deployment
- Build targets:
  - `x86_64-pc-windows-msvc`
  - `x86_64-unknown-linux-gnu` (and/or distro-specific packaging)
  - `aarch64-apple-darwin` + `x86_64-apple-darwin`
- Packaging via Tauri bundler.
