---
name: vault-sweep
description: "Comprehensive vault maintenance — cross-references blueprints against codebase state to produce a maintenance plan: archive consumed artifacts, audit docs for staleness, propose new docs for undocumented stable systems. Triggers: 'vault sweep', 'sweep the vault', 'clean up vault', 'vault maintenance', 'what can we archive', 'audit blueprints', 'vault hygiene', 'blueprint cleanup'. Use whenever the user wants a holistic view of vault health rather than archiving a single artifact (that's /archive). Also use when the user asks what's stale, what needs docs, or whether artifacts can be cleaned up."
argument-hint: "[--execute] [--docs-only] [--no-gap-analysis]"
allowed-tools:
  - Agent
  - Bash
  - Read
  - Glob
  - Grep
user-invocable: true
---

# Vault Sweep

Holistic vault maintenance. Cross-references the blueprints vault against codebase reality to produce a structured maintenance plan: what to archive, what docs are stale, what stable systems deserve new reference docs.

Analytical counterpart to `/archive` (which operates on individual artifacts). Vault sweep surveys the whole project, classifies everything, and proposes. Nothing executes until approved.

## Arguments

- `--execute` — skip the approval gate, execute the plan immediately
- `--docs-only` — skip artifact archival, only audit docs + gap analysis
- `--no-gap-analysis` — skip the "propose new docs" phase

## Phase 1: Inventory

List all active artifacts for the current project using JSON output for reliable parsing:

```bash
PROJECT_ROOT=$(git rev-parse --show-toplevel)
PROJECT_NAME=$(ct vault project)
ct spec list --json && ct plan list --json && ct review list --json && ct report list --json && ct doc list --json
```

Parse the JSON arrays into a combined working list. The enriched JSON includes `title`, `created`, `source`, and `tags` per artifact. Group artifacts by **topic** — the `title` field is the topic slug. Use `source` links to connect related artifacts (a plan's source points to its spec). Build a topic map:

```
topic → [{kind: "spec", name: "...", created: "...", source: "..."}, {kind: "plan", ...}, ...]
```

This grouping ensures related artifacts get classified and archived together. If the vault is empty or `ct` is not installed, report and stop.

## Phase 2: Cross-reference

Determine which features are shipped. Run these in parallel:

**a) Git history** — merged work since the earliest artifact (use the `created` field from Phase 1):

```bash
git log main --oneline --since="<earliest artifact created date>" | head -120
git branch --list --no-merged main | head -50
```

**b) Codebase verification** — dispatch an Explore subagent with the topic list:

```
Verify which features are fully implemented and merged to main in <project-root>.

For each topic, determine:
1. Does code for this feature exist on main? (search for key types, modules, files)
2. Are there merge commits or PRs referencing it in git log?
3. Are there open branches with unmerged work for it?

Topics to classify:
<topic list with artifact counts>

Output each topic as one of:
- MERGED — fully landed on main, no open branches
- IN-PROGRESS — partial code or open branch exists
- FUTURE — design-only, no corresponding code
- UNKNOWN — can't determine (explain why)
```

For UNKNOWN results, investigate manually before classifying. The subagent may miss features that landed under different names than the artifact topic — especially early foundation work. Cross-check against git log.

## Phase 3: Classify artifacts

Map cross-reference results onto individual artifacts:

| Classification | Criteria | Action |
|---|---|---|
| **Consumed** | Topic classified as MERGED | Archive candidate |
| **Active** | Topic is IN-PROGRESS | Leave alone |
| **Future** | Topic is FUTURE, no code | Leave alone |
| **Living** | Continuously updated docs (triage lists, needs-triage) | Leave alone |

Group consumed artifacts by feature area for the final report. One line per feature area is more useful than listing every artifact individually.

## Phase 4: Doc audit

Use `ct tool check-refs` for deterministic staleness scoring, then read docs that need qualitative judgment.

**Step 1 — Programmatic check:** For each doc, run:

```bash
ct tool check-refs <doc-stem> --project-root "$PROJECT_ROOT"
```

This extracts file paths, backtick identifiers, and crate names from the doc body, validates each against the project filesystem, and outputs a JSON report with a `staleness` score (0.0–1.0).

**Step 2 — Classify by staleness score:**

| Staleness | Initial classification |
|---|---|
| < 0.10 | Likely **Keep** — but still read docs describing frameworks or paradigms to check for wholesale replacement |
| 0.10 – 0.50 | Likely **Update** — read the doc to identify which sections are stale |
| > 0.50 | Likely **Archive** — read the doc to confirm wholesale replacement |

**Step 3 — Qualitative review:** Read ALL docs via `ct doc read <path>`, not just those with high staleness. A doc can score 0.00 staleness while describing an entirely replaced paradigm (e.g., a Dioxus architecture doc where the app migrated to egui — paths resolve because the module names were reused, but the framework context is wrong). The staleness score catches reference drift; qualitative review catches paradigm drift.

Classify each:

| Status | Criteria | Action |
|---|---|---|
| **Keep** | Low staleness, content matches codebase, paradigm current | No action |
| **Update** | Moderate staleness or identifiable stale sections | List what needs refreshing |
| **Archive** | High staleness or describes a replaced paradigm | Archive with reason |

For docs marked "update," be specific about which sections are stale and what changed in the codebase. Vague "needs updating" is not actionable.

## Phase 5: Gap analysis

Skip if `--no-gap-analysis`.

Identify stable, substantive systems that deserve reference docs. Use `ct tool churn` for deterministic data, then dispatch a subagent to assess documentation value.

