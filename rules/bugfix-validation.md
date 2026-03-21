# Bugfix Runtime Validation

When production reproduction data is available (from /dia-inspect-data, debug logs, server queries, or user-provided evidence), passing tests is necessary but NOT sufficient. Validate the fix against production data before declaring success.

Red flags that the bug is not actually fixed:
- **Test-only changes** — zero production code modified means the tests describe the fix, not implement it
- **Reproduction data not re-checked** — user provided live evidence of the bug and no one verified it's gone

After develop completes a bugfix with available reproduction data, re-run the reproduction scenario (same command, same query, same inspection) to confirm the bug is actually fixed before proceeding to review.

Origin: /vibe committed test-only changes and declared "PASS" on review while the production bug persisted. User discovered this manually via /dia-inspect-data. Cost ~2 hours of additional debugging.
