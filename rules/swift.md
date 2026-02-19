---
paths:
  - "**/*.swift"
---

# Swift

## Debug Logging

Use `fputs("...\n", stderr)` for debug logging.
Prints to stderr immediately (no buffering), visible in Xcode console + terminal, bypasses Logger system.

```swift
fputs("[ComponentName] actionDescription key=\(value)\n", stderr)
```

Remove `fputs` calls before merging to main — debug-only.
