# Session Completion ("Landing the Plane")

When ending work session, ALL steps mandatory:

1. File remaining work as beads issues
2. Run quality gates (tests, linters, build) if code changed
3. Update beads status — close finished, update in_progress
4. Sync + push:
   ```bash
   bd sync
   git push
   ```
5. Verify: `git status` must show up-to-date with origin

## Hard Rules

- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing — leaves work stranded locally
- NEVER say "ready to push when you are" — YOU push
- Push fails → resolve + retry until success
