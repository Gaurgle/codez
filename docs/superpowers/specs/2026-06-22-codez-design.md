# codez - design

**Date:** 2026-06-22
**Status:** Approved, pre-implementation

## Summary

`codez` is an interactive terminal dictionary of status/error codes across
domains (http, exit, curl, git). It offers a searchable ratatui TUI with a
detail pane, plus a plain print mode for quick lookups and pipes. Rust, a
sibling to `clockz`, `noiz`, and `repoz`. The bash tool `httpz` covers HTTP
status codes only as a fast print-and-exit reference; `codez` is the broader,
interactive, multi-domain dictionary.

## Goals

- Browse and search codes across multiple domains in one place.
- A detail pane that pays off: longer explanation, a fix hint, a source ref.
- A fast non-interactive path: `codez 404` prints the answer and exits.
- Adding a domain or code is a single TOML edit.

## Non-goals (v1)

- No common/extended tiers (search + category filter handle volume).
- No fuzzy search (substring only).
- No clipboard integration.
- Domains beyond http/exit/curl/git (kotlin, android, rust, ... come later as
  new TOML files).

## Stack

- `ratatui` 0.29, `crossterm` 0.28, `clap` 4 (derive) - same versions as clockz.
- `serde` (derive) + `toml` - the only additions, for data loading.
- Install: `cargo install --path .` (like clockz). No install.sh.
- Target: stable Rust, edition 2021.

## CLI and mode resolution

One rule: **a query means "answer me" (print); no query means "browse" (TUI).**
Piping (non-TTY) or `--plain` always forces plain print.

| Command | Behavior |
|---|---|
| `codez` | TUI, all categories |
| `codez --http` | TUI, started on the http category filter |
| `codez 404` | plain-print matching entries with full detail, exit |
| `codez --git rejected` | plain lookup restricted to git |
| `codez ... \| less` | non-TTY → plain print |
| `codez --plain` | force plain print |

Resolution in `main.rs`:

```
interactive = stdout_is_tty && !flags.plain && query.is_none()
if interactive { tui::run(app) } else { plain::run(app, query) }
```

clap surface:
- positional `query: Option<String>` - a code or text to match.
- `--http`, `--exit`, `--curl`, `--git` - category filter (mutually used as a
  starting filter; if more than one is passed, last wins for v1).
- `--plain` - force non-interactive output.
- `-h/--help`, `-V/--version` via clap derive.

Exit codes (plain mode): 0 if at least one entry printed, 1 if a query matched
nothing.

## TUI

### Layout

```
┌ codez ──────────────────────────────────┐
│ search: 4_                  [ all ▸http ]│   search box + category tabs
├──────────────────────────────────────────┤
│ 400  Bad Request    Malformed input      │   filtered, scrollable list
│ 404▸ Not Found      Resource missing     │
│ 429  Too Many Req.  Rate limited         │
├──────────────────────────────────────────┤
│ 404 Not Found · 4xx Client               │   detail pane for selection
│ The server has no representation…        │
│ fix: check URL/path; may be auth-masked  │
│ ref: RFC 9110 15.5.5                     │
└ ↑↓ move  ⇥ category  esc clear/quit ──────┘
```

Three vertical regions: header (search input + category tabs), the entry list
(scrollable, selection highlighted), and the detail pane for the selected
entry. A one-line footer shows key hints.

### Interaction (modeless, fzf-style)

- printable char → append to the search query (live filter)
- `Backspace` → delete last query char
- `↑` / `↓` → move selection (clamped; list auto-scrolls to keep selection visible)
- `Tab` / `Shift-Tab` → cycle category filter: all → http → exit → curl → git → all
- `Esc` → clear the query if non-empty; quit if already empty
- `Ctrl-C` → quit
- `Enter` → no-op in v1 (reserved for future "copy code")

No vim modes: arrows and Tab/Esc are non-printable, so they never conflict with
typing into the search box.

## Data

### Model

```rust
#[derive(serde::Deserialize, Clone)]
struct Entry {
    code: String,        // "404", or a git slug like "non-fast-forward"
    name: String,
    group: String,       // "4xx Client", "Push errors", ...
    summary: String,     // shown in the list row
    detail: Option<String>,
    fix: Option<String>,
    #[serde(rename = "ref")]
    reference: Option<String>,
}

#[derive(Copy, Clone, PartialEq)]
enum Category { Http, Exit, Curl, Git }
```

