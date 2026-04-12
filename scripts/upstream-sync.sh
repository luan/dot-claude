#!/usr/bin/env bash
# Sync commons plugin with upstream (luan/dot-claude).
# Usage: ./scripts/upstream-sync.sh [--dry-run]
#
# What it does:
#   1. Fetches upstream/main
#   2. Replaces shared skills, rules, hooks, ct, and config with upstream versions
#   3. Adds new upstream skills/rules
#   4. Deletes local skills/rules that upstream removed
#   5. Auto-preserves our unique additions (anything local that upstream never had)
#   6. Builds ct, reinstalls the plugin
#
# Conflict strategy: upstream wins for shared content. Local-only content is untouched.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

DRY_RUN=false
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true

# --- Helpers -----------------------------------------------------------------
info()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33m==>\033[0m %s\n' "$*"; }
error() { printf '\033[1;31m==>\033[0m %s\n' "$*"; exit 1; }

in_array() {
  local needle="$1"; shift
  for item in "$@"; do [[ "$item" == "$needle" ]] && return 0; done
  return 1
}

if [[ -n "$(git status --porcelain)" ]]; then
  error "Working tree is dirty. Commit or stash first."
fi

# --- Step 1: Fetch upstream --------------------------------------------------
info "Fetching upstream..."
git fetch upstream

UPSTREAM_SHA="$(git rev-parse upstream/main)"
UPSTREAM_SHORT="$(git rev-parse --short upstream/main)"
LAST_SYNC_TAG="$(git tag -l 'upstream-sync/*' --sort=-version:refname | head -1)"
if [[ -n "$LAST_SYNC_TAG" ]]; then
  LAST_SYNC_SHA="$(echo "$LAST_SYNC_TAG" | sed 's|upstream-sync/||')"
  LAST_SYNC_DATE="$(git log -1 --format=%ai "$LAST_SYNC_TAG" 2>/dev/null | cut -d' ' -f1)"
  info "Last sync: $LAST_SYNC_TAG ($LAST_SYNC_DATE)"
  NEW_COMMITS="$(git log --oneline "${LAST_SYNC_SHA}..upstream/main" 2>/dev/null | wc -l)"
  info "New upstream commits since last sync: $NEW_COMMITS"
  if [[ "$NEW_COMMITS" -eq 0 ]]; then
    info "Already up to date with upstream/main ($UPSTREAM_SHORT)."
    exit 0
  fi
else
  info "No previous sync tag found. Performing full sync."
fi
info "Syncing to upstream/main ($UPSTREAM_SHORT)"

# --- Step 2: Discover what each side has -------------------------------------
info "Discovering skills and rules..."

mapfile -t UPSTREAM_SKILLS < <(git ls-tree --name-only upstream/main skills/ | sed 's|skills/||')
mapfile -t UPSTREAM_RULES  < <(git ls-tree --name-only upstream/main rules/ | sed 's|rules/||')
mapfile -t OUR_SKILLS      < <(ls -1 skills/)
mapfile -t OUR_RULES       < <(ls -1 rules/)

# What upstream had last time we synced (from merge-base).
# Anything that existed in upstream at the merge-base but is gone now = intentionally removed.
# Anything local that NEVER existed upstream = ours to keep.
MERGE_BASE="$(git merge-base HEAD upstream/main)"
mapfile -t BASE_SKILLS < <(git ls-tree --name-only "$MERGE_BASE" skills/ 2>/dev/null | sed 's|skills/||' || true)
mapfile -t BASE_RULES  < <(git ls-tree --name-only "$MERGE_BASE" rules/ 2>/dev/null | sed 's|rules/||' || true)

# --- Step 3: Compute sync plan ----------------------------------------------
# Sync from upstream: everything upstream has
CHECKOUT_SKILLS=("${UPSTREAM_SKILLS[@]}")
CHECKOUT_RULES=("${UPSTREAM_RULES[@]}")

# Delete: exists locally, existed in upstream at merge-base, but upstream removed it
DELETE_SKILLS=()
for s in "${OUR_SKILLS[@]}"; do
  in_array "$s" "${UPSTREAM_SKILLS[@]}" && continue  # still in upstream
  in_array "$s" "${BASE_SKILLS[@]}" || continue       # never was in upstream = ours
  DELETE_SKILLS+=("$s")
done

DELETE_RULES=()
for r in "${OUR_RULES[@]}"; do
  in_array "$r" "${UPSTREAM_RULES[@]}" && continue
  in_array "$r" "${BASE_RULES[@]}" || continue
  DELETE_RULES+=("$r")
done

# Ours: exists locally, never existed in upstream
OWN_SKILLS=()
for s in "${OUR_SKILLS[@]}"; do
  in_array "$s" "${UPSTREAM_SKILLS[@]}" && continue
  in_array "$s" "${BASE_SKILLS[@]}" && continue
  OWN_SKILLS+=("$s")
done

OWN_RULES=()
for r in "${OUR_RULES[@]}"; do
  in_array "$r" "${UPSTREAM_RULES[@]}" && continue
  in_array "$r" "${BASE_RULES[@]}" && continue
  OWN_RULES+=("$r")
done

# --- Step 4: Print plan ------------------------------------------------------
echo ""
info "Sync plan:"
echo "  Skills from upstream: ${#CHECKOUT_SKILLS[@]}"
for s in "${CHECKOUT_SKILLS[@]}"; do echo "    + skills/$s/"; done

if [[ ${#DELETE_SKILLS[@]} -gt 0 ]]; then
  echo "  Skills to delete (upstream removed): ${#DELETE_SKILLS[@]}"
  for s in "${DELETE_SKILLS[@]}"; do echo "    - skills/$s/"; done
fi

if [[ ${#OWN_SKILLS[@]} -gt 0 ]]; then
  echo "  Skills preserved (ours): ${OWN_SKILLS[*]}"
fi

echo ""
echo "  Rules from upstream: ${#CHECKOUT_RULES[@]}"
for r in "${CHECKOUT_RULES[@]}"; do echo "    + rules/$r"; done

if [[ ${#DELETE_RULES[@]} -gt 0 ]]; then
  echo "  Rules to delete (upstream removed): ${#DELETE_RULES[@]}"
  for r in "${DELETE_RULES[@]}"; do echo "    - rules/$r"; done
fi

if [[ ${#OWN_RULES[@]} -gt 0 ]]; then
  echo "  Rules preserved (ours): ${OWN_RULES[*]}"
fi

echo ""
echo "  Also syncing: tools/crates/ct/, tools/Cargo.toml, tools/Cargo.lock,"
echo "                hooks/, plugins/config.json, rule-creator/, README.md, .gitignore"
echo ""

if $DRY_RUN; then
  if [[ -n "$LAST_SYNC_TAG" ]]; then
    echo "  New upstream commits:"
    git log --oneline "${LAST_SYNC_SHA}..upstream/main" 2>/dev/null | sed 's/^/    /'
    echo ""
  fi
  info "Dry run -- stopping here."
  exit 0
fi

# --- Step 5: Execute sync ----------------------------------------------------
info "Syncing skills..."
for s in "${CHECKOUT_SKILLS[@]}"; do
  git checkout upstream/main -- "skills/$s/"
done

for s in "${DELETE_SKILLS[@]}"; do
  rm -rf "skills/$s"
done

git add skills/
git commit -m "refactor(skills): upstream sync -- ${#CHECKOUT_SKILLS[@]} synced, ${#DELETE_SKILLS[@]} deleted" || true

info "Syncing rules..."
for r in "${CHECKOUT_RULES[@]}"; do
  git checkout upstream/main -- "rules/$r"
done

for r in "${DELETE_RULES[@]}"; do
  rm -f "rules/$r"
done

git add rules/
git commit -m "refactor(rules): upstream sync -- ${#CHECKOUT_RULES[@]} synced, ${#DELETE_RULES[@]} deleted" || true

info "Syncing ct toolchain..."
git checkout upstream/main -- tools/crates/ct/ tools/Cargo.toml tools/Cargo.lock
git add tools/
git commit -m "refactor(ct): sync with upstream" || true

info "Syncing hooks..."
mapfile -t UPSTREAM_HOOKS < <(git ls-tree --name-only upstream/main hooks/ | sed 's|hooks/||')
for f in hooks/*; do
  fname="$(basename "$f")"
  in_array "$fname" "${UPSTREAM_HOOKS[@]}" || rm -f "$f"
done
for h in "${UPSTREAM_HOOKS[@]}"; do
  git checkout upstream/main -- "hooks/$h"
done
git add hooks/
git commit -m "chore(hooks): sync with upstream" || true

info "Syncing config files..."
git checkout upstream/main -- plugins/config.json rule-creator/ README.md .gitignore 2>/dev/null || true
git add plugins/ rule-creator/ README.md .gitignore
git commit -m "chore: sync config files with upstream" || true

# --- Step 6: Build ct --------------------------------------------------------
info "Building ct..."
(cd tools && cargo build)

info "Installing ct..."
(cd tools && cargo install --path crates/ct)

# --- Step 7: Refresh plugin cache --------------------------------------------
info "Refreshing plugin cache..."
claude plugin install commons@local

# --- Step 8: Record sync point -----------------------------------------------
git tag -f "upstream-sync/$UPSTREAM_SHA" -m "Synced to upstream/main at $UPSTREAM_SHORT"
info "Tagged sync point: upstream-sync/$UPSTREAM_SHA"

# --- Done --------------------------------------------------------------------
echo ""
info "Upstream sync complete. Commits:"
git log --oneline "$MERGE_BASE..HEAD" | head -20
echo ""
warn "Review the commits, then push when ready."
