# codez

Interactive terminal dictionary of status and error codes - HTTP, shell exit, curl, git - with live search and a detail pane. A sibling to [clockz](https://github.com/Gaurgle/clockz), [noiz](https://github.com/Gaurgle/noiz), and [httpz](https://github.com/Gaurgle/httpz).

## What it does

```bash
codez            # browse everything in a searchable TUI
codez --http     # browse, started on HTTP status codes
codez 404        # quick lookup: print the match with detail, then exit
codez --git rejected   # lookup within a category
```

The rule: give it a query and it prints the answer and exits; give it none and it opens the interactive browser. Piping always prints plainly. Force plain output anytime with `--plain`.

Each entry carries a summary, a longer explanation, an optional fix hint, and a source reference.

### TUI keys

| Key | Action |
|-----|--------|
| type | filter live |
| `↑` / `↓` | move selection |
| `Tab` / `Shift-Tab` | cycle category (all/http/exit/curl/git) |
| `Esc` | clear search, or quit when empty |
| `Ctrl-C` | quit |

## Build

```bash
git clone https://github.com/Gaurgle/codez.git
cd codez
cargo install --path .
```

Requires [Rust](https://www.rust-lang.org/tools/install) (stable).

## Adding codes

Each domain is a TOML file in `data/` (`http.toml`, `exit.toml`, ...). Add an
`[[entry]]` block - `code`, `name`, `group`, `summary`, and optional `detail`,
`fix`, `ref` - and rebuild. New domains are a new TOML file plus a `Category`
arm.

## License

MIT