`fix` and `ref` are optional per entry (200 OK needs neither). git entries
carry a non-numeric `code` slug; the list renderer shows whatever `code` holds.

### Storage

One TOML file per category in `data/`, embedded via `include_str!` and parsed
once at startup:

```toml
# data/http.toml
[[entry]]
code = "404"
name = "Not Found"
group = "4xx Client"
summary = "Resource missing"
detail = "The server has no representation for this resource."
fix = "Check the URL/path; may be auth-masked (see 403)."
ref = "RFC 9110 15.5.5"
```

`load_all()` returns the entries grouped by category. A parse failure is a
programming error (data is baked in), so it panics with the offending file
name - it can never happen in a shipped binary that built.

### v1 content

- **http**: the ~58 HTTP status codes (full set, reusing the httpz data plus the
  extended codes), grouped 2xx/3xx/4xx/5xx.
- **exit**: common shell exit codes - 0, 1, 2, 126, 127, 128, 130 (SIGINT),
  137 (SIGKILL), 139 (SIGSEGV), 143 (SIGTERM), grouped Success/Error/Signal.
- **curl**: common curl exit codes - 1, 3, 6, 7, 22, 23, 26, 28, 35, 47, 52,
  56, 60, 67, grouped by area.
- **git**: common failure messages as slugs - detached-head, non-fast-forward,
  merge-conflict, no-upstream, dirty-tree-checkout, etc., grouped by area, each
  with a fix hint.

Exact rows are enumerated in the implementation plan.

## File structure

```
codez/
  Cargo.toml
  data/
    http.toml
    exit.toml
    curl.toml
    git.toml
  src/
    main.rs      # clap CLI + mode resolution (TUI vs plain)
    model.rs     # Entry, Category, load_all() via include_str! + serde
    app.rs       # App state: query, selection, category filter, filtered()
    tui.rs       # crossterm setup, event loop, draw
    plain.rs     # plain print + lookup (manual ANSI, httpz-style)
    theme.rs     # Catppuccin Mocha palette (ratatui Colors)
  README.md
  LICENSE        # MIT, matching siblings
  docs/superpowers/...
```

Each unit has one responsibility: `model` owns data, `app` owns state and
filtering (pure, unit-testable), `tui` owns interactive rendering, `plain` owns
non-interactive output, `theme` owns colors, `main` wires them.

## Architecture / data flow

```
                ┌─ load_all() (model) ─ Vec<Entry> per Category
argv ─ clap ─┤
                └─ interactive? ─┬─ yes → tui::run(App)
                                 └─ no  → plain::run(App, query)

App.filtered() = entries
    .filter(category matches active filter)
    .filter(query empty OR matches code/name/summary/group, case-insensitive)
```

`App` holds: all entries, active `Category` filter (or All), `query: String`,
`selected: usize`, `scroll_offset: usize`. `App` exposes pure methods
(`apply_char`, `backspace`, `move_selection`, `cycle_category`, `filtered`)
that the TUI event loop and the tests both drive.

## Error handling

- Data parse errors: panic at startup with the file name (baked-in data; cannot
  occur in a built binary). Covered by a test that loads every file.
- Terminal setup/teardown: restore the terminal on all exit paths (raw mode +
  alternate screen), including on panic, mirroring clockz's `tui::run` guard.
- Plain mode with a query that matches nothing: print a "no match" line to
  stderr and exit 1.

## Testing

`cargo test`, behavior over implementation:

- **model**: every TOML file parses; `load_all()` yields the expected category
  counts; a known lookup (`404` → "Not Found") resolves.
- **app** (pure, no rendering): `apply_char`/`backspace` change the query;
  `filtered()` narrows by query and by category; `move_selection` clamps at
  both ends; `cycle_category` walks all→http→exit→curl→git→all.
- **plain**: lookup of `404` contains the name and detail; a miss returns a
  non-zero result; category-restricted lookup excludes other categories.
- **tui smoke**: render one frame into `ratatui::TestBackend` and assert the
  header and a known row appear in the buffer.

## Future (out of scope, noted for the data model)

- More domains (kotlin, android, rust) as new TOML files + a `Category` arm.
- Optional `tier` field for common/extended toggling.
- Fuzzy search (`fuzzy-matcher`).
- `Enter` copies the selected code to the clipboard.
