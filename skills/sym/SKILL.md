---
name: sym
description: "Tree-sitter indexed code navigator (ct sym CLI). Use INSTEAD OF Read/Grep/Glob/Bash when exploring existing code, locating a symbol, tracing the call graph up (impact) or down (trace), finding implementations of an interface, scoping a diff to one symbol, or preparing to edit code you have not read yet. Triggers: 'where is X defined', 'who calls X', 'what does X call', 'find implementations of', 'what breaks if I change X', 'outline this file', 'map imports', 'show me this symbol', exploring unfamiliar repo, tracing call graph, scoping diff to a symbol, preparing to edit code I haven't read. Do NOT use for: writing new code from scratch, editing prose or config, running tests, or when a stack trace already names the file and line."
---

# sym — code navigation

## Default investigation loop

```
search  →  investigate / context  →  impact / trace / refs / impls  →  show / outline
                                                                              │
                                                                              └─ then targeted reads or rg
```

1. **Find it** — `ct sym search <query>` (add `--exact`, `--kind`, `--lang`, `--path`).
2. **Understand it** — `ct sym investigate <symbol>` (kind-adaptive: function → source + callers + shallow impact; type → source + members + references). Use `ct sym context <symbol>` only when you specifically want source + callers + imports bundled.
3. **Trace flow** — `ct sym impact` (upward), `ct sym trace` (downward), `ct sym refs` (direct uses), `ct sym impls` (who implements this).
4. **Read** — `ct sym show <symbol>` or `ct sym outline <file>` before `Read` / `rg`.

## Goal → Command

| I want to… | Command |
|---|---|
| Find a symbol or text | `ct sym search <q>` (symbols) / `--text` (grep, delegates to rg) |
| Get kind-appropriate context for a symbol | `ct sym investigate <symbol>` |
| Read source + refs + callers + imports together | `ct sym context <symbol>` |
| Read source by symbol or file range | `ct sym show <symbol \| file[:L1-L2]>` |
| List symbols in a file | `ct sym outline <file>` (`-s` for signatures, `--names` for piping) |
| Find direct references | `ct sym refs <symbol>` |
| See who depends on a symbol transitively | `ct sym impact <symbol>` |
| See what a symbol calls | `ct sym trace <symbol>` |
| Find types that implement an interface | `ct sym impls <symbol>` |
| See git diff for a symbol | `ct sym diff <symbol> [base]` (add `--stat`) |
| Find files importing a file/package | `ct sym importers <file\|pkg>` |
| Get a map of the repo | `ct sym structure` |
| List file tree / stats / indexed repos | `ct sym ls` / `--stats` / `--repos` |

Every command supports `--json`. sym resolves the right DB automatically per
repo and auto-refreshes the index when stale on each query — **do not run
`ct sym index` manually**.

## Command details

### `search` — starting point

```
ct sym search OpenStore
ct sym search parse --kind function --lang go
ct sym search "TODO" --text                              # full-text grep (uses rg)
ct sym search Handler --path 'internal/**' --exclude '**/*_test.go'
```

Ranked by symbol kind (class/struct/interface > function > method) and
path quality (src/pkg/lib/internal bump; test/vendor/generated/docs penalty).
Trust the first result on most queries.

### `investigate` — one call, right-shaped answer

```
ct sym investigate OpenStore
ct sym investigate config.go:Config       # file hint
ct sym investigate auth.Middleware        # parent/package hint
ct sym investigate Foo Bar Baz            # batch
```

Ambiguous names auto-resolve and list alternatives in `also` / `matches`. Use
this **before** `show`/`refs`/`context` on unfamiliar symbols.

### `context` — bundled read

```
ct sym context OpenStore
ct sym context ParseFile --callers 10
```

`--callers` is the only knob (default 20). Use when you already know the
symbol matters and want one payload. **Do not call both `investigate` and
`context` on the same symbol** — pick one.

### `show` — read source

```
ct sym show ParseFile
ct sym show internal/index/store.go
ct sym show internal/index/store.go:80-120
ct sym show Foo Bar Baz                                  # batch
ct sym outline big.go -s --names | ct sym show --stdin
ct sym show Handler --all                                # every definition
```

Supports `-C` context lines, `--path`, `--exclude`, `--all`, `--stdin`.

### `outline` — file map

```
ct sym outline internal/index/store.go
ct sym outline internal/index/store.go --signatures
ct sym outline internal/index/store.go -s --names        # one symbol per line, pipe-ready
```

Read this **before** opening large or unfamiliar files. The `--names` form
is the engine for batch mode in `show`/`refs`/`trace`/`impact`.

### `refs` — direct references

