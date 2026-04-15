# PRD — Clipboard Terminal Snippet Cleaner (Tauri)

## 1) Product Overview
A cross-platform desktop tray app (Windows, Linux, macOS) that watches clipboard changes and, when a copy event occurs from a terminal window, cleans terminal noise from the text and writes back a clean code snippet.

Primary goal: make pasted terminal-derived code snippets immediately usable.

---

## 2) Problem Statement
When copying from terminal windows, clipboard contents often include prompts, line numbers, shell artifacts, wrapped output, ANSI/control characters, and accidental extra text. This reduces productivity and causes copy/paste errors.

---

## 3) Goals
1. Run silently as a tray app on Win/Linux/macOS.
2. Detect clipboard changes in near real-time.
3. Determine whether the copy happened while a terminal window was focused.
4. Apply configurable cleanup heuristics to produce a clean snippet.
5. Log original snippet, cleaned snippet, and applied actions.
6. Replace clipboard contents with cleaned output.

### Non-goals (v1)
- Cloud sync of snippet history.
- Full GUI editor for rule authoring.
- IDE/editor plugins.
- OCR/image clipboard processing.

---

## 4) Target Users
- Developers who frequently copy code from terminals, REPLs, SSH sessions, and CLI tools.

---

## 5) Key Use Cases
1. User copies a stack trace block from terminal; app strips timestamps/prompts and leaves only code lines.
2. User copies shell command output with prompt prefixes (`$`, `PS>`, `➜`); app removes prompts while preserving command text.
3. User copies text from browser/editor; app does nothing unless terminal context is detected.

---

## 6) Functional Requirements

### FR-1: Tray App Lifecycle
- App starts minimized to tray.
- Tray menu includes: Enable/Disable monitoring, Open logs folder, Reload rules, Quit.
- Optional start-on-login setting.

### FR-2: Clipboard Monitoring
- Observe clipboard text changes continuously.
- Ignore non-text clipboard payloads in v1.
- Debounce duplicate events (same content hash within short window).

### FR-3: Terminal Context Detection
- On clipboard change, capture active/focused window metadata:
  - process name
  - window title
  - window class/app id (when available)
- Match against a configurable terminal signature list (default includes common terminals per OS).
- Only run sanitizer when terminal context confidence passes threshold.

### FR-4: Heuristic Cleanup Engine
- Rule-driven cleaning pipeline with ordered steps.
- Each rule can be enabled/disabled and configured without recompiling.
- Built-in rule types (v1):
  - remove ANSI escape/control chars
  - strip shell prompts (bash/zsh/fish/powershell/cmd)
  - strip REPL markers (`>>>`, `...`)
  - normalize indentation and tabs
  - trim leading/trailing blank lines
  - remove wrapped line-number prefixes
- Pipeline must preserve line breaks and code structure as much as possible.

### FR-5: Clipboard Rewrite
- If output differs from input, overwrite clipboard with cleaned text.
- Prevent rewrite loops using event origin markers or content hashes.

### FR-6: Logging & Audit
- Log each processed event:
  - timestamp
  - context metadata
  - input hash + optional truncated preview
  - output hash + optional truncated preview
  - applied rules/actions
  - duration and status
- Store logs locally (JSONL in v1).
- Include log rotation/retention policy.

### FR-7: Config & Rule Management
- Config file in user config directory.
- Runtime “Reload rules” from tray menu.
- Graceful fallback to defaults if config invalid.

---

## 7) Non-Functional Requirements
- Cross-platform support: Windows, Linux, macOS.
- Startup latency < 2s on typical dev machine.
- Clipboard processing latency target: < 50ms average for <= 20KB text.
- Privacy-first: local-only processing, no network calls by default.
- Reliable under frequent clipboard updates.

---

## 8) Platform Scope (v1)
- Windows: include Windows Terminal, PowerShell, cmd, ConEmu, WezTerm, Alacritty.
- Linux: GNOME Terminal, Konsole, Kitty, Alacritty, WezTerm, xterm (best-effort across X11/Wayland).
- macOS: Terminal.app, iTerm2, Warp, Alacritty, WezTerm.

---

## 9) Risks & Mitigations
1. **Foreground window APIs differ per OS**
   - Mitigation: OS-specific adapters with unified confidence scoring and fallback behavior.
2. **False positives/false negatives in terminal detection**
   - Mitigation: configurable signatures, logging for tuning, optional strict mode.
3. **Over-cleaning valid text**
   - Mitigation: conservative defaults, per-rule toggles, dry-run debug mode in logs.
4. **Clipboard event loops**
   - Mitigation: self-write guard (hash/time window + source marker).

---

## 10) MVP Acceptance Criteria
- App runs in tray on all 3 OS targets.
- Clipboard text copied from recognized terminal gets cleaned and replaced.
- Text copied from non-terminal apps remains unchanged.
- Logs include applied actions and can be inspected from tray menu.
- Rules can be changed in config and reloaded without restart.

---

## 11) Future Enhancements
- Rule testing sandbox UI.
- Per-terminal profile tuning.
- Snippet history browser UI.
- Structured metrics dashboard.
- Optional encrypted local database.