**Step 1 — Gather stability data:**

```bash
ct tool churn --project-root "$PROJECT_ROOT" --since 2w --min-loc 500
```

This outputs per-module LOC, recent commit count, and last change date as JSON.

**Step 2 — Dispatch Explore subagent** with the churn data and doc list:

```
Assess which undocumented systems deserve reference docs.

Here is the per-module stability data (LOC, commits in last 2 weeks, last change):
<ct tool churn output>

Existing docs already cover: <list of doc topics from Phase 4>

For each module with 500+ LOC and low recent churn (≤5 commits in 2 weeks), evaluate:
- Is this system undocumented? (no corresponding doc in the vault)
- Would a developer reach for a reference doc repeatedly?
- What questions would the doc answer?

Be proactive — propose docs for every system that clears the bar. A locked binary
format, a rendering pipeline, a macro attribute reference, a theme system with 20+
configuration points — these all earn docs. Don't be conservative; the user will
cut proposals that don't make sense.

For each candidate, report:
- System name and module path
- LOC and stability evidence (from the data above)
- What the doc would cover
- Why it earns a doc

Also note systems with high churn as "revisit after stabilization" — don't propose
docs for actively churning code, but flag it for future sweeps.
```

The bar: would a developer reach for this doc repeatedly? Err on the side of proposing — the user will prune. Under-proposing is worse than over-proposing because it leaves undocumented systems invisible.

## Phase 6: Present plan

Output a structured maintenance plan with these sections:

**Archive candidates** — grouped by feature area:

| Feature Area | Types Present | Count |
|---|---|---|
| Entity/World stack | spec ×2, plan ×2, review ×3, report ×2 | 9 |
| Animation system | spec, plan, review ×2 | 4 |
| **Total** | | **N** |

**Doc actions** — existing docs with classification and staleness score:

| Doc | Staleness | Status | Rationale |
|---|---|---|---|
| architecture | 0.25 | Update | Missing crates: X, Y; "Planned" section lists shipped items |
| ui-crate | 0.85 | Archive | Describes Dioxus system, app uses egui since Apr 2 |

**New doc proposals** (from gap analysis):

| Proposed Doc | System | ~LOC | Commits (2w) | Justification |
|---|---|---|---|---|
| rendering-pipeline | crates/renderer | 4,890 | 2 | Settled wgpu pipeline, 4-level LOD, 27 shader types |

**Active artifacts** — what's being left alone and why. Being explicit here builds trust. List each active/future topic with its reason (open branch, future work, living document).

End with: **"Want me to execute this plan?"**

Skip straight to Phase 7 if `--execute` was passed.

## Phase 7: Execute

On approval (or `--execute`). Execute in this order — archival first, then updates, then new docs. Each step may affect wiki-links, so run `ct vault check` only at the end.

### 7a. Archive consumed artifacts

Use batch mode for efficiency:

```bash
ct <type> archive --batch <file1> <file2> ...
```

Group by artifact type (all specs together, all plans together, etc.). Each batch produces a single commit. Report progress by feature area ("Archived 22 Entity/World artifacts").

Preview first with `--dry-run` if the batch is large (>20 artifacts).

### 7b. Archive stale docs

```bash
ct doc archive <path>
```

### 7c. Update partially-stale docs

For each doc marked "update": read the current content via `ct doc read`, research the stale sections against the codebase, and apply targeted edits. Present each update for review unless `--execute` — doc updates require the most judgment.

Focus on:
- Renamed types/functions — update references to current names
- Missing crates/modules — add to architecture sections
- Framework references — correct if the framework changed
- Stale code examples — verify against current APIs

Do NOT rewrite docs wholesale. Targeted fixes to stale sections preserve the author's voice and structure.

### 7d. Create new reference docs

Each new doc is a **research and writing task**, not a copy-paste from archived artifacts. Dispatch a subagent per doc with explicit quality expectations:

```
Write a reference doc for <system> in <project-root>.

Read the actual source code — do NOT summarize archived specs or plans. The doc
must describe the system as it exists today.

Structure:
- **Overview**: What the system does, in 2-3 sentences.
- **Architecture**: Key types, their relationships, data flow. Use code references
  (backtick identifiers) that a developer can grep for.
- **Key decisions**: Non-obvious design choices and why they were made. What would
  surprise someone reading the code for the first time?
- **Common tasks**: How to add a new <thing>, modify <behavior>, debug <problem>.
  Concrete steps, not abstract guidance.

Quality bar: a developer who has never seen this codebase can read this doc and
understand the system well enough to make their first change without asking someone.

Output the doc body only (no frontmatter — ct adds that).
```

Store with a clean topic name (no timestamps for docs):

```bash
echo "<doc content>" | ct doc create --topic "<clean-topic-name>" --project "$PROJECT_ROOT" --tags "<domain-tags>"
```

Link related artifacts after creation:

```bash
RELATED=$(ct vault related --project "$PROJECT_ROOT" "<topic>")
# If non-empty, append ## Related section with wiki-links
```

### 7e. Fix broken wiki-links

Run `ct vault check` and review the output. Broken links fall into categories:

| Category | Action |
|---|---|
| Links to archived artifacts | Leave — Obsidian resolves across archive/ |
| Links to renamed files | Update the link to the new stem |
| Forward references to unwritten specs | Leave — they'll resolve when created |
| Stale cross-references | Remove the link |

Only fix links in active (non-archived) artifacts. Don't chase links inside archived files.

### 7f. Final report

Report summary: N artifacts archived, N docs updated, N docs created, N docs archived, N links fixed.
