---
paths:
  - "**/*.swift"
---

## Pattern

Use `fputs("...\n", stderr)` for debug logging in Swift.
This prints to stderr immediately (no buffering issues),
visible in Xcode console and terminal, and doesn't go
through the Logger system.

## Format

```swift
fputs("[ComponentName] actionDescription key=\(value)\n", stderr)
```

## When to Use

- Debugging startup sequencing (which code path taken)
- Quick iteration — faster than adding Logger + rebuilding

## Cleanup

Remove `fputs` calls before merging to main.
They're debug-only, not permanent logging.
