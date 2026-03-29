# Git Workflow

## Pushing

Push only when the user explicitly requests it ("push", "ship", "submit", `/gt:submit`). Do not announce push decisions — no "Not pushing (not requested)" after commits. Just commit and move on.

Exception: `--auto` mode in autonomous pipeline skills (babysit, pr-ci, pr-comments) where pushing is part of the contract.

## Commit Discipline

During iterative refinement — adjustments after the main work is done, small tweaks the user requests — accumulate changes as unstaged edits. Do not make tiny incremental commits. Commit only when the user asks ("commit", "save", `/commit`).

Exception: autonomous pipelines (vibe, develop) where commits are structural to the workflow.

## Git vs Graphite

If on a Graphite-managed branch (the gt validation hook is active), use gt commands exclusively — never raw `git rebase`, `git push`, or `git checkout -b`. If not on a gt-managed branch, use git normally. Never mix git and gt operations on the same branch.
