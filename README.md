# pi-clipper

pi-clipper is a tray-first Tauri app that watches clipboard changes, detects when copied text came from a terminal, removes common terminal noise, and writes a cleaner version back to the clipboard.

It is built for developers who regularly copy commands, code snippets, JSON, and logs from terminals and want cleaner paste results with minimal friction.

## Status

**Current release status: early MVP**

What works today:
- Windows-focused terminal detection
- Clipboard monitoring with dedupe and self-write loop protection
- Deterministic rule pipeline for terminal cleanup
- Optional structured JSONL event logging
- Tray-only app with enable/disable, open logs folder, and quit actions
- Regression tests for sanitizer behavior

Current limitations:
- **Windows is the only implementation-ready platform right now**
- macOS and Linux adapters exist only as placeholders
- Tray "Reload rules" is not fully wired yet
- No packaged public release artifacts yet; build from source

## Features

- Detects clipboard text changes in near real time
- Checks whether the active app was a terminal before cleaning
- Removes common terminal artifacts such as:
  - ANSI escape sequences
  - shell prompts
  - PowerShell prompts
  - leading line numbers
  - one shared leading space across copied blocks
- Preserves multi-line structure for code and JSON
- Avoids clipboard rewrite loops
- Optional structured local logs for inspection and debugging
- Uses local-only processing; no outbound network behavior in the app

## Supported terminals

Current Windows matching includes:
- Windows Terminal
- PowerShell / pwsh
- cmd
- WezTerm
- Alacritty
- conhost-hosted shells

## How it works

Pipeline overview:

1. Watch clipboard text changes
2. Hash and dedupe recent events
3. Detect the currently focused application/window
4. If terminal confidence is high enough, run the sanitizer pipeline
5. If output changed, write cleaned text back to clipboard
6. Optionally log the decision and applied actions to JSONL

## Build from source

### Prerequisites

You will need:
- [Node.js](https://nodejs.org/)
- npm
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri build prerequisites for your OS

For Tauri setup, follow the official prerequisites:
- https://tauri.app/start/prerequisites/

### Install dependencies

```bash
npm install
```

### Run in development

```bash
npm run dev
```

### Build a release binary

```bash
npm run build
```

### Windows helper scripts

```powershell
.\run.bat
.\run-verified.bat
```

`run.bat` builds and launches a fresh release build.

`run-verified.bat` runs the clipboard regression tests first, then launches the app.

## Testing

Run the clipboard-focused regression suite:

```bash
npm run verify:clip
```

Or run Rust tests directly:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## Usage

1. Launch the app
2. It starts as a tray application with no main window
3. Copy text from a supported terminal
4. Paste anywhere to verify the cleaned result
5. Use the tray menu to:
   - enable or disable monitoring
   - open the logs folder
   - quit the app

## Rules

Built-in rules are defined in:
- `src-tauri/src/rules/builtins.rs`

Default config files live in:
- `config/default-rules.yaml`
- `config/default-signatures.yaml`

For deeper rule details and examples, see:
- `RULES.md`

## Logging and privacy

- Processing is local only
- No clipboard history is uploaded anywhere
- Event logging is **disabled by default**
- Full clipboard content is **not** stored by default
- When enabled, logs contain hashes, context metadata, truncated previews, rule actions, status, and duration
- When enabled, logs are stored as JSONL with rotation/retention behavior
- Enable logging by setting `event_logging_enabled` to `true` in the app config JSON

## Project docs

- `PRD.md` — product requirements
- `ARCHITECTURE.md` — system design
- `AGENTS.md` — execution plan and ownership
- `RULES.md` — rule configuration guidance

## Roadmap

Near-term priorities:
1. Finish runtime rule reload
2. Improve Windows detection coverage and confidence tuning
3. Implement real Linux adapter support
4. Implement real macOS adapter support
5. Add packaged release artifacts
6. Reduce compiler warnings and harden diagnostics

## Repository

GitHub repository:
- `https://github.com/fink-andreas/pi-clipper`

## License

MIT. See `LICENSE`.