```
ct sym refs ParseFile
ct sym refs ParseFile --file internal/                   # scope to a path fragment
ct sym refs ParseFile -C 0                               # single-line format
ct sym refs ParseFile --importers                        # files importing the defining file
ct sym refs ParseFile --impact                           # = --importers --depth 2
ct sym refs Foo Bar Baz                                  # batch
ct sym refs Foo --path 'cmd/**' --exclude '**/testdata/**'
```

Default `-C 1` shows one line of surrounding context around each call site,
grouped per (file, call text). `--depth` capped at 3. Best-effort AST name
matching, not semantic analysis — narrow with `--path` or `--file` when
names collide across packages.

### `impact` — upward, transitive

```
ct sym impact handleRegister
ct sym impact handleRegister -D 3 -C 2
ct sym impact Save Load Delete                           # union, attributed via hit_symbols
ct sym outline store.go -s --names | ct sym impact --stdin
```

### `trace` — downward call chain

```
ct sym trace handleRegister
ct sym trace handleRegister --depth 5
ct sym trace handleRegister --kinds call,use             # default: call only
ct sym outline svc.go -s --names | ct sym trace --stdin
```

### `impls` — who implements / extends / conforms

```
ct sym impls Handler                  # Java/C#/Kotlin/TS implements, Go embedding,
                                      # Swift conformance, Rust impl, Python bases,
                                      # Ruby include/extend, PHP implements, C++ bases
```

Externally-defined targets come back with `resolved=false`.

### `diff` — git diff scoped to a symbol

```
ct sym diff ParseFile                 # vs HEAD
ct sym diff ParseFile main            # vs branch
ct sym diff --stat ParseFile          # diffstat only
```

### `structure` / `ls` / `importers`

```
ct sym structure                      # entry points, hotspots, central packages
ct sym ls --stats                     # languages, file/symbol counts
ct sym ls --repos                     # all indexed repos
ct sym importers internal/index       # fan-in for a package
```

## Path filtering

`search`, `show`, and `refs` accept repeatable `--path` / `--exclude` globs.
Compose with `--kind` / `--lang`:

```
ct sym search Handler --lang go --path 'internal/**' --exclude '**/*_test.go'
ct sym refs OpenStore --path 'cmd/**'
```

On large repos this is the difference between useful and useless output.
(`context`, `investigate`, `trace`, `impact`, `impls` do not take `--path`;
filter at the consumer step instead.)

## JSON mode

`--json` works on every command. Look for:

- `also` / `matches` — alternate resolutions when a name is ambiguous.
- `ambiguous: true` — sym picked one; `also` lists the rest.
- `hit_symbols` — in batch mode (`refs`/`impact`/`trace`), which input brought each result in.
- `resolved: false` — in `impls`, the target is external (framework/stdlib).
- `context` — on `refs` / `impact`, surrounding source lines around each site.

## Pivot rule

If one or two searches miss, **stop searching synonyms.** Pivot to
implementation seams instead:

> spec · registry · bundle · runtime · policy · session · state · dispatch · config · store · manifest · descriptor · provider · handler

Run `ct sym structure` to surface the actual entry points and hotspots, then
`investigate` the seam symbols you find.

## Don't

- **Don't `rg`/`grep`/`Read` for a symbol sym can resolve directly.** Use `search` → `investigate`/`show`.
- **Don't `Read` a large (>500-line) file without `outline` first.** Outline it, then `show` the slice.
- **Don't run `ct sym index` manually.** Every query auto-refreshes on staleness.
- **Don't chain `investigate` → `context` → `show` on the same symbol.** Pick one; it already answers the question.
- **Don't call `show` before `investigate` on an unfamiliar symbol.** Investigate first for the right-shaped context.
- **Don't retry searches with synonyms more than twice.** Pivot to seam names (see above).
- **Don't treat `refs` as semantic "find all callers".** It is AST name matching — narrow with `--path`/`--file` when names collide.
- **Don't retry on ambiguity errors.** Read `also` / `matches` in the output and pick.
- **Don't hand-list batch inputs.** `outline -s --names | <cmd> --stdin` is built for it.
- **Don't paginate through `refs`/`impact` output.** Narrow with `--limit` or `--path`.

## Real constraints

- `refs` / `impact` / `trace` are AST name matching. Cross-package name
  collisions inflate results — narrow with `--path` or `--file`.
- `impls` returns `resolved=false` for externally-defined interfaces.
- TypeScript and other non-Go languages may have incomplete signature data.
- Imports are resolved best-effort per language; cross-language edges are
  not modeled.
- `search --text` delegates to `rg`; it does **not** use the symbol index.

## Outcome

Start with sym. Pivot on misses. Trust the first rank. Read last.
