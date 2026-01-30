---
name: stack-nav
description: Navigate between branches in stack (gt up, gt down, gt top, gt bottom)
user-invocable: true
allowed-tools:
  - "Bash(gt up:*)"
  - "Bash(gt u:*)"
  - "Bash(gt down:*)"
  - "Bash(gt d:*)"
  - "Bash(gt top:*)"
  - "Bash(gt t:*)"
  - "Bash(gt bottom:*)"
  - "Bash(gt b:*)"
  - "Bash(gt checkout:*)"
  - "Bash(gt co:*)"
  - "Bash(gt log:*)"
  - "Bash(gt ls:*)"
---

# Stack Navigation

Navigate between branches in the stack.

## Terminology

```
main (trunk)
  │
  └── feature-1  ← BOTTOM (base, closest to main)
        │
        └── feature-2  ← middle
              │
              └── feature-3  ← TOP (tip, furthest from main)
```

- **up** = toward children/top (away from main)
- **down** = toward parent/bottom (toward main)

## Commands

```bash
gt up [n]       # Move n branches up (toward tip), alias: gt u
gt down [n]     # Move n branches down (toward trunk), alias: gt d
gt top          # Jump to stack tip, alias: gt t
gt bottom       # Jump to stack base, alias: gt b
gt checkout     # Interactive branch picker, alias: gt co
gt log          # View stack structure, alias: gt ls
```

## Usage

| User Says | Command |
|-----------|---------|
| "go to top" / "go to tip" | `gt top` |
| "go to bottom" / "go to base" | `gt bottom` |
| "go up" / "next branch" | `gt up` |
| "go down" / "parent branch" | `gt down` |
| "show stack" | `gt log` |
