# Work Approval — Explicit User Consent Required

`work approve` is a user-acceptance gate. NEVER call it without
explicit user consent.

## When Approval Happens

Only when the user explicitly chooses an "Approve" option (e.g. in
/review continuation prompt) or directly requests it.

## Forbidden

- Auto-approving after build gate passes
- Auto-approving after /review passes (review clean ≠ user accepted)
- Any skill approving work without user selecting an approve action
- `work review && work approve` in the same command

## Semantics

- `work review` = "implementation done, ready for review" (worker/skill
  signal)
- `work approve` = "user reviewed + accepted" (requires explicit user
  action)
